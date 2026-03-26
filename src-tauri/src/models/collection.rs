use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::request::{now_iso8601, SavedRequest};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestCollection {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    #[serde(default = "default_collection_name")]
    pub name: String,
    #[serde(default)]
    pub requests: Vec<SavedRequest>,
    #[serde(default)]
    pub sort_order: i32,
    #[serde(default = "now_iso8601")]
    pub created_at: String,
}

fn default_collection_name() -> String {
    "New Collection".to_string()
}

impl Default for RequestCollection {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: default_collection_name(),
            requests: vec![],
            sort_order: 0,
            created_at: now_iso8601(),
        }
    }
}
