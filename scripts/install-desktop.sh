#!/usr/bin/env bash
set -euo pipefail

BINARY_PATH="$HOME/.local/bin/mmterm"
ICON_DIR="$HOME/.local/share/icons/hicolor/256x256/apps"
DESKTOP_DIR="$HOME/.local/share/applications"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "Building mmterm..."
cargo build --release --manifest-path "$PROJECT_DIR/Cargo.toml"

echo "Installing binary to $BINARY_PATH..."
mkdir -p "$(dirname "$BINARY_PATH")"
cp "$PROJECT_DIR/target/release/mmterm" "$BINARY_PATH"
chmod +x "$BINARY_PATH"

echo "Installing icon..."
mkdir -p "$ICON_DIR"
cp "$PROJECT_DIR/assets/icon.png" "$ICON_DIR/mmterm.png"

echo "Installing .desktop entry..."
mkdir -p "$DESKTOP_DIR"
cat > "$DESKTOP_DIR/mmterm.desktop" <<EOF
[Desktop Entry]
Name=mmterm
Comment=Cross-platform CPU-rendered terminal emulator
Exec=$BINARY_PATH
Icon=mmterm
Type=Application
Categories=System;TerminalEmulator;
StartupNotify=false
EOF

update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
update-icon-caches "$HOME/.local/share/icons/hicolor" 2>/dev/null || true

echo "Done. mmterm is ready to launch from your application menu."
