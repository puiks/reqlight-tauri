use serde::{Deserialize, Serialize};

/// Response record returned from send_request command to the frontend.
/// Not persisted to disk — only used for Tauri IPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseRecord {
    pub status_code: i32,
    pub headers: Vec<HeaderPair>,
    pub body_string: Option<String>,
    pub elapsed_time: f64, // milliseconds
    pub body_size: usize,  // bytes
    pub is_json: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderPair {
    pub key: String,
    pub value: String,
}
