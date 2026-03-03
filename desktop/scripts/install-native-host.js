#!/usr/bin/env node

/**
 * install-native-host.js — Registers the native messaging host with Chrome/Vivaldi.
 *
 * Usage:
 *   node install-native-host.js [--extension-id=YOUR_EXTENSION_ID]
 *
 * This creates the native messaging host manifest JSON file in the
 * correct system location so Chrome knows how to launch our bridge.
 */

const fs = require('fs');
const path = require('path');
const os = require('os');

const HOST_NAME = 'com.passwordmanager.app';
const NATIVE_MESSAGING_SCRIPT = path.resolve(
  __dirname,
  '../src/main/native-messaging.js'
);

// Parse command-line args
const args = process.argv.slice(2);
let extensionId = null;
for (const arg of args) {
  if (arg.startsWith('--extension-id=')) {
    extensionId = arg.split('=')[1];
  }
}

// The allowed_origins must include the extension's ID.
// If not provided, use a wildcard-like placeholder the user should replace.
const allowedOrigins = extensionId
  ? [`chrome-extension://${extensionId}/`]
  : ['chrome-extension://REPLACE_WITH_YOUR_EXTENSION_ID/'];

// The native messaging host manifest
const manifest = {
  name: HOST_NAME,
  description: 'Password Manager native messaging host',
  path: NATIVE_MESSAGING_SCRIPT,
  type: 'stdio',
  allowed_origins: allowedOrigins,
};

// --- Determine where to write the manifest ---

function getManifestDir() {
  switch (process.platform) {
    case 'darwin':
      // Chrome and Vivaldi on macOS
      return [
        path.join(
          os.homedir(),
          'Library/Application Support/Google/Chrome/NativeMessagingHosts'
        ),
        path.join(
          os.homedir(),
          'Library/Application Support/Vivaldi/NativeMessagingHosts'
        ),
      ];

    case 'linux':
      return [
        path.join(os.homedir(), '.config/google-chrome/NativeMessagingHosts'),
        path.join(os.homedir(), '.config/vivaldi/NativeMessagingHosts'),
      ];

    case 'win32':
      // On Windows we write to the same folder and register via the registry
      return [path.join(os.homedir(), '.password-manager')];

    default:
      console.error(`Unsupported platform: ${process.platform}`);
      process.exit(1);
  }
}

function installManifest() {
  const dirs = getManifestDir();

  // Make the native messaging script executable
  try {
    fs.chmodSync(NATIVE_MESSAGING_SCRIPT, 0o755);
  } catch {
    // May fail on Windows — that's OK
  }

  for (const dir of dirs) {
    fs.mkdirSync(dir, { recursive: true });
    const manifestPath = path.join(dir, `${HOST_NAME}.json`);
    fs.writeFileSync(manifestPath, JSON.stringify(manifest, null, 2), 'utf-8');
    console.log(`Installed: ${manifestPath}`);
  }

  // Windows: register in the registry
  if (process.platform === 'win32') {
    const manifestPath = path.join(dirs[0], `${HOST_NAME}.json`);
    const regKey = `HKCU\\Software\\Google\\Chrome\\NativeMessagingHosts\\${HOST_NAME}`;
    try {
      require('child_process').execSync(
        `reg add "${regKey}" /ve /t REG_SZ /d "${manifestPath}" /f`
      );
      console.log(`Registry key set: ${regKey}`);
    } catch (err) {
      console.error('Failed to set registry key:', err.message);
    }
  }

  if (!extensionId) {
    console.log(
      '\nIMPORTANT: You need to update the allowed_origins in the manifest'
    );
    console.log('with your actual extension ID after loading it in Chrome.');
    console.log(
      'Run again with: node install-native-host.js --extension-id=YOUR_ID'
    );
  }

  console.log('\nDone! Native messaging host registered.');
}

installManifest();
