# AI Context — Pvault

Read this file first. It tells you everything you need to know about this project.

## What this is

Pvault — a local-first password manager. Desktop app + browser extension + CLI tool.
No cloud, no accounts, no subscriptions. Everything stays on the user's machine.

## Tech stack

- **Desktop app**: Tauri v2 (Rust backend + React 19 frontend)
- **Frontend bundler**: Vite 6
- **Database**: SQLite via rusqlite (bundled), WAL mode
- **Encryption**: AES-256-GCM with PBKDF2-HMAC-SHA512 key derivation (100k iterations)
- **Password hashing**: bcrypt (12 rounds)
- **PGP**: pgp crate (Ed25519 + RSA-4096)
- **TOTP**: totp-rs crate with QR code generation (qrcode crate)
- **Browser extension**: Chrome Manifest V3, communicates via Native Messaging
- **Extension bridge**: Axum HTTP server on 127.0.0.1 with bearer token auth

## File map

### Rust backend (`src-tauri/src/`)

| File | What it does |
|------|-------------|
| `lib.rs` | App entry point. Sets up database, state, starts HTTP server, auto-lock timer, native host manifest install |
| `main.rs` | Just calls `password_manager::run()` |
| `commands.rs` | All Tauri IPC commands (frontend calls these via `invoke()`) |
| `database.rs` | SQLite schema, CRUD for entries, PGP keys, TOTP params, import/export |
| `crypto.rs` | AES-256-GCM encrypt/decrypt, PBKDF2 key derivation, password generator |
| `state.rs` | In-memory app state: master password (zeroized on lock), settings, auto-lock timer, rate limiting |
| `server.rs` | Axum HTTP server for extension bridge. Bearer token auth + origin validation |
| `native_host.rs` | Auto-installs native messaging manifests for all browsers on startup |
| `pgp.rs` | PGP key generation, encrypt, decrypt, sign, verify |
| `totp.rs` | TOTP secret generation, code generation, QR SVG, otpauth:// URI parsing |
| `clipboard.rs` | Copy to clipboard with auto-clear after timeout |
| `bin/cli.rs` | Interactive CLI tool (pvault). Self-contained, re-implements crypto inline |
| `bin/native-host.rs` | Native messaging host binary. Chrome launches this, it forwards to the HTTP server |

### React frontend (`src/`)

| File | What it does |
|------|-------------|
| `App.jsx` | Main app shell: toolbar, sidebar, routing, modals, help overlay |
| `api.js` | Wrapper around Tauri `invoke()` — every backend call goes through here |
| `ThemeContext.jsx` | Theme (dark/light) + appearance settings (accent color, bg color, opacity) |
| `main.jsx` | React entry point, wraps app in ThemeProvider |
| `styles.css` | All CSS. Uses CSS custom properties for theming. ~1200 lines |

### React components (`src/components/`)

| Component | What it does |
|-----------|-------------|
| `Login.jsx` | Master password setup (first time) or unlock screen |
| `Vault.jsx` | Entry table with columns: icon, site, username, URL, modified |
| `Sidebar.jsx` | Navigation: Database, All Entries, Favourites, PGP Keys, Settings |
| `DetailPanel.jsx` | Right panel showing selected entry details, password reveal, copy, 2FA |
| `AddEntry.jsx` | Modal form to add new entry with password generator |
| `EditEntry.jsx` | Modal form to edit existing entry |
| `Settings.jsx` | Security settings, appearance, export/import, browser extension ID |
| `PGPKeys.jsx` | PGP key list with generate/delete/export actions |
| `PGPActions.jsx` | Encrypt/decrypt/sign/verify modal |
| `GeneratePGPKey.jsx` | PGP key generation form (name, email, type, passphrase) |
| `SetupTotp.jsx` | TOTP setup modal: manual secret or paste URI, QR display, verify |
| `TotpDisplay.jsx` | Live TOTP code display with countdown timer |
| `PasswordStrength.jsx` | Password strength bar (weak/fair/good/strong/excellent) |

### Browser extension (`extension/`)

| File | What it does |
|------|-------------|
| `manifest.json` | Manifest V3. Has a pinned `key` for stable extension ID |
| `background/service-worker.js` | Connects to native host via `chrome.runtime.connectNative()`, routes messages |
| `popup/popup.js` | Popup UI logic: checks vault status, searches entries by current tab hostname, autofill |
| `popup/popup.html` | Popup markup with states: loading, disconnected, locked, empty, credentials |
| `popup/popup.css` | Popup styles |
| `content/content-script.js` | Injected into pages. Finds login fields, fills credentials via native input setter |

