use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

pub fn get_config_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Could not determine home directory");
    let config_dir = home.join(".rewrit");

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).expect("Could not create config directory");
        // Restrict directory permissions to owner only (Unix)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&config_dir, fs::Permissions::from_mode(0o700));
        }
    }

    config_dir
}

/// Write a file with restricted permissions (0600 on Unix)
fn write_private(path: &std::path::Path, contents: &str) -> std::io::Result<()> {
    fs::write(path, contents)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

pub fn get_device_id() -> Result<String, String> {
    let device_id_path = get_config_dir().join("device_id");

    // Try to read existing device ID
    if device_id_path.exists() {
        match fs::read_to_string(&device_id_path) {
            Ok(id) => {
                let trimmed = id.trim();
                if !trimmed.is_empty() {
                    return Ok(trimmed.to_string());
                }
            }
            Err(_) => {}
        }
    }

    // Generate new device ID
    let new_id = Uuid::new_v4().to_string();

    // Save to file with restricted permissions
    match write_private(&device_id_path, &new_id) {
        Ok(_) => Ok(new_id),
        Err(e) => Err(format!("Failed to save device ID: {}", e)),
    }
}

fn validate_provider(provider: &str) -> Result<(), String> {
    if provider.contains('/') || provider.contains('\\') || provider.contains("..") {
        return Err(format!("Invalid provider name: {}", provider));
    }
    Ok(())
}

pub fn get_api_key(provider: &str) -> Option<String> {
    if validate_provider(provider).is_err() { return None; }
    let filename = format!("{}_api_key", provider);
    let path = get_config_dir().join(filename);
    if path.exists() {
        if let Ok(key) = fs::read_to_string(&path) {
            let trimmed = key.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }
    None
}

pub fn set_api_key(provider: &str, key: &str) -> Result<(), String> {
    validate_provider(provider)?;
    let filename = format!("{}_api_key", provider);
    let path = get_config_dir().join(filename);
    write_private(&path, key.trim())
        .map_err(|e| format!("Failed to save API key: {}", e))
}

pub fn clear_api_key(provider: &str) -> Result<(), String> {
    validate_provider(provider)?;
    let filename = format!("{}_api_key", provider);
    let path = get_config_dir().join(filename);
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| format!("Failed to remove API key: {}", e))?;
    }
    Ok(())
}

pub fn get_show_preview() -> bool {
    let path = get_config_dir().join("show_preview");
    if path.exists() {
        if let Ok(val) = fs::read_to_string(&path) {
            return val.trim() == "true";
        }
    }
    true // default: enabled
}

pub fn set_show_preview(enabled: bool) -> Result<(), String> {
    let path = get_config_dir().join("show_preview");
    fs::write(&path, if enabled { "true" } else { "false" })
        .map_err(|e| format!("Failed to save show preview setting: {}", e))
}

pub fn get_quick_select_enabled() -> bool {
    let path = get_config_dir().join("quick_select_enabled");
    if path.exists() {
        if let Ok(val) = fs::read_to_string(&path) {
            return val.trim() == "true";
        }
    }
    true // default: enabled
}

pub fn set_quick_select_enabled(enabled: bool) -> Result<(), String> {
    let path = get_config_dir().join("quick_select_enabled");
    fs::write(&path, if enabled { "true" } else { "false" })
        .map_err(|e| format!("Failed to save quick select enabled: {}", e))
}

pub fn get_stream_response() -> bool {
    let path = get_config_dir().join("stream_response");
    if path.exists() {
        if let Ok(val) = fs::read_to_string(&path) {
            return val.trim() == "true";
        }
    }
    false // default: disabled (current behavior)
}

pub fn set_stream_response(enabled: bool) -> Result<(), String> {
    let path = get_config_dir().join("stream_response");
    fs::write(&path, if enabled { "true" } else { "false" })
        .map_err(|e| format!("Failed to save stream setting: {}", e))
}

fn get_bool(name: &str, default: bool) -> bool {
    match fs::read_to_string(get_config_dir().join(name)) {
        Ok(val) => val.trim() == "true",
        Err(_) => default,
    }
}

