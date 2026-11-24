# Building Pix Relay: Auto Linux Sync

**Pix Relay: Auto Linux Sync** is a native Linux Tauri v2 + Rust + React desktop companion for pairing with a Pixel 1–4 running the Pix Relay Android Receiver app over encrypted LAN Wi-Fi.

---

## 1. System Prerequisites

### Ubuntu / Debian
```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  patchelf \
  rpm
```

### Fedora / RHEL
```bash
sudo dnf install -y \
  webkit2gtk4.1-devel \
  openssl-devel \
  curl \
  wget \
  file \
  libappindicator-gtk3-devel \
  librsvg2-devel \
  libxdo-devel \
  rpm-build \
  @development-tools
```

### Arch Linux
```bash
sudo pacman -S --needed \
  webkit2gtk-4.1 \
  base-devel \
  curl \
  wget \
  file \
  openssl \
  libayatana-appindicator \
  librsvg \
  xdotool
```

### Optional: Video Thumbnails
To render video thumbnails in the gallery (instead of the fallback icon), install either `ffmpegthumbnailer` or `ffmpeg`:
- Ubuntu/Debian: `sudo apt-get install -y ffmpegthumbnailer` (or `ffmpeg`)
- Fedora: `sudo dnf install -y ffmpegthumbnailer`
- Arch Linux: `sudo pacman -S ffmpegthumbnailer`

### Toolchain
- **Node.js** 20+ (`node` and `npm`)
- **Rust** stable toolchain (`rustup` / `cargo` / `rustc`)

---

## 2. Development & Packaging Commands

### Run in Development Mode
```bash
npm install
npm run tauri dev
```

### Build `.deb`, `.rpm`, and `.AppImage` Bundles
Make the build helper executable (first time only) and run it:
```bash
chmod +x scripts/build-linux.sh
./scripts/build-linux.sh
```
Or run manually:
```bash
npm install
(cd src-tauri && cargo test)
npm run tauri build
```

Generated installers will be placed in:
- `src-tauri/target/release/bundle/deb/`
- `src-tauri/target/release/bundle/rpm/`
- `src-tauri/target/release/bundle/appimage/`

> Note: You can also trigger `.github/workflows/build-linux.yml` on GitHub Actions to produce `.deb`, `.rpm`, and `.AppImage` packages in Ubuntu CI.

---

## 3. Data Paths & Autostart

- **SQLite database**: `~/.local/share/pix-relay-auto-linux-sync/pix_relay_linux.db`
- **Thumbnail cache** (capped at 250 MB): `~/.cache/pix-relay-auto-linux-sync/thumb_cache/`
- **Start at Login**: Managed via XDG Autostart (`~/.config/autostart/*.desktop`) when enabled in **Settings**.
- **Trash Cleanup**: Uses FreeDesktop Trash (`~/.local/share/Trash`) via the `trash` crate.

---

## 4. Firewall Configuration (Auto-Discovery)

Pix Relay discovers nearby Pixels using **mDNS (`_pixrelay._tcp.local.`)** and **UDP port `42889`**, and transfers encrypted media over **TCP port `42888`**.

If you run an active host firewall and nearby Pixels do not appear automatically:

- **UFW (Ubuntu/Debian)**:
  ```bash
  sudo ufw allow 42889/udp comment "Pix Relay Discovery"
  ```
- **firewalld (Fedora/RHEL)**:
  ```bash
  sudo firewall-cmd --add-port=42889/udp --permanent
  sudo firewall-cmd --reload
  ```

You can always pair directly using **Connect by IP** or a **`PIX1:` pairing code** from the Pixel Receiver screen even if UDP broadcasts are blocked.
