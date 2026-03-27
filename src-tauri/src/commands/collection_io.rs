use crate::models::collection::RequestCollection;
use crate::models::RequestEnvironment;
use crate::services::collection_io;

/// IPC: Import a Postman Collection JSON string and return our RequestCollection.
#[tauri::command]
pub fn import_postman_collection(json_str: String) -> Result<RequestCollection, String> {
    collection_io::import_postman_collection(&json_str)
}

/// IPC: Export a RequestCollection as Postman Collection v2.1 JSON string.
#[tauri::command]
pub fn export_postman_collection(collection: RequestCollection) -> Result<String, String> {
    collection_io::export_postman_collection(&collection)
}

/// IPC: Import a Postman Environment JSON string and return our RequestEnvironment.
#[tauri::command]
pub fn import_postman_environment(json_str: String) -> Result<RequestEnvironment, String> {
    collection_io::import_postman_environment(&json_str)
}

/// IPC: Export a RequestEnvironment as Postman Environment JSON string.
#[tauri::command]
pub fn export_postman_environment(environment: RequestEnvironment) -> Result<String, String> {
    collection_io::export_postman_environment(&environment)
}
