use std::io::{self, Write};

mod cli_core {
    use rusqlite::{params, Connection};
    use std::sync::Mutex;

    // Re-implement minimal crypto inline to avoid complex module dependencies
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };
    use hmac::Hmac;
    use pbkdf2::pbkdf2;
    use rand::RngCore;
    use sha2::Sha512;
    use zeroize::Zeroize;

    const PBKDF2_ITERATIONS: u32 = 100_000;
    const KEY_LENGTH: usize = 32;
    const SALT_LENGTH: usize = 16;
    const IV_LENGTH: usize = 12;
    const TAG_LENGTH: usize = 16;
    const PASSWORD_CHARS: &[u8] =
        b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()_+-=";

    fn derive_key(master_password: &str, salt: &[u8]) -> [u8; KEY_LENGTH] {
        let mut key = [0u8; KEY_LENGTH];
        pbkdf2::<Hmac<Sha512>>(
            master_password.as_bytes(),
            salt,
            PBKDF2_ITERATIONS,
            &mut key,
        )
        .expect("PBKDF2 derivation failed");
        key
    }

    pub fn encrypt(plain_text: &str, master_password: &str) -> Result<String, String> {
        let mut salt = [0u8; SALT_LENGTH];
        let mut iv = [0u8; IV_LENGTH];
        rand::thread_rng().fill_bytes(&mut salt);
        rand::thread_rng().fill_bytes(&mut iv);

        let mut key = derive_key(master_password, &salt);
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;
        key.zeroize();

        let nonce = Nonce::from_slice(&iv);
        let ciphertext_with_tag = cipher
            .encrypt(nonce, plain_text.as_bytes())
            .map_err(|e| e.to_string())?;

        let ct_len = ciphertext_with_tag.len() - TAG_LENGTH;
        let ciphertext = &ciphertext_with_tag[..ct_len];
        let auth_tag = &ciphertext_with_tag[ct_len..];

        Ok(format!(
            "{}:{}:{}:{}",
            hex::encode(salt),
            hex::encode(iv),
            hex::encode(auth_tag),
            hex::encode(ciphertext)
        ))
    }

    pub fn decrypt(encrypted_data: &str, master_password: &str) -> Result<String, String> {
        let parts: Vec<&str> = encrypted_data.split(':').collect();
        if parts.len() != 4 {
            return Err("Invalid encrypted data format".to_string());
        }

        let salt = hex::decode(parts[0]).map_err(|e| e.to_string())?;
        let iv = hex::decode(parts[1]).map_err(|e| e.to_string())?;
        let auth_tag = hex::decode(parts[2]).map_err(|e| e.to_string())?;
        let ciphertext = hex::decode(parts[3]).map_err(|e| e.to_string())?;

        let mut key = derive_key(master_password, &salt);
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;
        key.zeroize();

        let nonce = Nonce::from_slice(&iv);
        let mut combined = ciphertext;
        combined.extend_from_slice(&auth_tag);

        let plaintext = cipher
            .decrypt(nonce, combined.as_ref())
            .map_err(|_| "Decryption failed".to_string())?;

        String::from_utf8(plaintext).map_err(|e| e.to_string())
    }

    // --- TOTP support ---
    pub struct TotpParams {
        pub secret: String,
        pub algorithm: String,
        pub digits: u32,
        pub period: u32,
    }

    pub fn get_totp_params(db: &Database, entry_id: i64, master_password: &str) -> Result<TotpParams, String> {
        let conn = db.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT totp_secret_encrypted, totp_algorithm, totp_digits, totp_period FROM entries WHERE id = ?1",
            params![entry_id],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<i32>>(2)?,
                    row.get::<_, Option<i32>>(3)?,
                ))
            },
        ).map_err(|e| e.to_string())?;

        let encrypted = result.0.unwrap_or_default();
        if encrypted.is_empty() {
            return Err("No TOTP configured for this entry".to_string());
        }

        let secret = decrypt(&encrypted, master_password)?;
        Ok(TotpParams {
            secret,
            algorithm: result.1.unwrap_or_else(|| "SHA1".to_string()),
            digits: result.2.unwrap_or(6) as u32,
            period: result.3.unwrap_or(30) as u32,
        })
    }

    pub fn setup_totp_secret(db: &Database, entry_id: i64, secret: &str, master_password: &str) -> Result<(), String> {
        let encrypted = encrypt(secret, master_password)?;
        let conn = db.conn.lock().unwrap();
        conn.execute(
            "UPDATE entries SET totp_secret_encrypted = ?1, totp_algorithm = 'SHA1', totp_digits = 6, totp_period = 30, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
            params![encrypted, entry_id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn remove_totp_secret(db: &Database, entry_id: i64) -> Result<(), String> {
        let conn = db.conn.lock().unwrap();
        conn.execute(
            "UPDATE entries SET totp_secret_encrypted = NULL, totp_algorithm = NULL, totp_digits = NULL, totp_period = NULL, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            params![entry_id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn generate_totp_code(params: &TotpParams) -> Result<(String, u64), String> {
        use totp_rs::{Algorithm, TOTP};

        let algo = match params.algorithm.as_str() {
            "SHA256" => Algorithm::SHA256,
            "SHA512" => Algorithm::SHA512,
            _ => Algorithm::SHA1,
        };

        let secret_bytes = data_encoding::BASE32.decode(params.secret.as_bytes())
            .map_err(|e| format!("Invalid Base32 secret: {}", e))?;

        let totp = TOTP::new(algo, params.digits as usize, 1, params.period as u64, secret_bytes, None, String::new())
            .map_err(|e| e.to_string())?;

        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();

        let code = totp.generate(time);
        let remaining = params.period as u64 - (time % params.period as u64);
        Ok((code, remaining))
    }

    pub fn generate_totp_secret_b32() -> String {
        let mut secret = [0u8; 20];
        rand::thread_rng().fill_bytes(&mut secret);
        data_encoding::BASE32.encode(&secret)
    }

    pub fn parse_otpauth_uri(uri: &str) -> Result<TotpParams, String> {
        use totp_rs::TOTP;
        let totp = TOTP::from_url(uri).map_err(|e| format!("Invalid otpauth URI: {}", e))?;
        let secret = data_encoding::BASE32.encode(&totp.secret);
        let algorithm = match totp.algorithm {
            totp_rs::Algorithm::SHA256 => "SHA256",
            totp_rs::Algorithm::SHA512 => "SHA512",
            _ => "SHA1",
        };
        Ok(TotpParams {
            secret,
            algorithm: algorithm.to_string(),
            digits: totp.digits as u32,
            period: totp.step as u32,
        })
    }

    pub fn generate_password(length: usize) -> String {
        let charset_len = PASSWORD_CHARS.len();
        let threshold = 256 - (256 % charset_len);
        let mut result = String::with_capacity(length);
        let mut rng = rand::thread_rng();

        while result.len() < length {
            let mut byte = [0u8; 1];
            rng.fill_bytes(&mut byte);
            let b = byte[0] as usize;
            if b < threshold {
                result.push(PASSWORD_CHARS[b % charset_len] as char);
            }
        }
        result
    }

    #[derive(Debug)]
    #[allow(dead_code)]
    pub struct Entry {
        pub id: i64,
        pub site_name: String,
        pub url: Option<String>,
        pub username: String,
        pub password_encrypted: String,
        pub created_at: String,
        pub updated_at: String,
    }

    pub struct Database {
        conn: Mutex<Connection>,
    }

    impl Database {
        pub fn open() -> Result<Self, String> {
            let config_dir = dirs::config_dir().ok_or("Cannot find config directory")?;
            let app_data = config_dir.join("password-manager");
            std::fs::create_dir_all(&app_data).map_err(|e| e.to_string())?;
            let db_path = app_data.join("vault.db");

            let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
            conn.execute_batch("PRAGMA journal_mode = WAL;")
                .map_err(|e| e.to_string())?;

            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS master (
                    id INTEGER PRIMARY KEY CHECK (id = 1),
                    hash TEXT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS entries (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    site_name TEXT NOT NULL,
                    url TEXT,
                    username TEXT NOT NULL,
                    password_encrypted TEXT NOT NULL,
                    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
                );",
            )
            .map_err(|e| e.to_string())?;

            Ok(Self {
                conn: Mutex::new(conn),
            })
        }

        pub fn has_master_password(&self) -> bool {
            let conn = self.conn.lock().unwrap();
            conn.query_row("SELECT COUNT(*) FROM master WHERE id = 1", [], |row| {
                row.get::<_, i64>(0)
            })
            .unwrap_or(0)
                > 0
        }

        pub fn set_master_password(&self, master_password: &str) -> Result<(), String> {
            let hash = bcrypt::hash(master_password, 12).map_err(|e| e.to_string())?;
            let conn = self.conn.lock().unwrap();
            conn.execute(
                "INSERT OR REPLACE INTO master (id, hash) VALUES (1, ?1)",
                params![hash],
            )
            .map_err(|e| e.to_string())?;
            Ok(())
        }

        pub fn verify_master_password(&self, master_password: &str) -> bool {
            let conn = self.conn.lock().unwrap();
            let hash: String = match conn.query_row(
                "SELECT hash FROM master WHERE id = 1",
                [],
                |row| row.get(0),
            ) {
                Ok(h) => h,
                Err(_) => return false,
            };
            bcrypt::verify(master_password, &hash).unwrap_or(false)
        }

        pub fn get_all_entries(&self) -> Vec<Entry> {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn
                .prepare("SELECT id, site_name, url, username, password_encrypted, created_at, updated_at FROM entries ORDER BY site_name ASC")
                .unwrap();
            stmt.query_map([], |row| {
                Ok(Entry {
                    id: row.get(0)?,
                    site_name: row.get(1)?,
                    url: row.get(2)?,
                    username: row.get(3)?,
                    password_encrypted: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
        }

        pub fn add_entry(
            &self,
            site_name: &str,
            url: &str,
            username: &str,
            plain_password: &str,
            master_password: &str,
        ) -> Result<i64, String> {
            let encrypted = encrypt(plain_password, master_password)?;
            let conn = self.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO entries (site_name, url, username, password_encrypted) VALUES (?1, ?2, ?3, ?4)",
                params![site_name, url, username, encrypted],
            )
            .map_err(|e| e.to_string())?;
            Ok(conn.last_insert_rowid())
        }

        pub fn delete_entry(&self, entry_id: i64) -> Result<(), String> {
            let conn = self.conn.lock().unwrap();
            conn.execute("DELETE FROM entries WHERE id = ?1", params![entry_id])
                .map_err(|e| e.to_string())?;
            Ok(())
        }

        pub fn search_entries(&self, query: &str) -> Vec<Entry> {
            let conn = self.conn.lock().unwrap();
            let pattern = format!("%{}%", query);
            let mut stmt = conn
                .prepare(
                    "SELECT id, site_name, url, username, password_encrypted, created_at, updated_at
                     FROM entries
                     WHERE site_name LIKE ?1 OR url LIKE ?1 OR username LIKE ?1
                     ORDER BY site_name ASC",
                )
                .unwrap();
            stmt.query_map(params![pattern], |row| {
                Ok(Entry {
                    id: row.get(0)?,
                    site_name: row.get(1)?,
                    url: row.get(2)?,
                    username: row.get(3)?,
                    password_encrypted: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
        }

        pub fn get_decrypted_password(
            &self,
            entry_id: i64,
            master_password: &str,
        ) -> Result<String, String> {
            let conn = self.conn.lock().unwrap();
            let encrypted: String = conn
                .query_row(
                    "SELECT password_encrypted FROM entries WHERE id = ?1",
                    params![entry_id],
                    |row| row.get(0),
                )
                .map_err(|e| e.to_string())?;
            decrypt(&encrypted, master_password)
        }
    }
}

const BLUE: &str = "\x1b[38;5;69m";
const CYAN: &str = "\x1b[38;5;75m";
const WHITE: &str = "\x1b[38;5;255m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";
const GREEN: &str = "\x1b[38;5;78m";
const YELLOW: &str = "\x1b[38;5;220m";
const RED: &str = "\x1b[38;5;196m";

fn print_banner() {
    println!();
    println!("  {BLUE}╔══════════════════════════════════════════════════════════╗{RESET}");
    println!("  {BLUE}║{RESET}                                                          {BLUE}║{RESET}");
    println!("  {BLUE}║{RESET}   {CYAN}██████{RESET}  {CYAN}██{RESET}   {CYAN}██{RESET}  {CYAN}█████{RESET}  {CYAN}██{RESET}    {CYAN}██{RESET} {CYAN}██{RESET}     {CYAN}████████{RESET}    {BLUE}║{RESET}");
    println!("  {BLUE}║{RESET}   {CYAN}██{RESET}  {CYAN}██{RESET}  {CYAN}██{RESET}   {CYAN}██{RESET} {CYAN}██{RESET}   {CYAN}██{RESET} {CYAN}██{RESET}    {CYAN}██{RESET} {CYAN}██{RESET}        {CYAN}██{RESET}       {BLUE}║{RESET}");
    println!("  {BLUE}║{RESET}   {CYAN}██████{RESET}   {CYAN}██{RESET} {CYAN}██{RESET}  {CYAN}███████{RESET} {CYAN}██{RESET}    {CYAN}██{RESET} {CYAN}██{RESET}        {CYAN}██{RESET}       {BLUE}║{RESET}");
    println!("  {BLUE}║{RESET}   {CYAN}██{RESET}        {CYAN}███{RESET}   {CYAN}██{RESET}   {CYAN}██{RESET} {CYAN}██{RESET}    {CYAN}██{RESET} {CYAN}██{RESET}        {CYAN}██{RESET}       {BLUE}║{RESET}");
    println!("  {BLUE}║{RESET}   {CYAN}██{RESET}         {CYAN}█{RESET}    {CYAN}██{RESET}   {CYAN}██{RESET}  {CYAN}██████{RESET}  {CYAN}██████{RESET}    {CYAN}██{RESET}       {BLUE}║{RESET}");
    println!("  {BLUE}║{RESET}                                                          {BLUE}║{RESET}");
    println!("  {BLUE}║{RESET}   {WHITE}{BOLD}Password Vault CLI{RESET}  {DIM}v1.0.0{RESET}                           {BLUE}║{RESET}");
    println!("  {BLUE}║{RESET}   {DIM}Secure, encrypted password management{RESET}                {BLUE}║{RESET}");
    println!("  {BLUE}║{RESET}   {DIM}from your terminal{RESET}                                    {BLUE}║{RESET}");
    println!("  {BLUE}║{RESET}                                                          {BLUE}║{RESET}");
    println!("  {BLUE}╚══════════════════════════════════════════════════════════╝{RESET}");
    println!();
}

fn print_help() {
    println!("  {WHITE}{BOLD}Commands:{RESET}");
    println!("    {CYAN}list{RESET}       {DIM}│{RESET} List all saved entries");
    println!("    {CYAN}search{RESET}     {DIM}│{RESET} Search entries by name, URL, or username");
    println!("    {CYAN}add{RESET}        {DIM}│{RESET} Add a new password entry");
    println!("    {CYAN}show{RESET}       {DIM}│{RESET} Show password for an entry");
    println!("    {CYAN}delete{RESET}     {DIM}│{RESET} Delete an entry");
    println!("    {CYAN}gen{RESET}        {DIM}│{RESET} Generate a random password");
    println!("    {CYAN}otp{RESET}        {DIM}│{RESET} Show current TOTP code for an entry");
    println!("    {CYAN}otp setup{RESET}  {DIM}│{RESET} Set up TOTP for an entry");
    println!("    {CYAN}otp uri{RESET}    {DIM}│{RESET} Set up TOTP from otpauth:// URI");
    println!("    {CYAN}otp remove{RESET} {DIM}│{RESET} Remove TOTP from an entry");
    println!("    {CYAN}help{RESET}       {DIM}│{RESET} Show this help message");
    println!("    {CYAN}quit{RESET}       {DIM}│{RESET} Exit the CLI");
    println!();
}

fn prompt(msg: &str) -> String {
    print!("  {BLUE}>{RESET} {msg}");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn prompt_password(msg: &str) -> String {
    print!("  {BLUE}>{RESET} {msg}");
    io::stdout().flush().unwrap();
    rpassword::read_password().unwrap_or_default()
}

fn print_divider() {
    println!("  {DIM}─────────────────────────────────────────────────────{RESET}");
}

fn print_entries(entries: &[cli_core::Entry]) {
    if entries.is_empty() {
        println!("  {DIM}No entries found.{RESET}");
        return;
    }

    println!();
    println!(
        "  {WHITE}{BOLD}{:<4}  {:<20}  {:<24}  {:<20}{RESET}",
        "ID", "Site", "Username", "URL"
    );
    print_divider();

    for entry in entries {
        let url = entry.url.as_deref().unwrap_or("");
        let site = if entry.site_name.len() > 18 {
            format!("{}...", &entry.site_name[..18])
        } else {
            entry.site_name.clone()
        };
        let user = if entry.username.len() > 22 {
            format!("{}...", &entry.username[..22])
        } else {
            entry.username.clone()
        };
        let url_display = if url.len() > 18 {
            format!("{}...", &url[..18])
        } else {
            url.to_string()
        };
        println!(
            "  {CYAN}{:<4}{RESET}  {:<20}  {DIM}{:<24}{RESET}  {DIM}{:<20}{RESET}",
            entry.id, site, user, url_display
        );
    }
    println!();
}

fn main() {
    print_banner();

    let db = match cli_core::Database::open() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("  {RED}Error opening database: {e}{RESET}");
            std::process::exit(1);
        }
    };

    // Authenticate
    let master_password;
    if !db.has_master_password() {
        println!("  {YELLOW}First time setup - create your master password{RESET}");
        loop {
            let pw = prompt_password("Master password: ");
            if pw.len() < 8 {
                println!("  {RED}Password must be at least 8 characters{RESET}");
                continue;
            }
            let confirm = prompt_password("Confirm password: ");
            if pw != confirm {
                println!("  {RED}Passwords do not match{RESET}");
                continue;
            }
            match db.set_master_password(&pw) {
                Ok(_) => {
                    master_password = pw;
                    println!("  {GREEN}Master password created successfully!{RESET}");
                    break;
                }
                Err(e) => {
                    println!("  {RED}Error: {e}{RESET}");
                }
            }
        }
    } else {
        loop {
            let pw = prompt_password("Master password: ");
            if db.verify_master_password(&pw) {
                master_password = pw;
                println!("  {GREEN}Vault unlocked{RESET}");
                break;
            } else {
                println!("  {RED}Incorrect password{RESET}");
            }
        }
    }

    println!();
    print_help();

    // REPL
    loop {
        let input = prompt(&format!("{BOLD}vault{RESET} {DIM}${RESET} "));
        let parts: Vec<&str> = input.split_whitespace().collect();
        let cmd = parts.first().map(|s| s.to_lowercase()).unwrap_or_default();

        match cmd.as_str() {
            "list" | "ls" => {
                let entries = db.get_all_entries();
                print_entries(&entries);
            }
            "search" | "find" => {
                let query = if parts.len() > 1 {
                    parts[1..].join(" ")
                } else {
                    prompt("Search query: ")
                };
                let entries = db.search_entries(&query);
                print_entries(&entries);
            }
            "add" | "new" => {
                let site_name = prompt("Site name: ");
                if site_name.is_empty() {
                    println!("  {RED}Site name is required{RESET}");
                    continue;
                }
                let url = prompt("URL (optional): ");
                let username = prompt("Username: ");
                if username.is_empty() {
                    println!("  {RED}Username is required{RESET}");
                    continue;
                }

                let password = {
                    let choice = prompt("Generate password? (y/n): ");
                    if choice.to_lowercase().starts_with('y') {
                        let len_str = prompt("Length (default 16): ");
                        let len: usize = len_str.parse().unwrap_or(16).max(8).min(128);
                        let pw = cli_core::generate_password(len);
                        println!("  {GREEN}Generated:{RESET} {pw}");
                        pw
                    } else {
                        prompt_password("Password: ")
                    }
                };

                if password.is_empty() {
                    println!("  {RED}Password is required{RESET}");
                    continue;
                }

                match db.add_entry(&site_name, &url, &username, &password, &master_password) {
                    Ok(id) => println!("  {GREEN}Entry added (ID: {id}){RESET}"),
                    Err(e) => println!("  {RED}Error: {e}{RESET}"),
                }
            }
            "show" | "get" | "reveal" => {
                let id_str = if parts.len() > 1 {
                    parts[1].to_string()
                } else {
                    prompt("Entry ID: ")
                };
                let id: i64 = match id_str.parse() {
                    Ok(id) => id,
                    Err(_) => {
                        println!("  {RED}Invalid ID{RESET}");
                        continue;
                    }
                };
                match db.get_decrypted_password(id, &master_password) {
                    Ok(pw) => {
                        println!("  {GREEN}Password:{RESET} {pw}");
                    }
                    Err(e) => println!("  {RED}Error: {e}{RESET}"),
                }
            }
            "delete" | "rm" | "remove" => {
                let id_str = if parts.len() > 1 {
                    parts[1].to_string()
                } else {
                    prompt("Entry ID: ")
                };
                let id: i64 = match id_str.parse() {
                    Ok(id) => id,
                    Err(_) => {
                        println!("  {RED}Invalid ID{RESET}");
                        continue;
                    }
                };
                let confirm = prompt(&format!(
                    "{YELLOW}Delete entry {id}? This cannot be undone (y/n): {RESET}"
                ));
                if confirm.to_lowercase().starts_with('y') {
                    match db.delete_entry(id) {
                        Ok(_) => println!("  {GREEN}Entry deleted{RESET}"),
                        Err(e) => println!("  {RED}Error: {e}{RESET}"),
                    }
                } else {
                    println!("  {DIM}Cancelled{RESET}");
                }
            }
            "gen" | "generate" => {
                let len_str = if parts.len() > 1 {
                    parts[1].to_string()
                } else {
                    prompt("Length (default 16): ")
                };
                let len: usize = len_str.parse().unwrap_or(16).max(8).min(128);
                let pw = cli_core::generate_password(len);
                println!("  {GREEN}Generated:{RESET} {pw}");
            }
            "otp" => {
                let sub = parts.get(1).map(|s| s.to_lowercase()).unwrap_or_default();
                match sub.as_str() {
                    "setup" => {
                        let id_str = if parts.len() > 2 {
                            parts[2].to_string()
                        } else {
                            prompt("Entry ID: ")
                        };
                        let id: i64 = match id_str.parse() {
                            Ok(id) => id,
                            Err(_) => { println!("  {RED}Invalid ID{RESET}"); continue; }
                        };
                        let secret = cli_core::generate_totp_secret_b32();
                        println!("  {GREEN}Generated TOTP secret:{RESET} {secret}");
                        println!("  {DIM}Add this secret to your authenticator app.{RESET}");
                        match cli_core::setup_totp_secret(&db, id, &secret, &master_password) {
                            Ok(_) => println!("  {GREEN}TOTP enabled for entry {id}{RESET}"),
                            Err(e) => println!("  {RED}Error: {e}{RESET}"),
                        }
                    }
                    "uri" => {
                        let id_str = if parts.len() > 2 {
                            parts[2].to_string()
                        } else {
                            prompt("Entry ID: ")
                        };
                        let id: i64 = match id_str.parse() {
                            Ok(id) => id,
                            Err(_) => { println!("  {RED}Invalid ID{RESET}"); continue; }
                        };
                        let uri = prompt("otpauth:// URI: ");
                        match cli_core::parse_otpauth_uri(&uri) {
                            Ok(params) => {
                                match cli_core::setup_totp_secret(&db, id, &params.secret, &master_password) {
                                    Ok(_) => println!("  {GREEN}TOTP enabled for entry {id}{RESET}"),
                                    Err(e) => println!("  {RED}Error: {e}{RESET}"),
                                }
                            }
                            Err(e) => println!("  {RED}Error: {e}{RESET}"),
                        }
                    }
                    "remove" | "rm" | "disable" => {
                        let id_str = if parts.len() > 2 {
                            parts[2].to_string()
                        } else {
                            prompt("Entry ID: ")
                        };
                        let id: i64 = match id_str.parse() {
                            Ok(id) => id,
                            Err(_) => { println!("  {RED}Invalid ID{RESET}"); continue; }
                        };
                        let confirm = prompt(&format!(
                            "{YELLOW}Remove TOTP from entry {id}? (y/n): {RESET}"
                        ));
                        if confirm.to_lowercase().starts_with('y') {
                            match cli_core::remove_totp_secret(&db, id) {
                                Ok(_) => println!("  {GREEN}TOTP removed from entry {id}{RESET}"),
                                Err(e) => println!("  {RED}Error: {e}{RESET}"),
                            }
                        } else {
                            println!("  {DIM}Cancelled{RESET}");
                        }
                    }
                    _ => {
                        // Default: show OTP code
                        let id_str = if sub.is_empty() {
                            if parts.len() > 1 { parts[1].to_string() } else { prompt("Entry ID: ") }
                        } else {
                            sub.clone()
                        };
                        let id: i64 = match id_str.parse() {
                            Ok(id) => id,
                            Err(_) => { println!("  {RED}Invalid ID. Usage: otp <id>, otp setup <id>, otp uri <id>, otp remove <id>{RESET}"); continue; }
                        };
                        match cli_core::get_totp_params(&db, id, &master_password) {
                            Ok(params) => {
                                match cli_core::generate_totp_code(&params) {
                                    Ok((code, remaining)) => {
                                        let half = code.len() / 2;
                                        let display = format!("{} {}", &code[..half], &code[half..]);
                                        println!("  {GREEN}Code:{RESET} {BOLD}{display}{RESET}  {DIM}({remaining}s remaining){RESET}");
                                    }
                                    Err(e) => println!("  {RED}Error: {e}{RESET}"),
                                }
                            }
                            Err(e) => println!("  {RED}{e}{RESET}"),
                        }
                    }
                }
            }
            "help" | "?" => {
                print_help();
            }
            "quit" | "exit" | "q" => {
                println!("  {DIM}Vault locked. Goodbye.{RESET}");
                println!();
                break;
            }
            "" => {}
            _ => {
                println!("  {RED}Unknown command:{RESET} {cmd}. Type {CYAN}help{RESET} for commands.");
            }
        }
    }
}
