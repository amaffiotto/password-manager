use arboard::Clipboard;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

/// Global counter to track clipboard clear generations.
/// Each new copy increments this; the clear thread only clears
/// if its generation matches (preventing clearing newer content).
static CLIPBOARD_GENERATION: AtomicU64 = AtomicU64::new(0);

pub fn copy_and_schedule_clear(text: &str, clear_ms: u64) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(text).map_err(|e| e.to_string())?;

    let generation = CLIPBOARD_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(clear_ms));
        if CLIPBOARD_GENERATION.load(Ordering::SeqCst) == generation {
            if let Ok(mut cb) = Clipboard::new() {
                let _ = cb.set_text(String::new());
            }
        }
    });

    Ok(())
}
