#!/bin/bash
# Build the native messaging host binary alongside the main app.
# After `tauri build`, copy it next to the app binary so it's discoverable at runtime.

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
TAURI_DIR="$PROJECT_DIR/src-tauri"

TARGET="${1:-$(rustc -vV | grep 'host:' | cut -d' ' -f2)}"

echo "Building native host for target: $TARGET"
cd "$TAURI_DIR"
cargo build --release --bin password-manager-native-host

echo ""
echo "Native host binary built: target/release/password-manager-native-host"
echo ""
echo "After running 'npm run tauri:build', copy the binary next to the app:"
echo ""

if [ "$(uname)" = "Darwin" ]; then
  BUNDLE="$TAURI_DIR/target/release/bundle/macos/Password Manager.app/Contents/MacOS"
  echo "  cp target/release/password-manager-native-host \"$BUNDLE/\""
elif [ "$(uname)" = "Linux" ]; then
  echo "  Install alongside the AppImage or .deb package"
else
  echo "  Copy password-manager-native-host.exe to the install directory"
fi
