import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Activity, Monitor, Moon, Sun, Terminal } from "lucide-react";
import { ActivityEntry, PcSettings, ThemeMode } from "../types";

interface Props {
  settings: PcSettings;
  onUpdateSettings: (next: PcSettings) => Promise<void>;
}

export function SettingsView({
  settings,
  onUpdateSettings,
}: Props) {
  const [nameDraft, setNameDraft] = useState(settings.deviceName);
  const [activity, setActivity] = useState<ActivityEntry[]>([]);

  useEffect(() => {
    setNameDraft(settings.deviceName);
  }, [settings.deviceName]);

  useEffect(() => {
    invoke<ActivityEntry[]>("list_activity_log").then(setActivity).catch(() => {});
  }, [settings]);

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 16, maxWidth: 880 }}>
      <div>
        <div className="eyebrow" style={{ marginBottom: 4 }}>Preferences &amp; System</div>
        <h1 style={{ margin: 0, fontSize: 24, fontWeight: 800 }}>Settings</h1>
      </div>

      {/* Linux Identity & Theme */}
      <div className="m3-card" style={{ display: "flex", flexDirection: "column", gap: 14 }}>
        <div style={{ fontWeight: 700, fontSize: 15 }}>Linux Identity &amp; Theme</div>

        <div style={{ display: "flex", gap: 10, alignItems: "center" }}>
          <div style={{ flex: 1 }}>
            <div style={{ fontSize: 12, color: "var(--text-muted)", marginBottom: 5 }}>Device Name (shown on your Pixel)</div>
            <input
              className="input-field"
              value={nameDraft}
              onChange={(e) => setNameDraft(e.target.value)}
            />
          </div>
          <button
            className="btn btn-tonal"
            style={{ alignSelf: "flex-end" }}
            onClick={() => onUpdateSettings({ ...settings, deviceName: nameDraft })}
          >
            Save Name
          </button>
        </div>

        <div>
          <div style={{ fontSize: 12, color: "var(--text-muted)", marginBottom: 8 }}>App Theme</div>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: 8 }}>
            {(
              [
                ["SYSTEM", "Auto", Monitor],
                ["DARK", "Darkroom", Moon],
                ["LIGHT", "Gallery Paper", Sun],
                ["MATRIX", "Matrix", Terminal],
              ] as [ThemeMode, string, typeof Sun][]
            ).map(([mode, label, Icon]) => (
              <button
                key={mode}
                className={`btn ${settings.themeMode === mode ? "btn-primary" : "btn-outline"}`}
                onClick={() => onUpdateSettings({ ...settings, themeMode: mode })}
              >
                <Icon size={14} /> {label}
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Sync & Linux Behavior */}
      <div className="m3-card" style={{ display: "flex", flexDirection: "column", gap: 12 }}>
        <div style={{ fontWeight: 700, fontSize: 15 }}>Sync &amp; Lightweight Background Mode</div>

        <ToggleRow
          title="Auto-Sync Watched Folders"
          subtitle="Automatically relay photos in Auto folders whenever your Pixel is reachable"
          checked={settings.autoSyncEnabled}
          onChange={(v) => onUpdateSettings({ ...settings, autoSyncEnabled: v })}
        />

        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            gap: 12,
            flexWrap: "wrap",
            padding: "8px 0",
            borderBottom: "1px solid var(--border-subtle)",
          }}
        >
          <div style={{ flex: "1 1 240px" }}>
            <div style={{ fontWeight: 600, fontSize: 13 }}>
              Auto-Send Photos:{" "}
              <span style={{ color: "var(--primary)" }}>
                {settings.autoSendCutoffEpochMs <= 0
                  ? "All Photos"
                  : `New from ${new Date(settings.autoSendCutoffEpochMs).toLocaleDateString()}`}
              </span>
            </div>
            <div style={{ fontSize: 11, color: "var(--text-muted)", marginTop: 2 }}>
              {settings.autoSendCutoffEpochMs <= 0
                ? "Uploads all existing photos in Auto folders, then any new ones."
                : "Only uploads new photos from this point on (skips older or restored photos)."}
            </div>
          </div>

          <div style={{ display: "flex", alignItems: "center", gap: 8, flexWrap: "wrap" }}>
            <div className="pill-bar">
              <button
                className={`pill-btn ${settings.autoSendCutoffEpochMs <= 0 ? "active" : ""}`}
                onClick={() => onUpdateSettings({ ...settings, autoSendCutoffEpochMs: 0 })}
              >
                All Photos
              </button>
              <button
                className={`pill-btn ${settings.autoSendCutoffEpochMs > 0 ? "active" : ""}`}
                onClick={() => onUpdateSettings({ ...settings, autoSendCutoffEpochMs: Date.now() })}
              >
                New Photos Only
              </button>
            </div>
            <input
              type="date"
              className="m3-input"
              aria-label="Custom auto-send start date"
              style={{ width: "auto", padding: "4px 8px", fontSize: 12 }}
              value={
                settings.autoSendCutoffEpochMs > 0
                  ? new Date(settings.autoSendCutoffEpochMs).toISOString().slice(0, 10)
                  : ""
              }
              onChange={(e) => {
                if (!e.target.value) {
                  onUpdateSettings({ ...settings, autoSendCutoffEpochMs: 0 });
                } else {
                  const [y, m, d] = e.target.value.split("-").map(Number);
                  const localMidnight = new Date(y, m - 1, d, 0, 0, 0, 0).getTime();
                  if (!Number.isNaN(localMidnight)) {
                    onUpdateSettings({ ...settings, autoSendCutoffEpochMs: localMidnight });
                  }
                }
              }}
            />
          </div>
        </div>

        <ToggleRow
          title="Send Photos Before Videos"
          subtitle="Uploads smaller still photos first before large video files"
          checked={settings.photosBeforeVideos}
          onChange={(v) => onUpdateSettings({ ...settings, photosBeforeVideos: v })}
        />

        <ToggleRow
          title="Include Camera RAW Files (.DNG, .CR3, .ARW, .NEF)"
          subtitle="Opt-in: RAW files are very large (25–80 MB each). Leave off if your Pixel storage is limited"
          checked={settings.includeRawFiles}
          onChange={(v) => onUpdateSettings({ ...settings, includeRawFiles: v })}
        />

        <ToggleRow
          title="Minimize to System Tray on Close (Ultra-Low RAM)"
          subtitle="Closing the window frees WebView memory and keeps only the ~12 MB Rust background engine alive"
          checked={settings.minimizeToTray}
          onChange={(v) => onUpdateSettings({ ...settings, minimizeToTray: v })}
        />

        <ToggleRow
          title="Start Quietly at Login"
          subtitle="Launches minimized to the system tray when you log in (XDG Autostart)"
          checked={settings.startAtLogin}
          onChange={(v) => onUpdateSettings({ ...settings, startAtLogin: v })}
        />
      </div>

      {/* Activity Log */}
      <div className="m3-card" style={{ display: "flex", flexDirection: "column", gap: 8 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 8, fontWeight: 700, fontSize: 14 }}>
          <Activity size={15} color="var(--primary)" /> Recent Activity
        </div>
        {activity.length === 0 ? (
          <div style={{ fontSize: 12, color: "var(--text-muted)" }}>No sync activity recorded yet.</div>
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: 6, maxHeight: 200, overflowY: "auto" }}>
            {activity.map((item) => (
              <div
                key={item.id}
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  fontSize: 12,
                  padding: "6px 8px",
                  borderRadius: 8,
                  background: "var(--surface-low)",
                }}
              >
                <span>
                  <span className="badge badge-muted" style={{ marginRight: 8 }}>{item.kind}</span>
                  {item.message}
                </span>
                <span className="tabular" style={{ color: "var(--text-muted)", fontSize: 11 }}>
                  {new Date(item.timestampMs).toLocaleTimeString()}
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function ToggleRow({
  title,
  subtitle,
  checked,
  onChange,
}: {
  title: string;
  subtitle: string;
  checked: boolean;
  onChange: (val: boolean) => void;
}) {
  return (
    <label
      style={{
        display: "flex",
        justifyContent: "space-between",
        alignItems: "center",
        gap: 16,
        padding: "10px 12px",
        borderRadius: 12,
        background: "var(--surface-low)",
        cursor: "pointer",
      }}
    >
      <div>
        <div style={{ fontWeight: 600, fontSize: 13 }}>{title}</div>
        <div style={{ fontSize: 12, color: "var(--text-muted)" }}>{subtitle}</div>
      </div>
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
        style={{ width: 18, height: 18, accentColor: "var(--primary)" }}
      />
    </label>
  );
}
