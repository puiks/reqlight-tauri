use std::fs;
use std::path::{Path, PathBuf};

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

    for env in &mut sanitized.environments {
        for var in &mut env.variables {
            if var.is_secret && !var.value.is_empty() {
                let key = keychain_key(&env.id, &var.id);
                secrets.push((key, var.value.clone()));
                var.value = String::new();
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
    let mut state = load_state_from_path(&path)?;

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

    let (sanitized, secrets) = sanitize_secrets(state);

    // Store secrets in keychain
    for (key, value) in secrets {
        keychain::save(&key, &value)?;
    }

    save_state_to_path(&path, &sanitized)
}

use tauri::Manager;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::environment::RequestEnvironment;
    use crate::models::request::KeyValuePair;

    #[test]
    fn keychain_key_format_uses_uppercased_uuids() {
        let env_id = uuid::Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap();
        let var_id = uuid::Uuid::parse_str("11111111-2222-3333-4444-555555555555").unwrap();

        let key = keychain_key(&env_id, &var_id);

        assert_eq!(
            key,
            "env_A1B2C3D4-E5F6-7890-ABCD-EF1234567890_11111111-2222-3333-4444-555555555555"
        );
    }

    #[test]
    fn load_from_nonexistent_file_returns_default() {
        let path = Path::new("/tmp/reqlight-test-nonexistent/data.json");
        let state = load_state_from_path(path).unwrap();

        assert!(state.collections.is_empty());
        assert!(state.environments.is_empty());
        assert!(state.history.is_empty());
        assert!(state.active_environment_id.is_none());
    }

    #[test]
    fn save_and_load_round_trip_preserves_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data.json");

        let mut state = AppState::default();
        state.environments.push(RequestEnvironment {
            id: uuid::Uuid::new_v4(),
            name: "Production".to_string(),
            variables: vec![KeyValuePair {
                id: uuid::Uuid::new_v4(),
                key: "BASE_URL".to_string(),
                value: "https://api.example.com".to_string(),
                is_enabled: true,
                is_secret: false,
            }],
        });

        save_state_to_path(&path, &state).unwrap();
        let loaded = load_state_from_path(&path).unwrap();

        assert_eq!(loaded.environments.len(), 1);
        assert_eq!(loaded.environments[0].name, "Production");
        assert_eq!(loaded.environments[0].variables.len(), 1);
        assert_eq!(loaded.environments[0].variables[0].key, "BASE_URL");
        assert_eq!(
            loaded.environments[0].variables[0].value,
            "https://api.example.com"
        );
    }

    #[test]
    fn sanitize_secrets_strips_secret_values() {
        let env_id = uuid::Uuid::new_v4();
        let secret_var_id = uuid::Uuid::new_v4();
        let normal_var_id = uuid::Uuid::new_v4();

        let mut state = AppState::default();
        state.environments.push(RequestEnvironment {
            id: env_id,
            name: "Test".to_string(),
            variables: vec![
                KeyValuePair {
                    id: secret_var_id,
                    key: "API_KEY".to_string(),
                    value: "super-secret-123".to_string(),
                    is_enabled: true,
                    is_secret: true,
                },
                KeyValuePair {
                    id: normal_var_id,
                    key: "HOST".to_string(),
                    value: "localhost".to_string(),
                    is_enabled: true,
                    is_secret: false,
                },
            ],
        });

        let (sanitized, secrets) = sanitize_secrets(&state);

        // Secret value should be stripped
        assert_eq!(sanitized.environments[0].variables[0].value, "");
        // Normal value should be preserved
        assert_eq!(sanitized.environments[0].variables[1].value, "localhost");
        // Secret should be returned for keychain storage
        assert_eq!(secrets.len(), 1);
        assert_eq!(secrets[0].1, "super-secret-123");
        assert!(secrets[0].0.contains(&env_id.to_string().to_uppercase()));
    }

    #[test]
    fn secret_values_not_written_to_json_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data.json");

        let mut state = AppState::default();
        state.environments.push(RequestEnvironment {
            id: uuid::Uuid::new_v4(),
            name: "Secrets Test".to_string(),
            variables: vec![KeyValuePair {
                id: uuid::Uuid::new_v4(),
                key: "TOKEN".to_string(),
                value: "my-secret-token".to_string(),
                is_enabled: true,
                is_secret: true,
            }],
        });

        // Sanitize and save (simulating what save_state does, minus keychain)
        let (sanitized, _secrets) = sanitize_secrets(&state);
        save_state_to_path(&path, &sanitized).unwrap();

        // Read the raw JSON and verify the secret is not present
        let raw_json = fs::read_to_string(&path).unwrap();
        assert!(
            !raw_json.contains("my-secret-token"),
            "Secret value should NOT appear in the JSON file"
        );
        // The key name should still be there
        assert!(raw_json.contains("TOKEN"));
    }

    #[test]
    fn sanitize_secrets_ignores_empty_secret_values() {
        let mut state = AppState::default();
        state.environments.push(RequestEnvironment {
            id: uuid::Uuid::new_v4(),
            name: "Test".to_string(),
            variables: vec![KeyValuePair {
                id: uuid::Uuid::new_v4(),
                key: "EMPTY_SECRET".to_string(),
                value: String::new(),
                is_enabled: true,
                is_secret: true,
            }],
        });

        let (_sanitized, secrets) = sanitize_secrets(&state);

        // Empty secret should not be stored in keychain
        assert!(secrets.is_empty());
    }

    #[test]
    fn load_from_invalid_json_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data.json");
        fs::write(&path, "not valid json{{{").unwrap();

        let result = load_state_from_path(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse data file"));
    }
}
