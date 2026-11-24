use std::collections::BTreeSet;
use std::net::IpAddr;

use serde::Serialize;

use crate::protocol::discovery::{list_local_lan_ipv4s, DiscoveryManager, UDP_DISCOVERY_PORT};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkProfileInfo {
    pub interface_alias: String,
    pub profile_category: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FirewallHealthStatus {
    pub local_ipv4s: Vec<String>,
    pub profiles: Vec<NetworkProfileInfo>,
    pub udp_listener_healthy: bool,
    pub udp_port: u16,
    pub firewall_backend: String,
    pub firewall_inbound_status: String,
    pub firewall_summary: String,
    pub recommended_firewall_command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinuxFirewallInspection {
    pub backend: String,
    pub active: bool,
    pub allowed: bool,
    pub blocked: bool,
    pub query_succeeded: bool,
    pub recommended_command: String,
}

pub fn check_firewall_health(discovery: &DiscoveryManager) -> FirewallHealthStatus {
    let local_ipv4s: Vec<String> = list_local_lan_ipv4s()
        .into_iter()
        .map(|ip| ip.to_string())
        .collect();
    let profiles = list_lan_network_profiles();
    let listener_bound = discovery.is_udp_listener_healthy();
    let inspection = inspect_linux_firewall(UDP_DISCOVERY_PORT);

    let (firewall_inbound_status, firewall_summary, udp_listener_healthy) =
        evaluate_linux_firewall_verdict(listener_bound, &inspection, UDP_DISCOVERY_PORT);

    FirewallHealthStatus {
        local_ipv4s,
        profiles,
        udp_listener_healthy,
        udp_port: UDP_DISCOVERY_PORT,
        firewall_backend: inspection.backend,
        firewall_inbound_status,
        firewall_summary,
        recommended_firewall_command: inspection.recommended_command,
    }
}

fn list_lan_network_profiles() -> Vec<NetworkProfileInfo> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    if let Ok(ifaces) = if_addrs::get_if_addrs() {
        for iface in ifaces {
            if iface.is_loopback() {
                continue;
            }
            if let IpAddr::V4(v4) = iface.ip() {
                if v4.is_private() && seen.insert(iface.name.clone()) {
                    out.push(NetworkProfileInfo {
                        interface_alias: iface.name,
                        profile_category: "LAN".to_string(),
                    });
                }
            }
        }
    }
    out
}

pub fn evaluate_linux_firewall_verdict(
    listener_bound: bool,
    inspection: &LinuxFirewallInspection,
    udp_port: u16,
) -> (String, String, bool) {
    if !listener_bound {
        return (
            "UNCONFIGURED".to_string(),
            format!(
                "Could not bind UDP discovery port {udp_port} locally (another app may be using it). Use Connect by IP or a PIX1 pairing code."
            ),
            false,
        );
    }

    if !inspection.active || inspection.backend == "none" {
        return (
            "INACTIVE".to_string(),
            format!(
                "No active host firewall (ufw or firewalld) detected; UDP {udp_port} LAN auto-discovery is open."
            ),
            true,
        );
    }

    if !inspection.query_succeeded {
        return (
            "UNKNOWN".to_string(),
            format!(
                "Local discovery socket is listening and {} appears active, but its rules require root privileges to inspect. Run the command below if your Pixel isn't detected.",
                inspection.backend
            ),
            true,
        );
    }

    if inspection.blocked {
        return (
            "BLOCKED".to_string(),
            format!(
                "An inbound {} rule explicitly denies UDP {udp_port}. Allow UDP {udp_port} with the command below or use Connect by IP.",
                inspection.backend
            ),
            false,
        );
    }

    if inspection.allowed {
        return (
            "ALLOWED".to_string(),
            format!(
                "{} allows inbound UDP {udp_port} LAN discovery.",
                inspection.backend
            ),
            true,
        );
    }

    (
        "UNCONFIGURED".to_string(),
        format!(
            "Active {} firewall has no explicit rule allowing inbound UDP {udp_port}. Run the command below to allow Pixel auto-discovery, or use Connect by IP.",
            inspection.backend
        ),
        false,
    )
}

#[cfg(target_os = "linux")]
fn inspect_linux_firewall(udp_port: u16) -> LinuxFirewallInspection {
    let ufw_cmd = format!("sudo ufw allow {udp_port}/udp comment \"Pix Relay Discovery\"");
    let firewalld_cmd =
        format!("sudo firewall-cmd --add-port={udp_port}/udp --permanent && sudo firewall-cmd --reload");

    // 1. Check ufw first
    let ufw_conf_enabled = std::fs::read_to_string("/etc/ufw/ufw.conf")
        .map(|s| {
            s.lines()
                .any(|l| l.trim().eq_ignore_ascii_case("ENABLED=yes"))
        })
        .unwrap_or(false);

    let ufw_status_out = std::process::Command::new("ufw").arg("status").output().ok();
    let ufw_status_text = ufw_status_out
        .as_ref()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string());

    let ufw_active_from_cmd = ufw_status_text
        .as_deref()
        .map(|t| t.lines().any(|l| l.trim().eq_ignore_ascii_case("Status: active")))
        .unwrap_or(false);

    if ufw_conf_enabled || ufw_active_from_cmd {
        if let Some(ref text) = ufw_status_text {
            let (allowed, blocked) = parse_ufw_rules_for_udp_port(text, udp_port);
            return LinuxFirewallInspection {
                backend: "ufw".to_string(),
                active: true,
                allowed,
                blocked,
                query_succeeded: true,
                recommended_command: ufw_cmd,
            };
        }
        if let Ok(rules) = std::fs::read_to_string("/etc/ufw/user.rules") {
            let (allowed, blocked) = parse_ufw_rules_for_udp_port(&rules, udp_port);
            return LinuxFirewallInspection {
                backend: "ufw".to_string(),
                active: true,
                allowed,
                blocked,
                query_succeeded: true,
                recommended_command: ufw_cmd,
            };
        }
        return LinuxFirewallInspection {
            backend: "ufw".to_string(),
            active: true,
            allowed: false,
            blocked: false,
            query_succeeded: false,
            recommended_command: ufw_cmd,
        };
    }

    // 2. Check firewalld
    if let Ok(state_out) = std::process::Command::new("firewall-cmd")
        .arg("--state")
        .output()
    {
        let state_txt = String::from_utf8_lossy(&state_out.stdout)
            .trim()
            .to_ascii_lowercase();
        if state_out.status.success() && state_txt == "running" {
            if let Ok(ports_out) = std::process::Command::new("firewall-cmd")
                .arg("--list-ports")
                .output()
            {
                if ports_out.status.success() {
                    let ports_txt = String::from_utf8_lossy(&ports_out.stdout);
                    let allowed = parse_firewalld_ports_for_udp_port(&ports_txt, udp_port);
                    return LinuxFirewallInspection {
                        backend: "firewalld".to_string(),
                        active: true,
                        allowed,
                        blocked: false,
                        query_succeeded: true,
                        recommended_command: firewalld_cmd,
                    };
                }
            }
            return LinuxFirewallInspection {
                backend: "firewalld".to_string(),
                active: true,
                allowed: false,
                blocked: false,
                query_succeeded: false,
                recommended_command: firewalld_cmd,
            };
        }
    }

    LinuxFirewallInspection {
        backend: "none".to_string(),
        active: false,
        allowed: false,
        blocked: false,
        query_succeeded: true,
        recommended_command: ufw_cmd,
    }
}

