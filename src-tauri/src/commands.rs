use std::sync::Arc;
use tauri::State;

use crate::clipboard;
use crate::crypto;
use crate::database::{Database, Entry, ImportEntry, PgpKeyRecord};
use crate::pgp;
use crate::state::{AppState, Settings};
use crate::totp;

// --- Master Password ---

#[tauri::command]
pub fn has_master_password(db: State<'_, Arc<Database>>) -> Result<bool, String> {
    Ok(db.has_master_password())
}

#[tauri::command]
pub fn set_master_password(
    password: String,
    db: State<'_, Arc<Database>>,
    state: State<'_, Arc<AppState>>,
) -> Result<bool, String> {
    db.set_master_password(&password)?;
    state.unlock(password);
    Ok(true)
}

#[tauri::command]
pub fn verify_master_password(
    password: String,
    db: State<'_, Arc<Database>>,
    state: State<'_, Arc<AppState>>,
) -> Result<bool, String> {
    // Check rate limiting
    if state.is_locked_out() {
        return Err("Too many failed attempts. Please wait before trying again.".to_string());
    }

    let valid = db.verify_master_password(&password);
    if valid {
        state.reset_failed_attempts();
        state.unlock(password);
    } else {
        state.record_failed_attempt()?;
    }
    Ok(valid)
}

// --- Vault Entries ---

#[tauri::command]
pub fn get_entries(db: State<'_, Arc<Database>>) -> Result<Vec<SafeEntry>, String> {
    Ok(db.get_all_entries().into_iter().map(SafeEntry::from).collect())
}

/// Entry without the encrypted password field — safe to send to frontend.
#[derive(serde::Serialize)]
pub struct SafeEntry {
    pub id: i64,
    pub site_name: String,
    pub url: Option<String>,
    pub username: String,
    pub created_at: String,
    pub updated_at: String,
    pub has_totp: bool,
}

impl From<Entry> for SafeEntry {
    fn from(e: Entry) -> Self {
        SafeEntry {
            id: e.id,
            site_name: e.site_name,
            url: e.url,
            username: e.username,
            created_at: e.created_at,
            updated_at: e.updated_at,
            has_totp: e.has_totp,
        }
    }
}

#[tauri::command]
pub fn add_entry(
    site_name: String,
    url: String,
    username: String,
    password: String,
    db: State<'_, Arc<Database>>,
    state: State<'_, Arc<AppState>>,
) -> Result<i64, String> {
    state.touch_activity();
    let mp = state.get_master_password()?;
    db.add_entry(&site_name, &url, &username, &password, &mp)
}

#[tauri::command]
pub fn update_entry(
    id: i64,
    site_name: String,
    url: String,
    username: String,
    password: String,
    db: State<'_, Arc<Database>>,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    state.touch_activity();
    let mp = state.get_master_password()?;
    db.update_entry(id, &site_name, &url, &username, &password, &mp)
}

#[tauri::command]
pub fn delete_entry(id: i64, db: State<'_, Arc<Database>>) -> Result<(), String> {
    db.delete_entry(id)
}

#[tauri::command]
pub fn get_decrypted_password(
    id: i64,
    db: State<'_, Arc<Database>>,
    state: State<'_, Arc<AppState>>,
) -> Result<String, String> {
    state.touch_activity();
    let mp = state.get_master_password()?;
    db.get_decrypted_password(id, &mp)
}

#[tauri::command]
pub fn search_entries(
    query: String,
    db: State<'_, Arc<Database>>,
) -> Result<Vec<SafeEntry>, String> {
    Ok(db.search_entries(&query).into_iter().map(SafeEntry::from).collect())
}

// --- Utilities ---

#[tauri::command]
pub fn generate_password(
    length: Option<usize>,
    state: State<'_, Arc<AppState>>,
) -> String {
    let len = length.unwrap_or_else(|| state.get_default_password_length());
    crypto::generate_password(len)
}

#[tauri::command]
pub fn copy_to_clipboard(
    text: String,
    state: State<'_, Arc<AppState>>,
) -> Result<bool, String> {
    clipboard::copy_and_schedule_clear(&text, state.get_clipboard_clear_ms())?;
    Ok(true)
}

#[tauri::command]
pub fn lock_vault(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    state.lock();
    Ok(())
}

// --- Settings ---

