use serde::{Deserialize, Serialize};

// ====================== Postman Collection v2.1 structures ======================

#[derive(Serialize, Deserialize)]
pub(crate) struct PostmanCollection {
    pub info: PostmanInfo,
    #[serde(default)]
    pub item: Vec<PostmanItem>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct PostmanInfo {
    pub name: String,
    #[serde(default)]
    pub schema: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct PostmanItem {
    pub name: String,
    #[serde(default)]
    pub request: Option<PostmanRequest>,
    /// Sub-items (folders)
    #[serde(default)]
    pub item: Vec<PostmanItem>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct PostmanRequest {
    #[serde(default)]
    pub method: String,
    pub url: serde_json::Value,
    #[serde(default)]
    pub header: Vec<PostmanHeader>,
    #[serde(default)]
    pub body: Option<PostmanBody>,
    #[serde(default)]
    pub auth: Option<PostmanAuth>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct PostmanHeader {
    pub key: String,
    pub value: String,
    #[serde(default = "default_true")]
    pub disabled: bool,
}

fn default_true() -> bool {
    false
}

#[derive(Serialize, Deserialize)]
pub(crate) struct PostmanBody {
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub raw: Option<String>,
    #[serde(default)]
    pub urlencoded: Option<Vec<PostmanUrlEncoded>>,
    #[serde(default)]
    pub options: Option<PostmanBodyOptions>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct PostmanBodyOptions {
    #[serde(default)]
    pub raw: Option<PostmanRawOptions>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct PostmanRawOptions {
    #[serde(default)]
    pub language: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct PostmanUrlEncoded {
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct PostmanAuth {
    #[serde(rename = "type")]
    pub auth_type: String,
    #[serde(default)]
    pub bearer: Option<Vec<PostmanKV>>,
    #[serde(default)]
    pub basic: Option<Vec<PostmanKV>>,
    #[serde(default)]
    pub apikey: Option<Vec<PostmanKV>>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct PostmanKV {
    pub key: String,
    pub value: serde_json::Value,
}

// ====================== Postman Environment ======================

#[derive(Serialize, Deserialize)]
pub(crate) struct PostmanEnvironment {
    pub name: String,
    #[serde(default)]
    pub values: Vec<PostmanEnvValue>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct PostmanEnvValue {
    pub key: String,
    pub value: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}
