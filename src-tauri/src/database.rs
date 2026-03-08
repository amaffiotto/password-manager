use rusqlite::{params, Connection};
use serde::Serialize;
use std::sync::Mutex;

use crate::crypto;

#[derive(Debug, Serialize, Clone)]
pub struct Entry {
    pub id: i64,
    pub site_name: String,
    pub url: Option<String>,
    pub username: String,
    pub password_encrypted: String,
    pub created_at: String,
    pub updated_at: String,
    pub has_totp: bool,
    pub totp_secret_encrypted: Option<String>,
    pub totp_algorithm: String,
    pub totp_digits: i32,
    pub totp_period: i32,
}

#[derive(Debug, Serialize, Clone)]
pub struct PgpKeyRecord {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub key_type: String,
    pub public_key: String,
    pub fingerprint: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct ExportEntry {
    pub site_name: String,
    pub url: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct ImportEntry {
    pub site_name: String,
    pub url: Option<String>,
    pub username: String,
    pub password: String,
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Open or create the database at the given path. Creates tables if needed.
    pub fn init(db_path: &str) -> Result<Self, String> {
        let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

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
            );

            CREATE TABLE IF NOT EXISTS pgp_keys (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                email TEXT NOT NULL,
                key_type TEXT NOT NULL,
                public_key TEXT NOT NULL,
                private_key_encrypted TEXT NOT NULL,
                fingerprint TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );",
        )
        .map_err(|e| e.to_string())?;

        // Migrate: add TOTP columns if not present
        let has_totp_col: bool = conn
            .prepare("SELECT totp_secret_encrypted FROM entries LIMIT 0")
            .is_ok();
        if !has_totp_col {
            conn.execute_batch(
                "ALTER TABLE entries ADD COLUMN totp_secret_encrypted TEXT;
                 ALTER TABLE entries ADD COLUMN totp_algorithm TEXT DEFAULT 'SHA1';
                 ALTER TABLE entries ADD COLUMN totp_digits INTEGER DEFAULT 6;
                 ALTER TABLE entries ADD COLUMN totp_period INTEGER DEFAULT 30;",
            )
            .map_err(|e| e.to_string())?;
        }

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    // --- Master Password ---

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

    // --- Entries ---

    pub fn add_entry(
        &self,
        site_name: &str,
        url: &str,
        username: &str,
        plain_password: &str,
        master_password: &str,
    ) -> Result<i64, String> {
        let encrypted = crypto::encrypt(plain_password, master_password)?;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO entries (site_name, url, username, password_encrypted) VALUES (?1, ?2, ?3, ?4)",
            params![site_name, url, username, encrypted],
        )
        .map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_all_entries(&self) -> Vec<Entry> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, site_name, url, username, password_encrypted, created_at, updated_at, totp_secret_encrypted, totp_algorithm, totp_digits, totp_period FROM entries ORDER BY site_name ASC")
            .unwrap();
        stmt.query_map([], |row| {
            let totp_secret: Option<String> = row.get(7)?;
            Ok(Entry {
                id: row.get(0)?,
                site_name: row.get(1)?,
                url: row.get(2)?,
                username: row.get(3)?,
                password_encrypted: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
                has_totp: totp_secret.is_some(),
                totp_secret_encrypted: totp_secret,
                totp_algorithm: row.get::<_, Option<String>>(8)?.unwrap_or_else(|| "SHA1".to_string()),
                totp_digits: row.get::<_, Option<i32>>(9)?.unwrap_or(6),
                totp_period: row.get::<_, Option<i32>>(10)?.unwrap_or(30),
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
        crypto::decrypt(&encrypted, master_password)
    }

    pub fn get_entry_username(&self, entry_id: i64) -> Option<String> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT username FROM entries WHERE id = ?1",
            params![entry_id],
            |row| row.get(0),
        )
        .ok()
    }

    pub fn update_entry(
        &self,
        entry_id: i64,
        site_name: &str,
        url: &str,
        username: &str,
        plain_password: &str,
        master_password: &str,
    ) -> Result<(), String> {
        let encrypted = crypto::encrypt(plain_password, master_password)?;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE entries SET site_name = ?1, url = ?2, username = ?3, password_encrypted = ?4, updated_at = CURRENT_TIMESTAMP WHERE id = ?5",
            params![site_name, url, username, encrypted, entry_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
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
                "SELECT id, site_name, url, username, password_encrypted, created_at, updated_at, totp_secret_encrypted, totp_algorithm, totp_digits, totp_period
                 FROM entries
                 WHERE site_name LIKE ?1 OR url LIKE ?1 OR username LIKE ?1
                 ORDER BY site_name ASC",
            )
            .unwrap();
        stmt.query_map(params![pattern], |row| {
            let totp_secret: Option<String> = row.get(7)?;
            Ok(Entry {
                id: row.get(0)?,
                site_name: row.get(1)?,
                url: row.get(2)?,
                username: row.get(3)?,
                password_encrypted: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
                has_totp: totp_secret.is_some(),
                totp_secret_encrypted: totp_secret,
                totp_algorithm: row.get::<_, Option<String>>(8)?.unwrap_or_else(|| "SHA1".to_string()),
                totp_digits: row.get::<_, Option<i32>>(9)?.unwrap_or(6),
                totp_period: row.get::<_, Option<i32>>(10)?.unwrap_or(30),
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    // --- Export / Import ---

    pub fn export_entries(&self, master_password: &str) -> Result<Vec<ExportEntry>, String> {
        let entries = self.get_all_entries();
        let mut result = Vec::new();
        for entry in entries {
            let password = crypto::decrypt(&entry.password_encrypted, master_password)?;
            result.push(ExportEntry {
                site_name: entry.site_name,
                url: entry.url.unwrap_or_default(),
                username: entry.username,
                password,
            });
        }
        Ok(result)
    }

    pub fn import_entries(
        &self,
        entries: Vec<ImportEntry>,
        master_password: &str,
    ) -> Result<usize, String> {
        let mut count = 0;
        for entry in &entries {
            self.add_entry(
                &entry.site_name,
                entry.url.as_deref().unwrap_or(""),
                &entry.username,
                &entry.password,
                master_password,
            )?;
            count += 1;
        }
        Ok(count)
    }

    // --- PGP Keys ---

    #[allow(clippy::too_many_arguments)]
    pub fn add_pgp_key(
        &self,
        name: &str,
        email: &str,
        key_type: &str,
        public_key: &str,
        private_key_armored: &str,
        fingerprint: &str,
        master_password: &str,
    ) -> Result<i64, String> {
        let private_key_encrypted = crypto::encrypt(private_key_armored, master_password)?;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO pgp_keys (name, email, key_type, public_key, private_key_encrypted, fingerprint) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![name, email, key_type, public_key, private_key_encrypted, fingerprint],
        )
        .map_err(|e| e.to_string())?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_all_pgp_keys(&self) -> Vec<PgpKeyRecord> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, name, email, key_type, public_key, fingerprint, created_at FROM pgp_keys ORDER BY name ASC")
            .unwrap();
        stmt.query_map([], |row| {
            Ok(PgpKeyRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                email: row.get(2)?,
                key_type: row.get(3)?,
                public_key: row.get(4)?,
                fingerprint: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn get_pgp_key(&self, key_id: i64) -> Result<PgpKeyRecord, String> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, name, email, key_type, public_key, fingerprint, created_at FROM pgp_keys WHERE id = ?1",
            params![key_id],
            |row| {
                Ok(PgpKeyRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    email: row.get(2)?,
                    key_type: row.get(3)?,
                    public_key: row.get(4)?,
                    fingerprint: row.get(5)?,
                    created_at: row.get(6)?,
                })
            },
        )
        .map_err(|e| e.to_string())
    }

    pub fn get_pgp_public_key(&self, key_id: i64) -> Result<String, String> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT public_key FROM pgp_keys WHERE id = ?1",
            params![key_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())
    }

    pub fn get_pgp_private_key_decrypted(
        &self,
        key_id: i64,
        master_password: &str,
    ) -> Result<String, String> {
        let conn = self.conn.lock().unwrap();
        let encrypted: String = conn
            .query_row(
                "SELECT private_key_encrypted FROM pgp_keys WHERE id = ?1",
                params![key_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        crypto::decrypt(&encrypted, master_password)
    }

    pub fn delete_pgp_key(&self, key_id: i64) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM pgp_keys WHERE id = ?1", params![key_id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // --- TOTP ---

    pub fn set_totp_secret(
        &self,
        entry_id: i64,
        secret_encrypted: &str,
        algorithm: &str,
        digits: i32,
        period: i32,
    ) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE entries SET totp_secret_encrypted = ?1, totp_algorithm = ?2, totp_digits = ?3, totp_period = ?4, updated_at = CURRENT_TIMESTAMP WHERE id = ?5",
            params![secret_encrypted, algorithm, digits, period, entry_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_totp_secret_encrypted(&self, entry_id: i64) -> Result<Option<String>, String> {
        let conn = self.conn.lock().unwrap();
        let result: Option<String> = conn
            .query_row(
                "SELECT totp_secret_encrypted FROM entries WHERE id = ?1",
                params![entry_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    pub fn get_totp_params(&self, entry_id: i64) -> Result<(String, String, i32, i32), String> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT totp_secret_encrypted, totp_algorithm, totp_digits, totp_period FROM entries WHERE id = ?1",
            params![entry_id],
            |row| {
                let secret: Option<String> = row.get(0)?;
                Ok((
                    secret.unwrap_or_default(),
                    row.get::<_, Option<String>>(1)?.unwrap_or_else(|| "SHA1".to_string()),
                    row.get::<_, Option<i32>>(2)?.unwrap_or(6),
                    row.get::<_, Option<i32>>(3)?.unwrap_or(30),
                ))
            },
        )
        .map_err(|e| e.to_string())
    }

    pub fn remove_totp_secret(&self, entry_id: i64) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE entries SET totp_secret_encrypted = NULL, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            params![entry_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_entry_site_name(&self, entry_id: i64) -> Result<(String, String), String> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT site_name, username FROM entries WHERE id = ?1",
            params![entry_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())
    }
}