#[cfg(not(target_os = "linux"))]
fn inspect_linux_firewall(udp_port: u16) -> LinuxFirewallInspection {
    LinuxFirewallInspection {
        backend: "none".to_string(),
        active: false,
        allowed: false,
        blocked: false,
        query_succeeded: true,
        recommended_command: format!("sudo ufw allow {udp_port}/udp comment \"Pix Relay Discovery\""),
    }
}

pub fn parse_ufw_rules_for_udp_port(text: &str, udp_port: u16) -> (bool, bool) {
    let port_str = udp_port.to_string();
    let port_udp = format!("{udp_port}/udp");
    let mut allowed = false;
    let mut blocked = false;

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        let low = line.to_ascii_lowercase();

        // /etc/ufw/user.rules tuple format:
        // ### tuple ### allow udp 42889 0.0.0.0/0 any 0.0.0.0/0 in
        if let Some(rest) = low.strip_prefix("### tuple ###") {
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() >= 3 && parts[1] == "udp" && parts[2] == port_str {
                if parts[0] == "allow" {
                    allowed = true;
                } else if parts[0] == "deny" || parts[0] == "reject" {
                    blocked = true;
                }
            }
            continue;
        }

        // iptables rule format inside user.rules:
        // -A ufw-user-input -p udp --dport 42889 -j ACCEPT
        if low.starts_with("-a ufw-user-input") && low.contains("-p udp") {
            let dport_flag = format!("--dport {port_str}");
            if low.split_whitespace().collect::<Vec<_>>().windows(2).any(|w| w[0] == "--dport" && w[1] == port_str)
                || low.contains(&dport_flag)
            {
                if low.contains("-j accept") {
                    allowed = true;
                } else if low.contains("-j drop") || low.contains("-j reject") {
                    blocked = true;
                }
            }
            continue;
        }

        // `ufw status` table lines:
        // 42889/udp                   ALLOW       Anywhere
        // 42889                       ALLOW IN    Anywhere
        let tokens: Vec<&str> = low.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }
        let target_matches = tokens[0] == port_udp || tokens[0] == port_str;
        if target_matches {
            if tokens.iter().any(|t| *t == "allow") {
                allowed = true;
            } else if tokens.iter().any(|t| *t == "deny" || *t == "reject") {
                blocked = true;
            }
        }
    }

    (allowed, blocked)
}

