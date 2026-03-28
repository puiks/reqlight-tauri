use crate::models::{RequestEnvironment, SavedRequest};
use crate::services::code_generator;

#[tauri::command]
pub fn generate_code(
    request: SavedRequest,
    environment: Option<RequestEnvironment>,
    language: String,
) -> Result<String, String> {
    code_generator::generate(&request, environment.as_ref(), &language)
}
