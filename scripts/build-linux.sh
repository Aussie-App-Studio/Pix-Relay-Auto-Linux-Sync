#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${ROOT_DIR}"

echo "==> Checking build prerequisites..."
for cmd in node npm cargo rustc; do
  if ! command -v "${cmd}" >/dev/null 2>&1; then
    echo "Error: '${cmd}' is not installed or not in PATH." >&2
    exit 1
  fi
done

if command -v pkg-config >/dev/null 2>&1; then
  for pkg in webkit2gtk-4.1 openssl; do
    if ! pkg-config --exists "${pkg}" 2>/dev/null; then
      echo "Warning: pkg-config could not find '${pkg}'. See LINUX_BUILD.md for system package dependencies." >&2
    fi
  done
fi

echo "==> Installing Node dependencies..."
if [ -f package-lock.json ]; then
  npm ci
else
  npm install
fi

echo "==> Running Rust backend unit & loopback tests..."
(cd src-tauri && cargo test)

echo "==> Building frontend & Linux bundles (.deb, .rpm, .AppImage)..."
npm run tauri build

echo "==> Build complete! Bundles generated under:"
echo "    ${ROOT_DIR}/src-tauri/target/release/bundle/"
