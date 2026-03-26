use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::request::KeyValuePair;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestEnvironment {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    #[serde(default = "default_env_name")]
    pub name: String,
    #[serde(default)]
    pub variables: Vec<KeyValuePair>,
}

fn default_env_name() -> String {
    "New Environment".to_string()
}

impl Default for RequestEnvironment {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: default_env_name(),
            variables: vec![],
        }
    }
}
