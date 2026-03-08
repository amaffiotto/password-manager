mod clipboard;
mod commands;
mod crypto;
mod database;
mod native_host;
mod pgp;
mod server;
mod state;
mod totp;

use std::sync::Arc;
use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Resolve database path
            let config_dir = dirs::config_dir().expect("Cannot find config directory");
            let app_data = config_dir.join("password-manager");
            std::fs::create_dir_all(&app_data).ok();
            let db_path = app_data.join("vault.db");

            // Initialize database
            let db = database::Database::init(db_path.to_str().unwrap())
                .expect("Failed to initialize database");
            let db = Arc::new(db);

            // Set restrictive permissions on the database file (owner-only)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = std::fs::Permissions::from_mode(0o600);
                let _ = std::fs::set_permissions(&db_path, perms);
            }

            // Initialize app state
            let app_state = Arc::new(state::AppState::new());

            // Register as Tauri managed state
            app.manage(db.clone());
            app.manage(app_state.clone());

            // Start local HTTP server for extension bridge (with bearer token auth)
            let server_state = server::create_server_state(db.clone(), app_state.clone());
            let auth_token = server_state.auth_token.clone();
            tauri::async_runtime::spawn(async move {
                match server::start_server(server_state).await {
                    Ok(port) => {
                        // Write port + auth token to file (with restrictive permissions)
                        if let Err(e) = server::write_server_info(port, &auth_token) {
                            eprintln!("Failed to write server info: {}", e);
                        }
                        println!("Extension bridge running on 127.0.0.1:{}", port);
                    }
                    Err(e) => eprintln!("Failed to start extension bridge: {}", e),
                }
            });

            // Install native messaging host manifests for browser extension
            native_host::install_native_host_manifests();

            // Start auto-lock background task
            let lock_state = app_state.clone();
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                    if lock_state.is_unlocked() && lock_state.check_auto_lock() {
                        lock_state.lock();
                        let _ = app_handle.emit("vault-locked", ());
                    }
                }
            });

            Ok(())
        })
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                server::stop_server();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::has_master_password,
            commands::set_master_password,
            commands::verify_master_password,
            commands::get_entries,
            commands::add_entry,
            commands::update_entry,
            commands::delete_entry,
            commands::get_decrypted_password,
            commands::search_entries,
            commands::generate_password,
            commands::copy_to_clipboard,
            commands::lock_vault,
            commands::get_settings,
            commands::update_settings,
            commands::export_vault,
            commands::import_vault,
            commands::generate_pgp_key,
            commands::get_pgp_keys,
            commands::get_pgp_key,
            commands::delete_pgp_key,
            commands::export_pgp_public_key,
            commands::export_pgp_private_key,
            commands::pgp_encrypt,
            commands::pgp_decrypt,
            commands::pgp_sign,
            commands::pgp_verify,
            commands::generate_totp_secret,
            commands::setup_totp,
            commands::setup_totp_from_uri,
            commands::generate_totp_code,
            commands::get_totp_qr_svg,
            commands::remove_totp,
            commands::set_extension_id,
            commands::reinstall_native_host,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