#[tauri::command]
pub fn get_settings(state: State<'_, Arc<AppState>>) -> Result<Settings, String> {
    Ok(state.get_settings())
}

#[tauri::command]
pub fn update_settings(
    settings: Settings,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    state.update_settings(settings);
    Ok(())
}

// --- Export / Import ---

#[tauri::command]
pub fn export_vault(
    format: String,
    db: State<'_, Arc<Database>>,
    state: State<'_, Arc<AppState>>,
) -> Result<String, String> {
    let mp = state.get_master_password()?;
    let entries = db.export_entries(&mp)?;

    match format.as_str() {
        "json" => serde_json::to_string_pretty(&entries).map_err(|e| e.to_string()),
        "csv" => {
            let mut wtr = csv::Writer::from_writer(Vec::new());
            for entry in &entries {
                wtr.serialize(entry).map_err(|e| e.to_string())?;
            }
            let data = wtr.into_inner().map_err(|e| e.to_string())?;
            String::from_utf8(data).map_err(|e| e.to_string())
        }
        _ => Err(format!("Unsupported format: {}", format)),
    }
}

#[tauri::command]
pub fn import_vault(
    format: String,
    data: String,
    db: State<'_, Arc<Database>>,
    state: State<'_, Arc<AppState>>,
) -> Result<usize, String> {
    let mp = state.get_master_password()?;

    let entries: Vec<ImportEntry> = match format.as_str() {
        "json" => serde_json::from_str(&data).map_err(|e| e.to_string())?,
        "csv" => {
            let mut rdr = csv::Reader::from_reader(data.as_bytes());
            rdr.deserialize()
                .collect::<Result<Vec<ImportEntry>, _>>()
                .map_err(|e| e.to_string())?
        }
        _ => return Err(format!("Unsupported format: {}", format)),
    };

    db.import_entries(entries, &mp)
}

// --- PGP ---

#[tauri::command]
pub fn generate_pgp_key(
    name: String,
    email: String,
    key_type: String,
    passphrase: String,
    db: State<'_, Arc<Database>>,
    state: State<'_, Arc<AppState>>,
) -> Result<PgpKeyRecord, String> {
    let mp = state.get_master_password()?;
    let key_pair = pgp::generate_key_pair(&name, &email, &key_type, &passphrase)?;

    let id = db.add_pgp_key(
        &name,
        &email,
        &key_type,
        &key_pair.public_key_armored,
        &key_pair.private_key_armored,
        &key_pair.fingerprint,
        &mp,
    )?;

    db.get_pgp_key(id)
}

#[tauri::command]
pub fn get_pgp_keys(db: State<'_, Arc<Database>>) -> Result<Vec<PgpKeyRecord>, String> {
    Ok(db.get_all_pgp_keys())
}

#[tauri::command]
pub fn get_pgp_key(
    key_id: i64,
    db: State<'_, Arc<Database>>,
) -> Result<PgpKeyRecord, String> {
    db.get_pgp_key(key_id)
}

#[tauri::command]
pub fn delete_pgp_key(
    key_id: i64,
    db: State<'_, Arc<Database>>,
) -> Result<(), String> {
    db.delete_pgp_key(key_id)
}

#[tauri::command]
pub fn export_pgp_public_key(
    key_id: i64,
    db: State<'_, Arc<Database>>,
) -> Result<String, String> {
    db.get_pgp_public_key(key_id)
}

#[tauri::command]
pub fn export_pgp_private_key(
    key_id: i64,
    db: State<'_, Arc<Database>>,
    state: State<'_, Arc<AppState>>,
) -> Result<String, String> {
    let mp = state.get_master_password()?;
    db.get_pgp_private_key_decrypted(key_id, &mp)
}

#[tauri::command]
pub fn pgp_encrypt(
    plaintext: String,
    key_id: i64,
    db: State<'_, Arc<Database>>,
) -> Result<String, String> {
    let public_key = db.get_pgp_public_key(key_id)?;
    pgp::encrypt_message(&plaintext, &public_key)
}

#[tauri::command]
pub fn pgp_decrypt(
    encrypted: String,
    key_id: i64,
    passphrase: String,
    db: State<'_, Arc<Database>>,
    state: State<'_, Arc<AppState>>,
) -> Result<String, String> {
    let mp = state.get_master_password()?;
    let private_key = db.get_pgp_private_key_decrypted(key_id, &mp)?;
    pgp::decrypt_message(&encrypted, &private_key, &passphrase)
}

