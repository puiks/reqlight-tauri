use std::fs;
use std::path::{Path, PathBuf};

use crate::constants::DATA_FILE_NAME;
use crate::models::auth::AuthConfig;
use crate::models::AppState;
use crate::services::keychain;

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

/// Keychain key format: "env_{envId}_{varId}"
fn keychain_key(env_id: &uuid::Uuid, var_id: &uuid::Uuid) -> String {
    format!(
        "env_{}_{}",
        env_id.to_string().to_uppercase(),
        var_id.to_string().to_uppercase()
    )
}

/// Keychain key for OAuth2 secrets on a specific request.
/// Format: "oauth_{collectionId}_{requestId}_{field}"
fn oauth_keychain_key(collection_id: &uuid::Uuid, request_id: &uuid::Uuid, field: &str) -> String {
    format!(
        "oauth_{}_{}_{}",
        collection_id.to_string().to_uppercase(),
        request_id.to_string().to_uppercase(),
        field,
    )
}

/// Load app state JSON from a file path. Returns default if file doesn't exist.
/// Does NOT restore secrets from keychain (caller is responsible for that).
fn load_state_from_path(path: &Path) -> Result<AppState, String> {
    if !path.exists() {
        return Ok(AppState::default());
    }

    let data = fs::read_to_string(path).map_err(|e| format!("Failed to read data file: {e}"))?;
    serde_json::from_str(&data).map_err(|e| format!("Failed to parse data file: {e}"))
}

/// Sanitize state for writing: strip secret values and return them as (key, value) pairs.
/// The returned state has all secret values cleared.
fn sanitize_secrets(state: &AppState) -> (AppState, Vec<(String, String)>) {
    let mut sanitized = state.clone();
    let mut secrets = Vec::new();

    // Strip secret environment variables
    for env in &mut sanitized.environments {
        for var in &mut env.variables {
            if var.is_secret && !var.value.is_empty() {
                let key = keychain_key(&env.id, &var.id);
                secrets.push((key, var.value.clone()));
                var.value = String::new();
            }
        }
    }

    // Strip OAuth2 tokens from request auth configs
    for collection in &mut sanitized.collections {
        for request in &mut collection.requests {
            if let AuthConfig::OAuth2 {
                ref mut access_token,
                ref mut refresh_token,
                ref mut client_secret,
                ..
            } = request.auth
            {
                let cid = &collection.id;
                let rid = &request.id;
                if !access_token.is_empty() {
                    let key = oauth_keychain_key(cid, rid, "access_token");
                    secrets.push((key, access_token.clone()));
                    *access_token = String::new();
                }
                if !refresh_token.is_empty() {
                    let key = oauth_keychain_key(cid, rid, "refresh_token");
                    secrets.push((key, refresh_token.clone()));
                    *refresh_token = String::new();
                }
                if !client_secret.is_empty() {
                    let key = oauth_keychain_key(cid, rid, "client_secret");
                    secrets.push((key, client_secret.clone()));
                    *client_secret = String::new();
                }
            }
        }
    }

    (sanitized, secrets)
}

/// Serialize state to JSON and write to a file path.
fn save_state_to_path(path: &Path, state: &AppState) -> Result<(), String> {
    let json = serde_json::to_string_pretty(state)
        .map_err(|e| format!("Failed to serialize state: {e}"))?;
    fs::write(path, json).map_err(|e| format!("Failed to write data file: {e}"))
}

/// Load app state from disk. Restores secret values from keychain.
pub fn load_state(app: &tauri::AppHandle) -> Result<AppState, String> {
    let path = data_file_path(app)?;
    tracing::info!(path = %path.display(), "Loading app state");
    let mut state = load_state_from_path(&path)?;

    // Restore secret environment variables from keychain
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

    // Restore OAuth2 tokens from keychain
    for collection in &mut state.collections {
        for request in &mut collection.requests {
            if let AuthConfig::OAuth2 {
                ref mut access_token,
                ref mut refresh_token,
                ref mut client_secret,
                ..
            } = request.auth
            {
                let cid = &collection.id;
                let rid = &request.id;
                if let Ok(Some(v)) = keychain::load(&oauth_keychain_key(cid, rid, "access_token")) {
                    *access_token = v;
                }
                if let Ok(Some(v)) = keychain::load(&oauth_keychain_key(cid, rid, "refresh_token"))
                {
                    *refresh_token = v;
                }
                if let Ok(Some(v)) = keychain::load(&oauth_keychain_key(cid, rid, "client_secret"))
                {
                    *client_secret = v;
                }
            }
        }
    }

    Ok(state)
}

/// Save app state to disk. Strips secret values and stores them in keychain.
pub fn save_state(app: &tauri::AppHandle, state: &AppState) -> Result<(), String> {
    let path = data_file_path(app)?;
    tracing::debug!(path = %path.display(), "Saving app state");

    let (sanitized, secrets) = sanitize_secrets(state);

    // Store secrets in keychain
    for (key, value) in secrets {
        keychain::save(&key, &value)?;
    }

    save_state_to_path(&path, &sanitized)
}

use tauri::Manager;

#[cfg(test)]
#[path = "persistence_tests.rs"]
mod tests;
