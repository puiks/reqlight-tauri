use crate::models::AppState;
use crate::services::persistence;

#[tauri::command]
pub fn load_state(app: tauri::AppHandle) -> Result<AppState, String> {
    persistence::load_state(&app)
}

#[tauri::command]
pub fn save_state(app: tauri::AppHandle, state: AppState) -> Result<(), String> {
    persistence::save_state(&app, &state)
}
