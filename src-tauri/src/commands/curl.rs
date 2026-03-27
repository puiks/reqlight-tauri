use crate::models::{RequestEnvironment, SavedRequest};
use crate::services::{curl_exporter, curl_parser};

/// IPC: Parse a cURL command string into a `SavedRequest`.
#[tauri::command]
pub fn parse_curl(curl_string: String) -> Result<SavedRequest, String> {
    curl_parser::parse(&curl_string)
}

/// IPC: Export a `SavedRequest` as a cURL command string.
/// If an environment is provided, variables are interpolated before export.
#[tauri::command]
pub fn export_curl(
    request: SavedRequest,
    environment: Option<RequestEnvironment>,
) -> Result<String, String> {
    Ok(curl_exporter::export(&request, environment.as_ref()))
}
