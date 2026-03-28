use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::collection::RequestCollection;
use super::environment::RequestEnvironment;
use super::history::RequestHistoryEntry;

/// Root persisted state.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    #[serde(default)]
    pub collections: Vec<RequestCollection>,
    #[serde(default)]
    pub environments: Vec<RequestEnvironment>,
    #[serde(default)]
    pub active_environment_id: Option<Uuid>,
    #[serde(default)]
    pub last_selected_collection_id: Option<Uuid>,
    #[serde(default)]
    pub last_selected_request_id: Option<Uuid>,
    #[serde(default)]
    pub history: Vec<RequestHistoryEntry>,
    #[serde(default)]
    pub proxy_config: super::proxy::ProxyConfig,
}
