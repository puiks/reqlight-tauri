use crate::services::keychain;

/// IPC: Store a secret value in the OS credential store.
#[tauri::command]
pub fn secret_set(key: String, value: String) -> Result<(), String> {
    keychain::save(&key, &value)
}

/// IPC: Retrieve a secret value from the OS credential store.
/// Returns `None` if the key does not exist.
#[tauri::command]
pub fn secret_get(key: String) -> Result<Option<String>, String> {
    keychain::load(&key)
}

/// IPC: Delete a secret value from the OS credential store.
/// No-op if the key does not exist.
#[tauri::command]
pub fn secret_delete(key: String) -> Result<(), String> {
    keychain::delete(&key)
}
