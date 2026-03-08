fn main() {
    if std::env::var("TAURI_SKIP_WINRES").is_ok() {
        return;
    }
    tauri_build::build()
}
