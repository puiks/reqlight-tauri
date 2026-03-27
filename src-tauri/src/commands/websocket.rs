use tauri::{AppHandle, Emitter, State};

use crate::services::websocket::WsManager;

#[tauri::command]
pub async fn ws_connect(
    connection_id: String,
    url: String,
    app_handle: AppHandle,
    ws_manager: State<'_, WsManager>,
) -> Result<(), String> {
    let emit = move |event| {
        let _ = app_handle.emit("ws-event", event);
    };

    ws_manager.connect(connection_id, &url, emit).await
}

#[tauri::command]
pub async fn ws_send(
    connection_id: String,
    message: String,
    ws_manager: State<'_, WsManager>,
) -> Result<(), String> {
    ws_manager.send(&connection_id, &message).await
}

#[tauri::command]
pub async fn ws_disconnect(
    connection_id: String,
    ws_manager: State<'_, WsManager>,
) -> Result<(), String> {
    ws_manager.disconnect(&connection_id).await
}
