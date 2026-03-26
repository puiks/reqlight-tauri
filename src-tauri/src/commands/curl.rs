use crate::models::{RequestEnvironment, SavedRequest};
use crate::services::{curl_exporter, curl_parser};

#[tauri::command]
pub fn parse_curl(curl_string: String) -> Result<SavedRequest, String> {
    curl_parser::parse(&curl_string)
}

#[tauri::command]
pub fn export_curl(
    request: SavedRequest,
    environment: Option<RequestEnvironment>,
) -> Result<String, String> {
    Ok(curl_exporter::export(&request, environment.as_ref()))
}
