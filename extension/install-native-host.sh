#!/bin/bash
# Install the native messaging host manifest for Chrome/Chromium/Brave/Edge browsers.
# This allows the Chrome extension to communicate with the desktop app.

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
HOST_NAME="com.passwordmanager.app"

# Build the native host binary if it doesn't exist
NATIVE_HOST_BIN="$PROJECT_DIR/src-tauri/target/release/password-manager-native-host"
if [ ! -f "$NATIVE_HOST_BIN" ]; then
  echo "Building native messaging host..."
  cd "$PROJECT_DIR/src-tauri"
  cargo build --release --bin password-manager-native-host
fi

# Determine browser native messaging hosts directories
if [ "$(uname)" = "Darwin" ]; then
  CHROME_DIR="$HOME/Library/Application Support/Google/Chrome/NativeMessagingHosts"
  CHROMIUM_DIR="$HOME/Library/Application Support/Chromium/NativeMessagingHosts"
  BRAVE_DIR="$HOME/Library/Application Support/BraveSoftware/Brave-Browser/NativeMessagingHosts"
  EDGE_DIR="$HOME/Library/Application Support/Microsoft Edge/NativeMessagingHosts"
elif [ "$(uname)" = "Linux" ]; then
  CHROME_DIR="$HOME/.config/google-chrome/NativeMessagingHosts"
  CHROMIUM_DIR="$HOME/.config/chromium/NativeMessagingHosts"
  BRAVE_DIR="$HOME/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts"
  EDGE_DIR="$HOME/.config/microsoft-edge/NativeMessagingHosts"
else
  echo "Unsupported OS. Please install manually."
  exit 1
fi

# Create the native messaging host manifest
MANIFEST='{
  "name": "'"$HOST_NAME"'",
  "description": "Password Manager native messaging host",
  "path": "'"$NATIVE_HOST_BIN"'",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://EXTENSION_ID_PLACEHOLDER/"
  ]
}'

install_manifest() {
  local dir="$1"
  local browser="$2"
  mkdir -p "$dir"
  echo "$MANIFEST" > "$dir/$HOST_NAME.json"
  echo "  Installed manifest for $browser"
}

echo "Installing native messaging host manifests..."
echo ""

# Install for all detected browsers
for BROWSER_DIR in "$CHROME_DIR" "$CHROMIUM_DIR" "$BRAVE_DIR" "$EDGE_DIR"; do
  BROWSER_PARENT="$(dirname "$BROWSER_DIR")"
  BROWSER_NAME="$(basename "$BROWSER_PARENT")"
  if [ -d "$BROWSER_PARENT" ]; then
    install_manifest "$BROWSER_DIR" "$BROWSER_NAME"
  fi
done

echo ""
echo "=== IMPORTANT ==="
echo "You need to update the extension ID in the manifest file(s)."
echo ""
echo "Steps:"
echo "  1. Open your browser and go to chrome://extensions (or brave://extensions)"
echo "  2. Enable 'Developer mode' (toggle in top right)"
echo "  3. Click 'Load unpacked' and select: $PROJECT_DIR/extension"
echo "  4. Copy the extension ID shown (e.g., abcdef1234567890...)"
echo "  5. Edit the manifest file and replace EXTENSION_ID_PLACEHOLDER"
echo "     with your actual extension ID."
echo ""
echo "Done! The extension should now connect to the desktop app."