fn set_bool(name: &str, enabled: bool) -> Result<(), String> {
    fs::write(get_config_dir().join(name), if enabled { "true" } else { "false" })
        .map_err(|e| format!("Failed to save {}: {}", name, e))
}

pub fn get_update_auto_check() -> bool {
    get_bool("update_auto_check", true)
}

pub fn set_update_auto_check(enabled: bool) -> Result<(), String> {
    set_bool("update_auto_check", enabled)
}

pub fn get_update_auto_install() -> bool {
    get_bool("update_auto_install", true)
}

pub fn set_update_auto_install(enabled: bool) -> Result<(), String> {
    set_bool("update_auto_install", enabled)
}

/// RFC 3339 timestamp of the last successful update check
pub fn get_update_last_check() -> Option<String> {
    fs::read_to_string(get_config_dir().join("update_last_check"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn set_update_last_check(timestamp: &str) -> Result<(), String> {
    fs::write(get_config_dir().join("update_last_check"), timestamp)
        .map_err(|e| format!("Failed to save last update check: {}", e))
}

/// Model chosen for a provider, if the user picked one
pub fn get_model(provider: &str) -> Option<String> {
    if validate_provider(provider).is_err() { return None; }
    fs::read_to_string(get_config_dir().join(format!("{}_model", provider)))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Save the model for a provider; an empty model restores the default
pub fn set_model(provider: &str, model: &str) -> Result<(), String> {
    validate_provider(provider)?;
    let path = get_config_dir().join(format!("{}_model", provider));
    if model.trim().is_empty() {
        if path.exists() {
            fs::remove_file(&path).map_err(|e| format!("Failed to reset model: {}", e))?;
        }
        return Ok(());
    }
    fs::write(&path, model.trim()).map_err(|e| format!("Failed to save model: {}", e))
}

pub fn get_quick_select_shortcut() -> String {
    let path = get_config_dir().join("quick_select_shortcut");
    if path.exists() {
        if let Ok(s) = fs::read_to_string(&path) {
            let trimmed = s.trim().to_string();
            if !trimmed.is_empty() {
                return trimmed;
            }
        }
    }
    "CommandOrControl+Shift+Space".to_string()
}

pub fn set_quick_select_shortcut(shortcut: &str) -> Result<(), String> {
    let path = get_config_dir().join("quick_select_shortcut");
    fs::write(&path, shortcut.trim())
        .map_err(|e| format!("Failed to save Quick Select shortcut: {}", e))
}

pub fn get_llm_provider() -> String {
    let path = get_config_dir().join("llm_provider");
    if path.exists() {
        if let Ok(provider) = fs::read_to_string(&path) {
            let trimmed = provider.trim().to_string();
            if !trimmed.is_empty() {
                return trimmed;
            }
        }
    }
    "groq".to_string()
}

pub fn set_llm_provider(provider: &str) -> Result<(), String> {
    let path = get_config_dir().join("llm_provider");
    fs::write(&path, provider.trim())
        .map_err(|e| format!("Failed to save LLM provider: {}", e))
}

pub fn get_shortcuts() -> Vec<(String, String)> {
    let path = get_config_dir().join("shortcuts.json");
    if path.exists() {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(shortcuts) = serde_json::from_str::<Vec<(String, String)>>(&data) {
                return shortcuts;
            }
        }
    }
    Vec::new()
}

pub fn set_shortcuts(shortcuts: &[(String, String)]) -> Result<(), String> {
    let path = get_config_dir().join("shortcuts.json");
    let json = serde_json::to_string(shortcuts)
        .map_err(|e| format!("Failed to serialize shortcuts: {}", e))?;
    fs::write(&path, json)
        .map_err(|e| format!("Failed to save shortcuts: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_id_generation() {
        let id = get_device_id();
        assert!(id.is_ok());
        assert!(!id.unwrap().is_empty());
    }

    #[test]
    fn test_device_id_persistence() {
        let id1 = get_device_id().unwrap();
        let id2 = get_device_id().unwrap();
        assert_eq!(id1, id2);
    }
}
