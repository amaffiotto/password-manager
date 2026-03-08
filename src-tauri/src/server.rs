use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use rand::RngCore;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::database::Database;
use crate::state::AppState;

pub struct ServerState {
    pub db: Arc<Database>,
    pub app_state: Arc<AppState>,
    pub auth_token: String,
}

/// Generate a cryptographically secure random bearer token.
fn generate_auth_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// Middleware: validate the bearer token and origin on every request.
async fn auth_middleware(
    axum::extract::State(state): axum::extract::State<Arc<ServerState>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Response {
    // Check Authorization: Bearer <token>
    let authorized = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|token| token == state.auth_token)
        .unwrap_or(false);

    if !authorized {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Unauthorized"})),
        )
            .into_response();
    }

    // Validate Origin — reject requests from external websites (CSRF protection)
    if let Some(origin) = headers.get("origin").and_then(|v| v.to_str().ok()) {
        let origin_lower = origin.to_lowercase();
        let allowed = origin_lower.starts_with("http://localhost")
            || origin_lower.starts_with("http://127.0.0.1")
            || origin_lower.starts_with("https://localhost")
            || origin_lower.starts_with("https://127.0.0.1")
            || origin_lower.starts_with("tauri://");

        if !allowed {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({"error": "Forbidden: invalid origin"})),
            )
                .into_response();
        }
    }

    next.run(request).await
}

async fn handle_api(
    axum::extract::State(state): axum::extract::State<Arc<ServerState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let action = body["action"].as_str().unwrap_or("");

    match action {
        "status" => Json(json!({ "unlocked": state.app_state.is_unlocked() })),

        "search-entries" => {
            if !state.app_state.is_unlocked() {
                return Json(json!({ "error": "Vault is locked" }));
            }
            let query = body["query"].as_str().unwrap_or("");
            let entries = state.db.search_entries(query);
            let safe_entries: Vec<Value> = entries
                .iter()
                .map(|e| {
                    json!({
                        "id": e.id,
                        "site_name": e.site_name,
                        "url": e.url,
                        "username": e.username
                    })
                })
                .collect();
            Json(json!({ "entries": safe_entries }))
        }

        "get-all-entries" => {
            if !state.app_state.is_unlocked() {
                return Json(json!({ "error": "Vault is locked" }));
            }
            let entries = state.db.get_all_entries();
            let safe_entries: Vec<Value> = entries
                .iter()
                .map(|e| {
                    json!({
                        "id": e.id,
                        "site_name": e.site_name,
                        "url": e.url,
                        "username": e.username
                    })
                })
                .collect();
            Json(json!({ "entries": safe_entries }))
        }

        "get-decrypted-password" => {
            if !state.app_state.is_unlocked() {
                return Json(json!({ "error": "Vault is locked" }));
            }
            let entry_id = body["entryId"].as_i64().unwrap_or(0);
            match state.app_state.get_master_password() {
                Ok(mp) => {
                    let password = state.db.get_decrypted_password(entry_id, &mp);
                    let username = state.db.get_entry_username(entry_id);
                    match password {
                        Ok(p) => Json(json!({ "password": p, "username": username })),
                        Err(e) => Json(json!({ "error": e })),
                    }
                }
                Err(e) => Json(json!({ "error": e })),
            }
        }

        _ => Json(json!({ "error": format!("Unknown action: {}", action) })),
    }
}

/// Start the local HTTP server for extension communication.
/// Returns the port number.
pub async fn start_server(state: Arc<ServerState>) -> Result<u16, String> {
    let app = Router::new()
        .route("/api", post(handle_api))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| e.to_string())?;

    let port = listener.local_addr().map_err(|e| e.to_string())?.port();

    tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });

    Ok(port)
}

fn get_port_file_path() -> Result<std::path::PathBuf, String> {
    let config_dir = dirs::config_dir().ok_or("Cannot find config directory")?;
    Ok(config_dir.join("password-manager").join(".server-port"))
}

/// Write port and auth token to the port file.
/// Format: "port\ntoken"
/// File permissions are set to owner-only (0600) on Unix.
pub fn write_server_info(port: u16, token: &str) -> Result<(), String> {
    let path = get_port_file_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = format!("{}\n{}", port, token);
    std::fs::write(&path, content).map_err(|e| e.to_string())?;

    // Set restrictive permissions (owner read/write only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&path, perms).map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn stop_server() {
    if let Ok(path) = get_port_file_path() {
        let _ = std::fs::remove_file(path);
    }
}

pub fn create_server_state(db: Arc<Database>, app_state: Arc<AppState>) -> Arc<ServerState> {
    Arc::new(ServerState {
        db,
        app_state,
        auth_token: generate_auth_token(),
    })
}
