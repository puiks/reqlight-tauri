use crate::models::AppState;
use crate::services::persistence;

/// IPC: Load persisted app state (collections, environments, history) from disk.
/// Secret values are restored from the OS keychain.
#[tauri::command]
pub fn load_state(app: tauri::AppHandle) -> Result<AppState, String> {
    persistence::load_state(&app)
}

/// IPC: Save app state to disk.
/// Secret values are stripped from the JSON file and stored in the OS keychain.
#[tauri::command]
pub fn save_state(app: tauri::AppHandle, state: AppState) -> Result<(), String> {
    persistence::save_state(&app, &state)
}
