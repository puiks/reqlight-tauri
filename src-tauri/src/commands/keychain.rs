use crate::services::keychain;

#[tauri::command]
pub fn secret_set(key: String, value: String) -> Result<(), String> {
    keychain::save(&key, &value)
}

#[tauri::command]
pub fn secret_get(key: String) -> Result<Option<String>, String> {
    keychain::load(&key)
}

#[tauri::command]
pub fn secret_delete(key: String) -> Result<(), String> {
    keychain::delete(&key)
}