pub fn parse_firewalld_ports_for_udp_port(ports_output: &str, udp_port: u16) -> bool {
    for token in ports_output.split_whitespace() {
        let low = token.trim().to_ascii_lowercase();
        let Some(port_part) = low.strip_suffix("/udp") else {
            continue;
        };
        if let Ok(single) = port_part.parse::<u16>() {
            if single == udp_port {
                return true;
            }
        } else if let Some((start_s, end_s)) = port_part.split_once('-') {
            if let (Ok(start), Ok(end)) = (start_s.parse::<u16>(), end_s.parse::<u16>()) {
                if udp_port >= start && udp_port <= end {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ufw_status_and_user_rules() {
        let status_allow = "\
Status: active

To                         Action      From
--                         ------      ----
22/tcp                     ALLOW       Anywhere
42889/udp                  ALLOW       Anywhere
42889/udp (v6)             ALLOW       Anywhere (v6)
";
        assert_eq!(parse_ufw_rules_for_udp_port(status_allow, 42889), (true, false));
        assert_eq!(parse_ufw_rules_for_udp_port(status_allow, 42888), (false, false));

        let status_deny = "42889/udp                  DENY        Anywhere\n";
        assert_eq!(parse_ufw_rules_for_udp_port(status_deny, 42889), (false, true));

        let user_rules = "\
### tuple ### allow udp 42889 0.0.0.0/0 any 0.0.0.0/0 in
-A ufw-user-input -p udp --dport 42889 -j ACCEPT
";
        assert_eq!(parse_ufw_rules_for_udp_port(user_rules, 42889), (true, false));
    }

    #[test]
    fn parses_firewalld_ports_single_and_range() {
        assert!(parse_firewalld_ports_for_udp_port("8080/tcp 42889/udp", 42889));
        assert!(parse_firewalld_ports_for_udp_port("42880-42895/udp", 42889));
        assert!(!parse_firewalld_ports_for_udp_port("42889/tcp 5353/udp", 42889));
    }

    #[test]
    fn evaluates_linux_firewall_verdicts_accurately() {
        let cmd = "sudo ufw allow 42889/udp comment \"Pix Relay Discovery\"".to_string();

        let inactive = LinuxFirewallInspection {
            backend: "none".to_string(),
            active: false,
            allowed: false,
            blocked: false,
            query_succeeded: true,
            recommended_command: cmd.clone(),
        };
        let (status, _, healthy) = evaluate_linux_firewall_verdict(true, &inactive, 42889);
        assert_eq!(status, "INACTIVE");
        assert!(healthy);

        let ufw_allowed = LinuxFirewallInspection {
            backend: "ufw".to_string(),
            active: true,
            allowed: true,
            blocked: false,
            query_succeeded: true,
            recommended_command: cmd.clone(),
        };
        let (status, _, healthy) = evaluate_linux_firewall_verdict(true, &ufw_allowed, 42889);
        assert_eq!(status, "ALLOWED");
        assert!(healthy);

        let ufw_missing = LinuxFirewallInspection {
            backend: "ufw".to_string(),
            active: true,
            allowed: false,
            blocked: false,
            query_succeeded: true,
            recommended_command: cmd.clone(),
        };
        let (status, _, healthy) = evaluate_linux_firewall_verdict(true, &ufw_missing, 42889);
        assert_eq!(status, "UNCONFIGURED");
        assert!(!healthy);

        let ufw_blocked = LinuxFirewallInspection {
            backend: "ufw".to_string(),
            active: true,
            allowed: false,
            blocked: true,
            query_succeeded: true,
            recommended_command: cmd.clone(),
        };
        let (status, _, healthy) = evaluate_linux_firewall_verdict(true, &ufw_blocked, 42889);
        assert_eq!(status, "BLOCKED");
        assert!(!healthy);

        let ufw_unknown = LinuxFirewallInspection {
            backend: "ufw".to_string(),
            active: true,
            allowed: false,
            blocked: false,
            query_succeeded: false,
            recommended_command: cmd,
        };
        let (status, _, healthy) = evaluate_linux_firewall_verdict(true, &ufw_unknown, 42889);
        assert_eq!(status, "UNKNOWN");
        assert!(healthy);

        let (status, _, healthy) = evaluate_linux_firewall_verdict(false, &inactive, 42889);
        assert_eq!(status, "UNCONFIGURED");
        assert!(!healthy);
    }
}
