use std::sync::Mutex;
use std::time::{Duration, Instant};
use zeroize::Zeroize;

const MAX_FAILED_ATTEMPTS: u32 = 10;
const LOCKOUT_DURATION_SECS: u64 = 60;

pub struct AppState {
    master_password: Mutex<Option<String>>,
    last_activity: Mutex<Instant>,
    auto_lock_minutes: Mutex<u64>,
    clipboard_clear_seconds: Mutex<u64>,
    default_password_length: Mutex<usize>,
    failed_attempts: Mutex<u32>,
    lockout_until: Mutex<Option<Instant>>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Settings {
    pub auto_lock_minutes: u64,
    pub default_password_length: usize,
    pub clipboard_clear_seconds: u64,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            master_password: Mutex::new(None),
            last_activity: Mutex::new(Instant::now()),
            auto_lock_minutes: Mutex::new(5),
            clipboard_clear_seconds: Mutex::new(30),
            default_password_length: Mutex::new(16),
            failed_attempts: Mutex::new(0),
            lockout_until: Mutex::new(None),
        }
    }

    /// Check if account is locked out due to too many failed attempts.
    pub fn is_locked_out(&self) -> bool {
        let lockout = self.lockout_until.lock().unwrap();
        if let Some(until) = *lockout {
            if Instant::now() < until {
                return true;
            }
        }
        false
    }

    /// Record a failed login attempt. Returns error if locked out.
    pub fn record_failed_attempt(&self) -> Result<(), String> {
        if self.is_locked_out() {
            let lockout = self.lockout_until.lock().unwrap();
            if let Some(until) = *lockout {
                let remaining = until.duration_since(Instant::now()).as_secs();
                return Err(format!(
                    "Too many failed attempts. Try again in {} seconds.",
                    remaining
                ));
            }
        }

        let mut attempts = self.failed_attempts.lock().unwrap();
        *attempts += 1;

        if *attempts >= MAX_FAILED_ATTEMPTS {
            let mut lockout = self.lockout_until.lock().unwrap();
            *lockout = Some(Instant::now() + Duration::from_secs(LOCKOUT_DURATION_SECS));
            return Err(format!(
                "Too many failed attempts. Locked out for {} seconds.",
                LOCKOUT_DURATION_SECS
            ));
        }

        Ok(())
    }

    /// Reset failed attempts on successful login.
    pub fn reset_failed_attempts(&self) {
        *self.failed_attempts.lock().unwrap() = 0;
        *self.lockout_until.lock().unwrap() = None;
    }

    pub fn unlock(&self, password: String) {
        let mut mp = self.master_password.lock().unwrap();
        *mp = Some(password);
        self.touch_activity();
    }

    pub fn lock(&self) {
        let mut mp = self.master_password.lock().unwrap();
        if let Some(ref mut p) = *mp {
            p.zeroize();
        }
        *mp = None;
    }

    pub fn is_unlocked(&self) -> bool {
        self.master_password.lock().unwrap().is_some()
    }

    pub fn get_master_password(&self) -> Result<String, String> {
        self.touch_activity();
        let mp = self.master_password.lock().unwrap();
        mp.clone().ok_or_else(|| "Vault is locked".to_string())
    }

    pub fn touch_activity(&self) {
        let mut last = self.last_activity.lock().unwrap();
        *last = Instant::now();
    }

    pub fn check_auto_lock(&self) -> bool {
        let last = self.last_activity.lock().unwrap();
        let minutes = *self.auto_lock_minutes.lock().unwrap();
        last.elapsed() > Duration::from_secs(minutes * 60)
    }

    pub fn get_clipboard_clear_ms(&self) -> u64 {
        *self.clipboard_clear_seconds.lock().unwrap() * 1000
    }

    pub fn get_default_password_length(&self) -> usize {
        *self.default_password_length.lock().unwrap()
    }

    pub fn get_settings(&self) -> Settings {
        Settings {
            auto_lock_minutes: *self.auto_lock_minutes.lock().unwrap(),
            default_password_length: *self.default_password_length.lock().unwrap(),
            clipboard_clear_seconds: *self.clipboard_clear_seconds.lock().unwrap(),
        }
    }

    pub fn update_settings(&self, settings: Settings) {
        *self.auto_lock_minutes.lock().unwrap() = settings.auto_lock_minutes;
        *self.default_password_length.lock().unwrap() = settings.default_password_length;
        *self.clipboard_clear_seconds.lock().unwrap() = settings.clipboard_clear_seconds;
    }
}
