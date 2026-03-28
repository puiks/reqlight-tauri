use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Key-value pair used for headers, query params, form data, and environment variables.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyValuePair {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub value: String,
    #[serde(default = "default_true")]
    pub is_enabled: bool,
    #[serde(default)]
    pub is_secret: bool,
}

fn default_true() -> bool {
    true
}

impl KeyValuePair {
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            key: String::new(),
            value: String::new(),
            is_enabled: true,
            is_secret: false,
        }
    }
}

/// HTTP method enum. Serializes as UPPERCASE string ("GET", "POST", etc.)
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HttpMethod {
    #[default]
    #[serde(rename = "GET")]
    Get,
    #[serde(rename = "POST")]
    Post,
    #[serde(rename = "PUT")]
    Put,
    #[serde(rename = "PATCH")]
    Patch,
    #[serde(rename = "DELETE")]
    Delete,
    #[serde(rename = "HEAD")]
    Head,
    #[serde(rename = "OPTIONS")]
    Options,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
        }
    }
}

/// Request body types.
///
/// Serialized format (externally tagged, camelCase):
///   None         → "none"
///   Json("x")    → {"json": "x"}
///   FormData([]) → {"formData": [...]}
///   RawText("x") → {"rawText": "x"}
///   Multipart([])→ {"multipart": [...]}
///   GraphQL{..}  → {"graphql": {"query": "...", "variables": "..."}}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RequestBody {
    #[default]
    None,
    Json(String),
    FormData(Vec<KeyValuePair>),
    RawText(String),
    Multipart(Vec<MultipartField>),
    GraphQL {
        query: String,
        variables: String,
    },
}

/// A single field in a multipart form.
/// If `file_path` is Some, the field is a file upload; otherwise text.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultipartField {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub name: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub file_path: Option<String>,
    #[serde(default = "default_true")]
    pub is_enabled: bool,
}

/// A saved HTTP request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedRequest {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    #[serde(default = "default_request_name")]
    pub name: String,
    #[serde(default)]
    pub method: HttpMethod,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub query_params: Vec<KeyValuePair>,
    #[serde(default)]
    pub headers: Vec<KeyValuePair>,
    #[serde(default)]
    pub body: RequestBody,
    #[serde(default)]
    pub auth: super::auth::AuthConfig,
    #[serde(default)]
    pub sort_order: i32,
    #[serde(default = "now_iso8601")]
    pub created_at: String,
    #[serde(default = "now_iso8601")]
    pub updated_at: String,
    #[serde(default)]
    pub response_extractions: Vec<super::extraction::ExtractionRule>,
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

fn default_request_name() -> String {
    "New Request".to_string()
}

pub fn now_iso8601() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

impl Default for SavedRequest {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: default_request_name(),
            method: HttpMethod::default(),
            url: String::new(),
            query_params: vec![],
            headers: vec![],
            body: RequestBody::default(),
            auth: super::auth::AuthConfig::default(),
            sort_order: 0,
            created_at: now_iso8601(),
            updated_at: now_iso8601(),
            response_extractions: vec![],
            timeout_secs: None,
        }
    }
}
