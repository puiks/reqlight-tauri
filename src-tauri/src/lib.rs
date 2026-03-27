mod commands;
mod models;
mod services;
#[cfg(test)]
mod test_utils;

use std::sync::Arc;

use commands::{collection_io, curl, http, keychain, persistence, websocket};
use services::websocket::WsManager;
use tokio::sync::Notify;

/// Shared HTTP client with cookie jar enabled.
/// Wrapped in Arc so it can be cloned into async contexts.
pub struct SharedHttpClient(pub Arc<reqwest::Client>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let http_client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .expect("Failed to create HTTP client");

    tauri::Builder::default()
        .manage(SharedHttpClient(Arc::new(http_client)))
        .manage(http::RequestCanceller(Arc::new(Notify::new())))
        .manage(WsManager::new())
        .invoke_handler(tauri::generate_handler![
            // HTTP
            http::send_request,
            http::cancel_request,
            // Persistence
            persistence::load_state,
            persistence::save_state,
            // Keychain
            keychain::secret_set,
            keychain::secret_get,
            keychain::secret_delete,
            // cURL
            curl::parse_curl,
            curl::export_curl,
            // WebSocket
            websocket::ws_connect,
            websocket::ws_send,
            websocket::ws_disconnect,
            // Collection I/O
            collection_io::import_postman_collection,
            collection_io::export_postman_collection,
            collection_io::import_postman_environment,
            collection_io::export_postman_environment,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
