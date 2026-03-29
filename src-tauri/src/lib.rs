mod commands;
mod constants;
mod error;
pub mod models;
pub mod services;
#[cfg(test)]
mod test_utils;

use std::sync::Arc;

use commands::{
    codegen, collection_io, curl, har, http, keychain, oauth, openapi, persistence, websocket,
};
use services::websocket::WsManager;
use tokio::sync::Notify;

/// Shared HTTP client with cookie jar enabled.
/// Wrapped in Arc so it can be cloned into async contexts.
pub struct SharedHttpClient(pub Arc<reqwest::Client>);

fn init_logging() {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let log_dir = dirs_next().unwrap_or_else(std::env::temp_dir);
    let file_appender = tracing_appender::rolling::daily(log_dir, "reqlight.log");

    fmt()
        .with_env_filter(filter)
        .with_writer(file_appender)
        .with_ansi(false)
        .init();
}

/// Returns the platform-specific log directory for Reqlight.
fn dirs_next() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir().map(|h| h.join(format!("Library/Logs/{}", constants::APP_NAME)))
    }
    #[cfg(target_os = "windows")]
    {
        dirs::data_local_dir().map(|d| d.join(format!("{}/logs", constants::APP_NAME)))
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        dirs::data_dir().map(|d| d.join(format!("{}/logs", constants::APP_NAME.to_lowercase())))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logging();

    tracing::info!("Reqlight starting up");

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
            // Code Generation
            codegen::generate_code,
            // Collection I/O
            collection_io::import_postman_collection,
            collection_io::export_postman_collection,
            collection_io::import_postman_environment,
            collection_io::export_postman_environment,
            // OpenAPI Import
            openapi::import_openapi,
            // HAR Import
            har::import_har,
            // OAuth
            oauth::oauth_client_credentials,
            oauth::oauth_authorization_code,
            oauth::oauth_refresh_token,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
