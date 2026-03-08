use std::path::PathBuf;

const HOST_NAME: &str = "com.passwordmanager.app";

// Stable extension ID derived from the key in extension/manifest.json.
// If you change the key, regenerate this with:
//   echo "<key_b64>" | base64 -d | shasum -a 256 | cut -c1-32 | tr '0123456789abcdef' 'abcdefghijklmnop'
const EXTENSION_ID: &str = "ddjeepljblipgikmpdpiddbckeokpamd";

/// Install native messaging host manifests for all supported browsers.
/// Called automatically on desktop app startup.
pub fn install_native_host_manifests() {
    let native_host_path = match find_native_host_binary() {
        Some(p) => p,
        None => {
            eprintln!("Native host binary not found; skipping manifest install");
            return;
        }
    };

    let manifest_dirs = get_browser_manifest_dirs();
    if manifest_dirs.is_empty() {
        return;
    }

    let manifest = build_manifest(&native_host_path);

    for (browser, dir) in &manifest_dirs {
        if let Err(e) = write_manifest(dir, &manifest) {
            eprintln!("Failed to install native host manifest for {}: {}", browser, e);
        }
    }
}

/// Locate the native host binary.
/// In release builds it sits next to the main executable.
/// In dev builds it may be in the cargo target directory.
fn find_native_host_binary() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let exe_dir = exe.parent()?;

    #[cfg(target_os = "windows")]
    let bin_name = "password-manager-native-host.exe";
    #[cfg(not(target_os = "windows"))]
    let bin_name = "password-manager-native-host";

    // Check next to the running executable (release bundle)
    let candidate = exe_dir.join(bin_name);
    if candidate.exists() {
        return Some(candidate);
    }

    // macOS .app bundle: binary is in Contents/MacOS/
    #[cfg(target_os = "macos")]
    {
        if let Some(macos_dir) = exe_dir.parent() {
            let candidate = macos_dir.join("MacOS").join(bin_name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    // Dev fallback: check cargo target directories
    let dev_paths = [
        exe_dir.join("../..").join(bin_name),
        exe_dir.join(bin_name),
    ];
    for path in &dev_paths {
        if let Ok(canonical) = path.canonicalize() {
            if canonical.exists() {
                return Some(canonical);
            }
        }
    }

    None
}

fn build_manifest(native_host_path: &PathBuf) -> String {
    let path_str = native_host_path.to_string_lossy().replace('\\', "\\\\");

    // In release, use the published Chrome Web Store extension ID.
    // During development, this uses a placeholder that gets replaced by install script.
    // For self-distribution, the key in manifest.json pins a stable ID.
    let allowed_origins = get_allowed_origins();

    format!(
        r#"{{
  "name": "{}",
  "description": "Password Manager native messaging host — bridges the browser extension to the desktop app",
  "path": "{}",
  "type": "stdio",
  "allowed_origins": [
    {}
  ]
}}"#,
        HOST_NAME,
        path_str,
        allowed_origins
    )
}

fn get_allowed_origins() -> String {
    // Read the extension ID from a config file if it exists, otherwise use
    // a wildcard-style list of common IDs. Users who load unpacked will need
    // to update this, but published extensions have a fixed ID.
    let config_dir = dirs::config_dir();
    if let Some(dir) = config_dir {
        let id_file = dir.join("password-manager").join(".extension-id");
        if let Ok(id) = std::fs::read_to_string(&id_file) {
            let id = id.trim();
            if !id.is_empty() {
                return format!("\"chrome-extension://{}/\"", id);
            }
        }
    }

    // Use the stable ID derived from the key in extension/manifest.json
    format!("\"chrome-extension://{}/\"", EXTENSION_ID)
}

