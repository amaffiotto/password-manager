# Password Manager

A cross-platform password manager with a Chrome/Vivaldi browser extension, built with Electron + React and the help of AI.

![Status](https://img.shields.io/badge/Status-Work%20In%20Progress-yellow)
![Node](https://img.shields.io/badge/Node.js-18+-green)
![Electron](https://img.shields.io/badge/GUI-Electron-47848F)
![Encryption](https://img.shields.io/badge/Encryption-AES--256--GCM-red)

## What is it?

A local-first password manager that keeps your credentials safe on your own machine — no cloud, no subscriptions. It consists of two parts:

- **Desktop App** — Electron-based GUI that runs on Linux, Mac, and Windows
- **Browser Extension** — Chrome/Vivaldi extension for autofilling login forms

All passwords are encrypted with **AES-256-GCM** and protected by a master password. Nothing ever leaves your computer.

## How I built it

| Component | Technology |
|-----------|------------|
| **Desktop App** | Electron |
| **UI** | React |
| **Backend** | Node.js |
| **Database** | SQLite (local) |
| **Encryption** | AES-256-GCM + PBKDF2 |
| **Password Hashing** | bcrypt |
| **Browser Extension** | Manifest V3 |
| **AI Assistance** | Claude Code |

## Security

| What | How |
|------|-----|
| Master password | Never stored in plaintext — only its bcrypt hash |
| Saved passwords | Encrypted with AES-256-GCM, decrypted only in RAM |
| Encryption key | Derived from master password via PBKDF2 (100,000 iterations) |
| Clipboard | Auto-clears after 30 seconds |
| Database | Local file on your PC, no cloud sync |
| Auto-lock | App locks after 5 minutes of inactivity |

## Project Structure

```
password-manager/
├── desktop/                    # Electron desktop app
│   └── src/
│       ├── main/               # Main process
│       │   ├── index.js        # App entry point
│       │   ├── database.js     # SQLite operations
│       │   ├── crypto.js       # AES-256 encryption
│       │   ├── ipc-handlers.js # UI ↔ backend communication
│       │   ├── local-server.js # Local HTTP server for extension bridge
│       │   └── native-messaging.js # Native messaging host for Chrome
│       └── renderer/           # React UI
│           ├── components/
│           │   ├── Login.jsx
│           │   ├── Vault.jsx
│           │   ├── AddEntry.jsx
│           │   └── Settings.jsx
│           ├── App.jsx
│           └── index.html
├── extension/                  # Chrome/Vivaldi extension
│   ├── manifest.json
│   ├── popup/
│   ├── content/
│   ├── background/
│   └── icons/
└── shared/                     # Shared utilities
    ├── constants.js
    └── validators.js
```

## How to try it

### Prerequisites

- **Node.js 18+** — [Download](https://nodejs.org/)
- **Git** — [Download](https://git-scm.com/)

### macOS / Linux

```bash
# Clone the repo
git clone https://github.com/amaffiotto/password-manager.git
cd password-manager

# Install dependencies
cd desktop && npm install && cd ..

# Run the app
cd desktop && npm start
```

### Windows

```powershell
# Clone the repo
git clone https://github.com/amaffiotto/password-manager.git
cd password-manager

# Install dependencies
cd desktop
npm install

# Run the app
npm start
```

### Browser Extension Setup

1. Open Chrome/Vivaldi and go to `chrome://extensions/`
2. Enable **Developer mode** (toggle in the top right)
3. Click **Load unpacked** and select the `extension/` folder
4. Copy the extension ID shown on the card
5. Register the native messaging host:

```bash
node desktop/scripts/install-native-host.js --extension-id=YOUR_EXTENSION_ID
```

6. Make sure the desktop app is running and the vault is unlocked
7. Click the extension icon on any login page to autofill

## Roadmap

### Core

- [x] AES-256-GCM encryption module
- [x] SQLite database with encrypted entries
- [x] Master password hashing with bcrypt
- [x] Secure random password generator
- [x] Electron app window and IPC setup
- [x] React UI (Login, Vault, Add Entry, Settings)

### Browser Extension

- [x] Manifest V3 extension scaffold
- [x] Popup with credentials for current site
- [x] Content script for autofill
- [x] Native messaging bridge to desktop app

### Polish

- [x] Auto-lock on inactivity
- [x] Clipboard auto-clear
- [ ] Cross-platform build (.exe, .dmg, .AppImage)
- [ ] Password strength indicator

## Why?

I believe AI is a powerful tool that enables people to build real software, even without years of coding experience. This project is part of my journey building useful applications with the help of AI.

---

*Created by Andrea Maffiotto — built with Claude Code and Cursor.*