### Config files

| File | What it does |
|------|-------------|
| `package.json` | npm scripts: dev, build, tauri:dev, tauri:build, cli:build, native-host:build |
| `src-tauri/Cargo.toml` | Rust dependencies. Three binaries: password-manager, pvault, password-manager-native-host |
| `src-tauri/tauri.conf.json` | Tauri config: window size, bundle targets, icons |
| `vite.config.js` | Vite config: React plugin, port 5173 |
| `.github/workflows/release.yml` | GitHub Actions: builds for macOS (ARM+Intel), Linux, Windows on tag push |

## How the pieces connect

### Desktop app startup flow
1. `main.rs` → `lib.rs::run()`
2. Opens SQLite database at `~/.config/password-manager/vault.db`
3. Creates `AppState` (holds master password in memory)
4. Starts Axum HTTP server on random port on 127.0.0.1
5. Writes port + auth token to `~/.config/password-manager/.server-port`
6. Installs native messaging host manifests for all detected browsers
7. Starts auto-lock background timer (checks every 30s)
8. Opens Tauri window loading the React frontend

### Frontend → Backend communication
- React calls `api.someMethod()` → calls `invoke('rust_command_name', { args })`
- Tauri routes to the matching `#[tauri::command]` function in `commands.rs`
- Commands access `Database` and `AppState` via Tauri's managed state

### Extension → Desktop app communication
```
Extension popup → service-worker.js (sendNativeMessage)
  → chrome.runtime.connectNative('com.passwordmanager.app')
  → password-manager-native-host binary (stdin/stdout JSON)
  → reads .server-port file for port + token
  → HTTP POST to 127.0.0.1:{port}/api with Bearer token
  → server.rs handles the request
  → returns JSON response back through the chain
```

### Encryption flow
- User enters master password → bcrypt hash stored in DB
- On unlock: raw password held in `AppState` (zeroized on lock)
- To encrypt: PBKDF2 derives 256-bit key from password + random salt → AES-256-GCM
- Stored format: `salt:iv:authTag:ciphertext` (all hex-encoded)
- PGP private keys: double-encrypted (AES + passphrase)

## Database schema

```sql
-- Master password hash
CREATE TABLE master (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    hash TEXT NOT NULL
);

-- Password entries
CREATE TABLE entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    site_name TEXT NOT NULL,
    url TEXT,
    username TEXT NOT NULL,
    password_encrypted TEXT NOT NULL,
    totp_secret_encrypted TEXT,
    totp_algorithm TEXT,
    totp_digits INTEGER,
    totp_period INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- PGP keys
CREATE TABLE pgp_keys (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    email TEXT NOT NULL,
    key_type TEXT NOT NULL,
    public_key_armored TEXT NOT NULL,
    private_key_encrypted TEXT NOT NULL,
    fingerprint TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

## Important patterns

- **Favourites** are stored in `localStorage` on the frontend, not in the database
- **Appearance settings** (accent color, bg color, opacity) are in `localStorage`, applied via CSS custom properties
- **Security settings** (auto-lock, clipboard clear, password length) are in `AppState` in Rust memory
- **The CLI (`pvault`)** is fully self-contained — it re-implements crypto and DB access inline, doesn't depend on the Tauri app
- **Native host manifests** are re-written on every app launch (idempotent)
- **Extension ID** is pinned via the `key` field in manifest.json → ID: `ddjeepljblipgikmpdpiddbckeokpamd`

## Common tasks for AI

### Adding a new setting
1. Add field to `Settings` struct in `state.rs`
2. Add to `get_settings` / `update_settings` in `state.rs`
3. Add Tauri command in `commands.rs` if needed (or reuse existing settings flow)
4. Add to `api.js`
5. Add UI in `Settings.jsx`

### Adding a new Tauri command
1. Write the function in `commands.rs` with `#[tauri::command]`
2. Register it in `lib.rs` in the `invoke_handler` list
3. Add the JS wrapper in `api.js`
4. Call it from a component

### Adding a new database table/column
1. Add migration in `Database::init()` in `database.rs` (use `ALTER TABLE` for existing DBs)
2. Add CRUD methods on `Database`
3. Expose via commands

## Build commands

```bash
npm run tauri:dev          # Dev mode with hot reload
npm run tauri:build        # Production build (.dmg/.exe/.AppImage)
npm run cli:build          # Build CLI tool (pvault)
npm run native-host:build  # Build native messaging host
```

## Release process

See `.dev/RELEASE-GUIDE.md` for the full step-by-step.
Short version: bump versions, `git tag v1.x.x`, `git push --tags`.
