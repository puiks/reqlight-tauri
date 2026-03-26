use std::fs;
use std::path::PathBuf;

use crate::models::AppState;
use crate::services::keychain;

const DATA_FILE_NAME: &str = "data.json";

/// Get the data directory path.
/// On macOS: ~/Library/Application Support/Reqlight/
/// (Matching the original Swift app's path for data compatibility)
fn data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    // Use Tauri's app_data_dir which respects the identifier in tauri.conf.json
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {e}"))?;
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create data dir: {e}"))?;
    Ok(dir)
}

fn data_file_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(data_dir(app)?.join(DATA_FILE_NAME))
}

/// Keychain key format matching Swift: "env_{envId}_{varId}"
fn keychain_key(env_id: &uuid::Uuid, var_id: &uuid::Uuid) -> String {
    // Swift uses uppercased UUID strings
    format!(
        "env_{}_{}",
        env_id.to_string().to_uppercase(),
        var_id.to_string().to_uppercase()
    )
}

/// Load app state from disk. Restores secret values from keychain.
pub fn load_state(app: &tauri::AppHandle) -> Result<AppState, String> {
    let path = data_file_path(app)?;

    if !path.exists() {
        return Ok(AppState::default());
    }

    let data = fs::read_to_string(&path).map_err(|e| format!("Failed to read data file: {e}"))?;
    let mut state: AppState =
        serde_json::from_str(&data).map_err(|e| format!("Failed to parse data file: {e}"))?;

    // Restore secret values from keychain
    for env in &mut state.environments {
        for var in &mut env.variables {
            if var.is_secret {
                let key = keychain_key(&env.id, &var.id);
                if let Ok(Some(secret)) = keychain::load(&key) {
                    var.value = secret;
                }
            }
        }
    }

    Ok(state)
}

/// Save app state to disk. Strips secret values and stores them in keychain.
pub fn save_state(app: &tauri::AppHandle, state: &AppState) -> Result<(), String> {
    let path = data_file_path(app)?;

    // Clone and sanitize: store secrets in keychain, strip from JSON
    let mut sanitized = state.clone();
    for env in &mut sanitized.environments {
        for var in &mut env.variables {
            if var.is_secret && !var.value.is_empty() {
                let key = keychain_key(&env.id, &var.id);
                keychain::save(&key, &var.value)?;
                var.value = String::new();
            }
        }
    }

    let json = serde_json::to_string_pretty(&sanitized)
        .map_err(|e| format!("Failed to serialize state: {e}"))?;
    fs::write(&path, json).map_err(|e| format!("Failed to write data file: {e}"))?;

    Ok(())
}

use tauri::Manager;
