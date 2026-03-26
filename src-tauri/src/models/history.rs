use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::request::{now_iso8601, HttpMethod};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestHistoryEntry {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub method: HttpMethod,
    pub url: String,
    #[serde(default)]
    pub status_code: Option<i32>,
    #[serde(default = "now_iso8601")]
    pub timestamp: String,
    #[serde(default)]
    pub elapsed_time: Option<f64>,
}
