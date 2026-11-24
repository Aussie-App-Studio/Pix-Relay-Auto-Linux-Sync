# Privacy Policy — Pix Relay: Auto Linux Sync

**Effective Date:** November 24, 2026  
**Publisher:** Aussie App Studio

## The Short Version
**Pix Relay: Auto Linux Sync never connects to the internet and never collects, tracks, or shares any of your data.** Everything stays on your Linux PC and your own Pixel phone over your home Wi-Fi network.

---

## 1. Zero Internet Connections
Pix Relay: Auto Linux Sync does not communicate with any remote servers, analytics services, telemetry endpoints, ad networks, or cloud backends. Your photos, videos, filenames, and settings never leave your local Wi-Fi network through this app.

## 2. Zero Data Collection or Tracking
We do **not** collect, store, transmit, sell, or share:
- Personal information (name, email, or Google account)
- Photos, videos, folders, or photo metadata
- Device identifiers, IP addresses, or usage statistics
- Crash reports or behavioral tracking data

## 3. Local Wi-Fi Transfer Only
To back up your photos, the app discovers your paired Pixel 1–4 receiver directly on your local Wi-Fi network (`UDP 42889` / mDNS `_pixrelay._tcp.local.`) and sends media over a direct encrypted local connection (`TCP 42888` using P-256 ECDH + AES-256-GCM) straight to your Pixel device. Once files arrive on your Pixel, your Pixel's own Google Photos app handles cloud backup according to your Google account settings.

## 4. Local-Only Linux Storage
Your preferences, paired Pixel keys, watched folder list, cached thumbnails, and transfer history are stored strictly locally on your Linux PC inside:
- `~/.local/share/pix-relay-auto-linux-sync/`
- `~/.cache/pix-relay-auto-linux-sync/`

You can remove all app data at any time by deleting the app and those local folders.

## 5. External Links
If you click an external link (such as this Privacy Policy or the project homepage), your default web browser opens that webpage directly. Pix Relay itself never makes background network requests.

## 6. Contact
Questions about this Privacy Policy can be directed to **Aussie App Studio** via the [Pix Relay Auto Linux Sync GitHub repository](https://github.com/Aussie-App-Studio/Pix-Relay-Auto-Linux-Sync).
