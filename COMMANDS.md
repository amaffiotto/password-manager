# Password Manager — Commands Reference

## GUI Commands

### Toolbar
| Action | Description |
|--------|-------------|
| **+** | Add a new password entry |
| **Search bar** | Search entries by name, URL, or username |
| **Sun/Moon toggle** | Switch between light and dark theme |
| **Lock icon** | Lock the vault |
| **Info (i) button** | Open the help & shortcuts overlay |

### Sidebar Navigation
| Item | Description |
|------|-------------|
| **Database** | View all vault entries |
| **All Entries** | Show complete entry list |
| **Favourites** | View starred/favourite entries |
| **PGP Keys** | Manage PGP encryption keys |
| **Settings** | Configure auto-lock, clipboard, password generation, appearance, export/import |

### Entry Actions (Detail Panel)
| Action | Description |
|--------|-------------|
| **Favourite / Unfavourite** | Star or unstar an entry |
| **Enable 2FA** | Set up TOTP (Time-based One-Time Password) for an entry |
| **Disable 2FA** | Remove TOTP from an entry |
| **Edit** | Modify entry fields (site name, URL, username, password) |
| **Delete** | Permanently remove the entry |
| **Eye icon** | Show or hide the decrypted password |
| **Copy icon** | Copy username or password to clipboard |

### 2FA / TOTP Setup
| Step | Description |
|------|-------------|
| **Manual tab** | Enter or generate a Base32 secret, choose algorithm, digits, period |
| **Paste URI tab** | Paste an `otpauth://` URI from your authenticator app |
| **QR Code** | Scan the displayed QR code with your authenticator app |
| **Verify** | Enter the code from your authenticator to confirm setup |

### PGP Key Management
| Action | Description |
|--------|-------------|
| **Generate Key** | Create a new PGP key pair (RSA or ECC) |
| **Encrypt** | Encrypt text with a public key |
| **Decrypt** | Decrypt text with a private key |
| **Sign** | Sign a message with a private key |
| **Verify** | Verify a signed message with a public key |
| **Export Public/Private** | Export key in armored format |

### Settings
| Setting | Description |
|---------|-------------|
| **Auto-lock after (minutes)** | Time before the vault auto-locks (1–60 min) |
| **Clipboard auto-clear (seconds)** | Time before clipboard is cleared (10–300 sec) |
| **Default password length** | Default length for generated passwords (8–128) |
| **Accent color** | Customize the UI accent / highlight color |
| **Background color** | Set a custom background color |
| **UI opacity** | Adjust overall UI transparency (50–100%) |
| **Export Vault** | Export all entries as JSON or CSV to clipboard |
| **Import Vault** | Import entries from JSON or CSV data |

---

## CLI Commands (`pvault`)

Run with: `cargo run --bin pvault` (or the built binary)

| Command | Aliases | Description |
|---------|---------|-------------|
| `list` | `ls` | List all saved entries |
| `search <query>` | `find` | Search entries by name, URL, or username |
| `add` | `new` | Add a new password entry interactively |
| `show <id>` | `get`, `reveal` | Show decrypted password for an entry |
| `delete <id>` | `rm`, `remove` | Delete an entry (with confirmation) |
| `gen [length]` | `generate` | Generate a random password (default: 16 chars) |
| `otp <id>` | — | Show current TOTP code and remaining seconds |
| `otp setup <id>` | — | Generate and store a TOTP secret for an entry |
| `otp uri <id>` | — | Set up TOTP from an `otpauth://` URI |
| `otp remove <id>` | `otp rm`, `otp disable` | Remove TOTP from an entry |
| `help` | `?` | Show help message |
| `quit` | `exit`, `q` | Lock vault and exit |