fn write_manifest(dir: &PathBuf, manifest: &str) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{}.json", HOST_NAME));
    std::fs::write(&path, manifest).map_err(|e| e.to_string())?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o644);
        let _ = std::fs::set_permissions(&path, perms);
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn get_browser_manifest_dirs() -> Vec<(&'static str, PathBuf)> {
    let mut dirs = Vec::new();

    if let Ok(appdata) = std::env::var("LOCALAPPDATA") {
        let base = PathBuf::from(&appdata);
        let browsers = [
            ("Chrome", "Google/Chrome/User Data/NativeMessagingHosts"),
            ("Edge", "Microsoft/Edge/User Data/NativeMessagingHosts"),
            ("Brave", "BraveSoftware/Brave-Browser/User Data/NativeMessagingHosts"),
            ("Chromium", "Chromium/User Data/NativeMessagingHosts"),
        ];
        for (name, rel) in &browsers {
            let dir = base.join(rel);
            let parent = dir.parent().unwrap_or(&dir);
            if parent.exists() || *name == "Chrome" {
                dirs.push((*name, dir));
            }
        }
    }

    // Also write registry key for Windows native messaging
    write_windows_registry(&dirs);

    dirs
}

#[cfg(target_os = "windows")]
fn write_windows_registry(dirs: &[(&str, PathBuf)]) {
    use std::process::Command;
    for (_, dir) in dirs {
        let manifest_path = dir.join(format!("{}.json", HOST_NAME));
        let manifest_str = manifest_path.to_string_lossy();
        // Write to HKCU so no admin rights needed
        let key = format!(
            "HKCU\\Software\\Google\\Chrome\\NativeMessagingHosts\\{}",
            HOST_NAME
        );
        let _ = Command::new("reg")
            .args(["add", &key, "/ve", "/t", "REG_SZ", "/d", &manifest_str, "/f"])
            .output();
    }
}

#[cfg(target_os = "macos")]
fn get_browser_manifest_dirs() -> Vec<(&'static str, PathBuf)> {
    let mut dirs = Vec::new();
    let home = match std::env::var("HOME") {
        Ok(h) => PathBuf::from(h),
        Err(_) => return dirs,
    };

    let browsers = [
        ("Chrome", "Library/Application Support/Google/Chrome/NativeMessagingHosts"),
        ("Chromium", "Library/Application Support/Chromium/NativeMessagingHosts"),
        ("Brave", "Library/Application Support/BraveSoftware/Brave-Browser/NativeMessagingHosts"),
        ("Edge", "Library/Application Support/Microsoft Edge/NativeMessagingHosts"),
        ("Vivaldi", "Library/Application Support/Vivaldi/NativeMessagingHosts"),
    ];

    for (name, rel) in &browsers {
        let dir = home.join(rel);
        let parent = dir.parent().unwrap_or(&dir);
        if parent.exists() || *name == "Chrome" {
            dirs.push((*name, dir));
        }
    }

    dirs
}

#[cfg(target_os = "linux")]
fn get_browser_manifest_dirs() -> Vec<(&'static str, PathBuf)> {
    let mut dirs = Vec::new();
    let home = match std::env::var("HOME") {
        Ok(h) => PathBuf::from(h),
        Err(_) => return dirs,
    };

    let browsers = [
        ("Chrome", ".config/google-chrome/NativeMessagingHosts"),
        ("Chromium", ".config/chromium/NativeMessagingHosts"),
        ("Brave", ".config/BraveSoftware/Brave-Browser/NativeMessagingHosts"),
        ("Edge", ".config/microsoft-edge/NativeMessagingHosts"),
        ("Vivaldi", ".config/vivaldi/NativeMessagingHosts"),
    ];

    for (name, rel) in &browsers {
        let dir = home.join(rel);
        let parent = dir.parent().unwrap_or(&dir);
        if parent.exists() || *name == "Chrome" {
            dirs.push((*name, dir));
        }
    }

    dirs
}

/// Save the extension ID so future manifest writes use the correct origin.
/// Called from Settings or via CLI.
#[allow(dead_code)]
pub fn save_extension_id(id: &str) -> Result<(), String> {
    let config_dir = dirs::config_dir().ok_or("Cannot find config directory")?;
    let app_dir = config_dir.join("password-manager");
    std::fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    std::fs::write(app_dir.join(".extension-id"), id).map_err(|e| e.to_string())?;
    // Re-install manifests with updated ID
    install_native_host_manifests();
    Ok(())
}
