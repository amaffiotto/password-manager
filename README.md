# Password Manager

A local-first, cross-platform password manager with PGP support and browser extension autofill.

![Status](https://img.shields.io/badge/Status-Ready-green)
![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Linux%20%7C%20Windows-blue)
![Encryption](https://img.shields.io/badge/Encryption-AES--256--GCM-red)

## Install

Download the latest release for your platform:

| Platform | File |
|----------|------|
| **macOS** | `Password Manager_x.x.x_aarch64.dmg` or `_x64.dmg` |
| **Windows** | `Password Manager_x.x.x_x64-setup.exe` |
| **Linux** | `Password Manager_x.x.x_amd64.AppImage` |

> [Download from Releases](https://github.com/amaffiotto/password-manager/releases)

### Browser Extension (optional)

The extension lets you autofill login forms from your vault. It requires the desktop app to be running.

1. Install and launch the desktop app at least once
2. Install the extension from the Chrome Web Store *(or load `extension/` as unpacked in developer mode)*
3. That's it — the desktop app auto-registers everything the extension needs

Works with **Chrome, Brave, Edge, Chromium, and Vivaldi**.

## What It Does

- **Password Vault** — Store, search, edit, and delete credentials
- **Password Generator** — Cryptographically secure, configurable length
- **TOTP / 2FA** — Set up and generate time-based OTP codes with QR support
- **PGP Keys** — Generate Ed25519 or RSA-4096 key pairs; encrypt, decrypt, sign, verify
- **Browser Autofill** — Fill login forms from the extension popup
- **Import / Export** — JSON and CSV
- **CLI Tool** — Interactive terminal access to your vault (`pvault`)
- **Appearance** — Customizable accent color, background color, UI opacity
- **Auto-Lock** — Configurable inactivity timeout
- **Clipboard Auto-Clear** — Clears copied passwords after a set time

## Security

| | |
|---|---|
| Passwords | Encrypted with AES-256-GCM, decrypted only in RAM |
| Master password | Hashed with bcrypt (12 rounds), never stored in plaintext |
| Encryption key | Derived via PBKDF2-HMAC-SHA512 (100k iterations) |
| PGP private keys | Encrypted at rest + passphrase-protected |
| Extension bridge | Bound to 127.0.0.1 with bearer token auth |
| Database | Local SQLite file with owner-only permissions |
| Memory | Master password zeroized on lock |

## Architecture

| Component | Technology |
|-----------|------------|
| Desktop Framework | Tauri v2 |
| Backend | Rust |
| Frontend | React 19 + Vite |
| Database | SQLite (WAL mode) |
| Browser Extension | Manifest V3 + Native Messaging |

---

## Development

*Everything below is for building from source — not needed for end users.*

### Prerequisites

- [Rust](https://rustup.rs/)
- [Node.js 18+](https://nodejs.org/)

### Run Locally

```bash
git clone https://github.com/amaffiotto/password-manager.git
cd password-manager
npm install
npm run tauri:dev
```

### Build Installers

```bash
# Desktop app (.dmg / .exe / .AppImage)
npm run tauri:build

# CLI tool
npm run cli:build

# Native messaging host (for extension dev)
npm run native-host:build
```

### Extension Development

1. Run `npm run native-host:build`
2. Start the desktop app with `npm run tauri:dev`
3. Go to `chrome://extensions`, enable Developer mode, click **Load unpacked**, select `extension/`
4. If your extension ID differs from the pinned one, set it in **Settings > Browser Extension**

### Project Structure

```
src-tauri/src/          Rust backend (encryption, database, server, TOTP, PGP)
src/                    React frontend (components, API layer, styles)
extension/              Browser extension (Manifest V3, popup, content script)
scripts/                Build helpers
```

---

*Created by Andrea Maffiotto.*
