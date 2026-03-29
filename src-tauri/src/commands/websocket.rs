use crate::models::{KeyValuePair, RequestEnvironment};
use crate::services::interpolator;
use crate::services::websocket::WsManager;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub async fn ws_connect(
    connection_id: String,
    url: String,
    headers: Option<Vec<KeyValuePair>>,
    environment: Option<RequestEnvironment>,
    ws_manager: State<'_, WsManager>,
    app_handle: AppHandle,
) -> Result<(), String> {
    // Interpolate URL and headers with environment variables
    let (final_url, final_headers) = if let Some(env) = &environment {
        let u = interpolator::interpolate(&url, &env.variables);
        let h = headers
            .unwrap_or_default()
            .iter()
            .filter(|h| h.is_enabled && !h.key.is_empty())
            .map(|h| {
                (
                    interpolator::interpolate(&h.key, &env.variables),
                    interpolator::interpolate(&h.value, &env.variables),
                )
            })
            .collect::<Vec<_>>();
        (u, h)
    } else {
        let h = headers
            .unwrap_or_default()
            .iter()
            .filter(|h| h.is_enabled && !h.key.is_empty())
            .map(|h| (h.key.clone(), h.value.clone()))
            .collect::<Vec<_>>();
        (url, h)
    };

    let emit = move |event| {
        let _ = app_handle.emit("ws-event", event);
    };

    ws_manager
        .connect(connection_id, &final_url, &final_headers, emit)
        .await
        .map_err(String::from)
}

#[tauri::command]
pub async fn ws_send(
    connection_id: String,
    message: String,
    ws_manager: State<'_, WsManager>,
) -> Result<(), String> {
    ws_manager
        .send(&connection_id, &message)
        .await
        .map_err(String::from)
}

#[tauri::command]
pub async fn ws_disconnect(
    connection_id: String,
    ws_manager: State<'_, WsManager>,
) -> Result<(), String> {
    ws_manager
        .disconnect(&connection_id)
        .await
        .map_err(String::from)
}
