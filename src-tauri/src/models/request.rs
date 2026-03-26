use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

/// Key-value pair used for headers, query params, form data, and environment variables.
/// Compatible with Swift's KeyValuePair Codable format.
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

/// HTTP method enum. Serializes as raw string ("GET", "POST", etc.)
/// to match Swift's `HTTPMethod: String, Codable` with rawValue.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum HttpMethod {
    #[default]
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl Serialize for HttpMethod {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for HttpMethod {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "PATCH" => Ok(HttpMethod::Patch),
            "DELETE" => Ok(HttpMethod::Delete),
            _ => Err(serde::de::Error::custom(format!(
                "Unknown HTTP method: {s}"
            ))),
        }
    }
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Delete => "DELETE",
        }
    }
}

/// Request body types.
///
/// Swift auto-synthesized Codable for enums with associated values encodes as:
///   .none        → {"none": {}}
///   .json("x")   → {"json": {"_0": "x"}}
///   .formData([]) → {"formData": {"_0": [...]}}
///   .rawText("x") → {"rawText": {"_0": "x"}}
///
/// We use a custom serde implementation to match this format.
#[derive(Debug, Clone, Default)]
pub enum RequestBody {
    #[default]
    None,
    Json(String),
    FormData(Vec<KeyValuePair>),
    RawText(String),
}

// Custom Serialize to match Swift's auto-synthesized Codable format
impl Serialize for RequestBody {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(1))?;
        match self {
            RequestBody::None => {
                map.serialize_entry("none", &serde_json::json!({}))?;
            }
            RequestBody::Json(s) => {
                map.serialize_entry("json", &serde_json::json!({"_0": s}))?;
            }
            RequestBody::FormData(pairs) => {
                map.serialize_entry("formData", &serde_json::json!({"_0": pairs}))?;
            }
            RequestBody::RawText(s) => {
                map.serialize_entry("rawText", &serde_json::json!({"_0": s}))?;
            }
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for RequestBody {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        let obj = value
            .as_object()
            .ok_or_else(|| serde::de::Error::custom("RequestBody must be a JSON object"))?;

        if obj.contains_key("none") {
            Ok(RequestBody::None)
        } else if let Some(inner) = obj.get("json") {
            let s = inner
                .get("_0")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            Ok(RequestBody::Json(s))
        } else if let Some(inner) = obj.get("formData") {
            let pairs: Vec<KeyValuePair> = inner
                .get("_0")
                .map(|v| serde_json::from_value(v.clone()).unwrap_or_default())
                .unwrap_or_default();
            Ok(RequestBody::FormData(pairs))
        } else if let Some(inner) = obj.get("rawText") {
            let s = inner
                .get("_0")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            Ok(RequestBody::RawText(s))
        } else {
            Ok(RequestBody::None)
        }
    }
}

/// A saved HTTP request. Matches Swift's SavedRequest Codable format.
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
    pub sort_order: i32,
    #[serde(default = "now_iso8601")]
    pub created_at: String,
    #[serde(default = "now_iso8601")]
    pub updated_at: String,
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
            sort_order: 0,
            created_at: now_iso8601(),
            updated_at: now_iso8601(),
        }
    }
}