#[tauri::command]
pub fn pgp_sign(
    message: String,
    key_id: i64,
    passphrase: String,
    db: State<'_, Arc<Database>>,
    state: State<'_, Arc<AppState>>,
) -> Result<String, String> {
    let mp = state.get_master_password()?;
    let private_key = db.get_pgp_private_key_decrypted(key_id, &mp)?;
    pgp::sign_message(&message, &private_key, &passphrase)
}

#[tauri::command]
pub fn pgp_verify(
    signed_message: String,
    key_id: i64,
    db: State<'_, Arc<Database>>,
) -> Result<(bool, String), String> {
    let public_key = db.get_pgp_public_key(key_id)?;
    pgp::verify_message(&signed_message, &public_key)
}

// --- TOTP ---

#[tauri::command]
pub fn generate_totp_secret() -> String {
    totp::generate_secret()
}

#[tauri::command]
pub fn setup_totp(
    entry_id: i64,
    secret: String,
    algorithm: String,
    digits: i32,
    period: i32,
    db: State<'_, Arc<Database>>,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    state.touch_activity();
    let mp = state.get_master_password()?;

    // Validate by generating a code
    totp::generate_code(&secret, &algorithm, digits as u32, period as u32)?;

    let encrypted_secret = crypto::encrypt(&secret, &mp)?;
    db.set_totp_secret(entry_id, &encrypted_secret, &algorithm, digits, period)
}

#[tauri::command]
pub fn setup_totp_from_uri(
    entry_id: i64,
    uri: String,
    db: State<'_, Arc<Database>>,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    state.touch_activity();
    let mp = state.get_master_password()?;

    let params = totp::parse_otpauth_uri(&uri)?;
    let encrypted_secret = crypto::encrypt(&params.secret, &mp)?;
    db.set_totp_secret(
        entry_id,
        &encrypted_secret,
        &params.algorithm,
        params.digits as i32,
        params.period as i32,
    )
}

#[derive(serde::Serialize)]
pub struct TotpCodeResult {
    pub code: String,
    pub remaining: u64,
    pub period: u64,
}

#[tauri::command]
pub fn generate_totp_code(
    entry_id: i64,
    db: State<'_, Arc<Database>>,
    state: State<'_, Arc<AppState>>,
) -> Result<TotpCodeResult, String> {
    state.touch_activity();
    let mp = state.get_master_password()?;

    let (encrypted_secret, algorithm, digits, period) = db.get_totp_params(entry_id)?;
    if encrypted_secret.is_empty() {
        return Err("No TOTP configured for this entry".to_string());
    }

    let secret = crypto::decrypt(&encrypted_secret, &mp)?;
    let result = totp::generate_code(&secret, &algorithm, digits as u32, period as u32)?;

    Ok(TotpCodeResult {
        code: result.code,
        remaining: result.remaining,
        period: result.period,
    })
}

#[tauri::command]
pub fn get_totp_qr_svg(
    entry_id: i64,
    db: State<'_, Arc<Database>>,
    state: State<'_, Arc<AppState>>,
) -> Result<String, String> {
    state.touch_activity();
    let mp = state.get_master_password()?;

    let (encrypted_secret, algorithm, digits, period) = db.get_totp_params(entry_id)?;
    if encrypted_secret.is_empty() {
        return Err("No TOTP configured for this entry".to_string());
    }

    let secret = crypto::decrypt(&encrypted_secret, &mp)?;
    let (site_name, username) = db.get_entry_site_name(entry_id)?;
    let account = if username.is_empty() {
        site_name.clone()
    } else {
        username
    };

    let uri = totp::generate_uri(&secret, &account, &site_name, &algorithm, digits as u32, period as u32)?;
    totp::generate_qr_svg(&uri)
}

#[tauri::command]
pub fn remove_totp(
    entry_id: i64,
    db: State<'_, Arc<Database>>,
) -> Result<(), String> {
    db.remove_totp_secret(entry_id)
}

// --- Extension ---

#[tauri::command]
pub fn set_extension_id(id: String) -> Result<(), String> {
    crate::native_host::save_extension_id(&id)
}

#[tauri::command]
pub fn reinstall_native_host() -> Result<(), String> {
    crate::native_host::install_native_host_manifests();
    Ok(())
}
