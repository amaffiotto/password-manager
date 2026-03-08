use std::io::{self, Read, Write};

fn main() {
    loop {
        // Read 4-byte little-endian length header from stdin
        let mut header = [0u8; 4];
        if io::stdin().read_exact(&mut header).is_err() {
            break;
        }
        let length = u32::from_le_bytes(header) as usize;

        if length == 0 || length > 1_048_576 {
            break;
        }

        // Read the JSON body
        let mut body = vec![0u8; length];
        if io::stdin().read_exact(&mut body).is_err() {
            break;
        }

        let message: serde_json::Value = match serde_json::from_slice(&body) {
            Ok(v) => v,
            Err(_) => break,
        };

        let request_id = message.get("_requestId").cloned();

        // Forward to local HTTP server
        let (port, auth_token) = match get_server_info() {
            Some(info) => info,
            None => {
                write_response(
                    serde_json::json!({"error": "Desktop app is not running"}),
                    request_id.as_ref(),
                );
                continue;
            }
        };

        let mut payload = message.clone();
        if let Some(obj) = payload.as_object_mut() {
            obj.remove("_requestId");
        }

        let response = forward_to_server(port, &payload, &auth_token);
        write_response(response, request_id.as_ref());
    }
}

/// Read server port and auth token from the port file.
/// Format: "port\ntoken"
fn get_server_info() -> Option<(u16, String)> {
    let port_file = get_port_file_path()?;
    let content = std::fs::read_to_string(port_file).ok()?;
    let mut lines = content.lines();
    let port: u16 = lines.next()?.trim().parse().ok()?;
    let token = lines.next()?.trim().to_string();
    if token.is_empty() {
        return None;
    }
    Some((port, token))
}

fn get_port_file_path() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").ok()?;
        Some(
            std::path::PathBuf::from(home)
                .join("Library/Application Support/password-manager/.server-port"),
        )
    }

    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").ok()?;
        Some(
            std::path::PathBuf::from(home)
                .join(".config/password-manager/.server-port"),
        )
    }

    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").ok()?;
        Some(
            std::path::PathBuf::from(appdata)
                .join("password-manager/.server-port"),
        )
    }
}

fn forward_to_server(port: u16, payload: &serde_json::Value, auth_token: &str) -> serde_json::Value {
    let body = serde_json::to_vec(payload).unwrap_or_default();

    // Use a simple blocking HTTP POST with bearer token auth
    match std::net::TcpStream::connect(format!("127.0.0.1:{}", port)) {
        Ok(mut stream) => {
            use std::io::Write as _;
            let request = format!(
                "POST /api HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nAuthorization: Bearer {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                port,
                auth_token,
                body.len()
            );
            if stream.write_all(request.as_bytes()).is_err() {
                return serde_json::json!({"error": "Failed to send request"});
            }
            if stream.write_all(&body).is_err() {
                return serde_json::json!({"error": "Failed to send request body"});
            }

            let mut response = Vec::new();
            if stream.read_to_end(&mut response).is_err() {
                return serde_json::json!({"error": "Failed to read response"});
            }

            // Parse HTTP response - find body after \r\n\r\n
            let response_str = String::from_utf8_lossy(&response);
            if let Some(body_start) = response_str.find("\r\n\r\n") {
                let json_body = &response_str[body_start + 4..];
                serde_json::from_str(json_body)
                    .unwrap_or(serde_json::json!({"error": "Invalid JSON response"}))
            } else {
                serde_json::json!({"error": "Invalid HTTP response"})
            }
        }
        Err(_) => serde_json::json!({"error": format!("Cannot connect to desktop app on port {}", port)}),
    }
}

fn write_response(mut response: serde_json::Value, request_id: Option<&serde_json::Value>) {
    if let Some(id) = request_id {
        response["_requestId"] = id.clone();
    }
    let json = serde_json::to_vec(&response).unwrap_or_default();
    let header = (json.len() as u32).to_le_bytes();
    let _ = io::stdout().write_all(&header);
    let _ = io::stdout().write_all(&json);
    let _ = io::stdout().flush();
}
