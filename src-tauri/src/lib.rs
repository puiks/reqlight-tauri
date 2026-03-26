mod commands;
mod models;
mod services;

use std::sync::Arc;

use commands::{curl, http, keychain, persistence};
use tokio::sync::Notify;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(http::RequestCanceller(Arc::new(Notify::new())))
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
