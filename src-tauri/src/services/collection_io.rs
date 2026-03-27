use crate::models::collection::RequestCollection;
use crate::models::{
    AuthConfig, HttpMethod, KeyValuePair, RequestBody, RequestEnvironment, SavedRequest,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ====================== Postman Collection v2.1 structures ======================

#[derive(Serialize, Deserialize)]
struct PostmanCollection {
    info: PostmanInfo,
    #[serde(default)]
    item: Vec<PostmanItem>,
}

#[derive(Serialize, Deserialize)]
struct PostmanInfo {
    name: String,
    #[serde(default)]
    schema: String,
}

#[derive(Serialize, Deserialize)]
struct PostmanItem {
    name: String,
    #[serde(default)]
    request: Option<PostmanRequest>,
    /// Sub-items (folders)
    #[serde(default)]
    item: Vec<PostmanItem>,
}

#[derive(Serialize, Deserialize)]
struct PostmanRequest {
    #[serde(default)]
    method: String,
    url: serde_json::Value,
    #[serde(default)]
    header: Vec<PostmanHeader>,
    #[serde(default)]
    body: Option<PostmanBody>,
    #[serde(default)]
    auth: Option<PostmanAuth>,
}

#[derive(Serialize, Deserialize)]
struct PostmanHeader {
    key: String,
    value: String,
    #[serde(default = "default_true")]
    disabled: bool,
}

fn default_true() -> bool {
    false
}

#[derive(Serialize, Deserialize)]
struct PostmanBody {
    #[serde(default)]
    mode: String,
    #[serde(default)]
    raw: Option<String>,
    #[serde(default)]
    urlencoded: Option<Vec<PostmanUrlEncoded>>,
    #[serde(default)]
    options: Option<PostmanBodyOptions>,
}

#[derive(Serialize, Deserialize)]
struct PostmanBodyOptions {
    #[serde(default)]
    raw: Option<PostmanRawOptions>,
}

#[derive(Serialize, Deserialize)]
struct PostmanRawOptions {
    #[serde(default)]
    language: String,
}

#[derive(Serialize, Deserialize)]
struct PostmanUrlEncoded {
    key: String,
    value: String,
    #[serde(default)]
    disabled: bool,
}

#[derive(Serialize, Deserialize)]
struct PostmanAuth {
    #[serde(rename = "type")]
    auth_type: String,
    #[serde(default)]
    bearer: Option<Vec<PostmanKV>>,
    #[serde(default)]
    basic: Option<Vec<PostmanKV>>,
    #[serde(default)]
    apikey: Option<Vec<PostmanKV>>,
}

#[derive(Serialize, Deserialize)]
struct PostmanKV {
    key: String,
    value: serde_json::Value,
}

// ====================== Postman Environment ======================

#[derive(Serialize, Deserialize)]
struct PostmanEnvironment {
    name: String,
    #[serde(default)]
    values: Vec<PostmanEnvValue>,
}

#[derive(Serialize, Deserialize)]
struct PostmanEnvValue {
    key: String,
    value: String,
    #[serde(default = "default_enabled")]
    enabled: bool,
}

fn default_enabled() -> bool {
    true
}

// ====================== Import ======================

/// Import a Postman Collection v2.1 JSON string into our RequestCollection.
pub fn import_postman_collection(json_str: &str) -> Result<RequestCollection, String> {
    let postman: PostmanCollection =
        serde_json::from_str(json_str).map_err(|e| format!("Invalid Postman collection: {e}"))?;

    let mut requests = Vec::new();
    flatten_items(&postman.item, &mut requests, 0);

    Ok(RequestCollection {
        id: Uuid::new_v4(),
        name: postman.info.name,
        requests,
        sort_order: 0,
        created_at: crate::models::request::now_iso8601(),
    })
}

fn flatten_items(items: &[PostmanItem], requests: &mut Vec<SavedRequest>, base_order: i32) {
    for (i, item) in items.iter().enumerate() {
        if let Some(ref req) = item.request {
            requests.push(convert_request(&item.name, req, base_order + i as i32));
        }
        // Recurse into folders
        if !item.item.is_empty() {
            flatten_items(&item.item, requests, base_order + requests.len() as i32);
        }
    }
}

fn convert_request(name: &str, req: &PostmanRequest, sort_order: i32) -> SavedRequest {
    let method = match req.method.to_uppercase().as_str() {
        "POST" => HttpMethod::Post,
        "PUT" => HttpMethod::Put,
        "PATCH" => HttpMethod::Patch,
        "DELETE" => HttpMethod::Delete,
        _ => HttpMethod::Get,
    };

    let url = extract_url(&req.url);

    let headers: Vec<KeyValuePair> = req
        .header
        .iter()
        .map(|h| KeyValuePair {
            id: Uuid::new_v4(),
            key: h.key.clone(),
            value: h.value.clone(),
            is_enabled: !h.disabled,
            is_secret: false,
        })
        .collect();

    let body = req
        .body
        .as_ref()
        .map(convert_body)
        .unwrap_or(RequestBody::None);

    let auth = req
        .auth
        .as_ref()
        .map(convert_auth)
        .unwrap_or(AuthConfig::None);

    let now = crate::models::request::now_iso8601();
    SavedRequest {
        id: Uuid::new_v4(),
        name: name.to_string(),
        method,
        url,
        query_params: vec![],
        headers,
        body,
        auth,
        sort_order,
        created_at: now.clone(),
        updated_at: now,
    }
}

fn extract_url(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Object(obj) => obj
            .get("raw")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        _ => String::new(),
    }
}

fn convert_body(body: &PostmanBody) -> RequestBody {
    match body.mode.as_str() {
        "raw" => {
            let text = body.raw.clone().unwrap_or_default();
            let is_json = body
                .options
                .as_ref()
                .and_then(|o| o.raw.as_ref())
                .map(|r| r.language == "json")
                .unwrap_or(false);
            if is_json {
                RequestBody::Json(text)
            } else {
                RequestBody::RawText(text)
            }
        }
        "urlencoded" => {
            let pairs = body
                .urlencoded
                .as_ref()
                .map(|pairs| {
                    pairs
                        .iter()
                        .map(|p| KeyValuePair {
                            id: Uuid::new_v4(),
                            key: p.key.clone(),
                            value: p.value.clone(),
                            is_enabled: !p.disabled,
                            is_secret: false,
                        })
                        .collect()
                })
                .unwrap_or_default();
            RequestBody::FormData(pairs)
        }
        _ => RequestBody::None,
    }
}

fn convert_auth(auth: &PostmanAuth) -> AuthConfig {
    match auth.auth_type.as_str() {
        "bearer" => {
            let token = auth
                .bearer
                .as_ref()
                .and_then(|kvs| kvs.iter().find(|kv| kv.key == "token"))
                .and_then(|kv| kv.value.as_str())
                .unwrap_or("")
                .to_string();
            AuthConfig::BearerToken { token }
        }
        "basic" => {
            let get_val = |key: &str| -> String {
                auth.basic
                    .as_ref()
                    .and_then(|kvs| kvs.iter().find(|kv| kv.key == key))
                    .and_then(|kv| kv.value.as_str())
                    .unwrap_or("")
                    .to_string()
            };
            AuthConfig::BasicAuth {
                username: get_val("username"),
                password: get_val("password"),
            }
        }
        _ => AuthConfig::None,
    }
}

// ====================== Export ======================

/// Export our RequestCollection as a Postman Collection v2.1 JSON string.
pub fn export_postman_collection(collection: &RequestCollection) -> Result<String, String> {
    let postman = PostmanCollection {
        info: PostmanInfo {
            name: collection.name.clone(),
            schema: "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
                .to_string(),
        },
        item: collection
            .requests
            .iter()
            .map(|r| PostmanItem {
                name: r.name.clone(),
                request: Some(to_postman_request(r)),
                item: vec![],
            })
            .collect(),
    };

    serde_json::to_string_pretty(&postman).map_err(|e| format!("Serialization error: {e}"))
}

fn to_postman_request(req: &SavedRequest) -> PostmanRequest {
    PostmanRequest {
        method: req.method.as_str().to_string(),
        url: serde_json::Value::String(req.url.clone()),
        header: req
            .headers
            .iter()
            .map(|h| PostmanHeader {
                key: h.key.clone(),
                value: h.value.clone(),
                disabled: !h.is_enabled,
            })
            .collect(),
        body: to_postman_body(&req.body),
        auth: to_postman_auth(&req.auth),
    }
}

fn to_postman_body(body: &RequestBody) -> Option<PostmanBody> {
    match body {
        RequestBody::Json(text) => Some(PostmanBody {
            mode: "raw".to_string(),
            raw: Some(text.clone()),
            urlencoded: None,
            options: Some(PostmanBodyOptions {
                raw: Some(PostmanRawOptions {
                    language: "json".to_string(),
                }),
            }),
        }),
        RequestBody::RawText(text) => Some(PostmanBody {
            mode: "raw".to_string(),
            raw: Some(text.clone()),
            urlencoded: None,
            options: None,
        }),
        RequestBody::FormData(pairs) => Some(PostmanBody {
            mode: "urlencoded".to_string(),
            raw: None,
            urlencoded: Some(
                pairs
                    .iter()
                    .map(|p| PostmanUrlEncoded {
                        key: p.key.clone(),
                        value: p.value.clone(),
                        disabled: !p.is_enabled,
                    })
                    .collect(),
            ),
            options: None,
        }),
        _ => None,
    }
}

fn to_postman_auth(auth: &AuthConfig) -> Option<PostmanAuth> {
    match auth {
        AuthConfig::BearerToken { token } => Some(PostmanAuth {
            auth_type: "bearer".to_string(),
            bearer: Some(vec![PostmanKV {
                key: "token".to_string(),
                value: serde_json::Value::String(token.clone()),
            }]),
            basic: None,
            apikey: None,
        }),
        AuthConfig::BasicAuth { username, password } => Some(PostmanAuth {
            auth_type: "basic".to_string(),
            bearer: None,
            basic: Some(vec![
                PostmanKV {
                    key: "username".to_string(),
                    value: serde_json::Value::String(username.clone()),
                },
                PostmanKV {
                    key: "password".to_string(),
                    value: serde_json::Value::String(password.clone()),
                },
            ]),
            apikey: None,
        }),
        _ => None,
    }
}

/// Import a Postman Environment JSON string.
pub fn import_postman_environment(json_str: &str) -> Result<RequestEnvironment, String> {
    let env: PostmanEnvironment =
        serde_json::from_str(json_str).map_err(|e| format!("Invalid Postman environment: {e}"))?;

    Ok(RequestEnvironment {
        id: Uuid::new_v4(),
        name: env.name,
        variables: env
            .values
            .into_iter()
            .map(|v| KeyValuePair {
                id: Uuid::new_v4(),
                key: v.key,
                value: v.value,
                is_enabled: v.enabled,
                is_secret: false,
            })
            .collect(),
    })
}

/// Export our RequestEnvironment as Postman Environment JSON.
pub fn export_postman_environment(env: &RequestEnvironment) -> Result<String, String> {
    let postman_env = PostmanEnvironment {
        name: env.name.clone(),
        values: env
            .variables
            .iter()
            .map(|v| PostmanEnvValue {
                key: v.key.clone(),
                value: v.value.clone(),
                enabled: v.is_enabled,
            })
            .collect(),
    };

    serde_json::to_string_pretty(&postman_env).map_err(|e| format!("Serialization error: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_simple_postman_collection() {
        let json = r#"{
            "info": { "name": "Test API", "schema": "" },
            "item": [
                {
                    "name": "Get Users",
                    "request": {
                        "method": "GET",
                        "url": "https://api.example.com/users",
                        "header": [],
                        "body": null
                    }
                }
            ]
        }"#;
        let result = import_postman_collection(json).unwrap();
        assert_eq!(result.name, "Test API");
        assert_eq!(result.requests.len(), 1);
        assert_eq!(result.requests[0].name, "Get Users");
        assert_eq!(result.requests[0].url, "https://api.example.com/users");
    }

    #[test]
    fn import_postman_with_nested_folders() {
        let json = r#"{
            "info": { "name": "Nested", "schema": "" },
            "item": [
                {
                    "name": "Auth",
                    "item": [
                        {
                            "name": "Login",
                            "request": { "method": "POST", "url": "/login", "header": [] }
                        }
                    ]
                }
            ]
        }"#;
        let result = import_postman_collection(json).unwrap();
        assert_eq!(result.requests.len(), 1);
        assert_eq!(result.requests[0].name, "Login");
    }

    #[test]
    fn import_postman_with_bearer_auth() {
        let json = r#"{
            "info": { "name": "Auth Test", "schema": "" },
            "item": [{
                "name": "Protected",
                "request": {
                    "method": "GET",
                    "url": "https://api.example.com",
                    "header": [],
                    "auth": {
                        "type": "bearer",
                        "bearer": [{ "key": "token", "value": "abc123" }]
                    }
                }
            }]
        }"#;
        let result = import_postman_collection(json).unwrap();
        match &result.requests[0].auth {
            AuthConfig::BearerToken { token } => assert_eq!(token, "abc123"),
            _ => panic!("Expected BearerToken auth"),
        }
    }

    #[test]
    fn roundtrip_collection_export_import() {
        let collection = RequestCollection {
            id: Uuid::new_v4(),
            name: "Roundtrip Test".to_string(),
            requests: vec![SavedRequest {
                id: Uuid::new_v4(),
                name: "Test Request".to_string(),
                method: HttpMethod::Post,
                url: "https://example.com/api".to_string(),
                query_params: vec![],
                headers: vec![],
                body: RequestBody::Json(r#"{"key":"value"}"#.to_string()),
                auth: AuthConfig::None,
                sort_order: 0,
                created_at: String::new(),
                updated_at: String::new(),
            }],
            sort_order: 0,
            created_at: String::new(),
        };

        let exported = export_postman_collection(&collection).unwrap();
        let imported = import_postman_collection(&exported).unwrap();

        assert_eq!(imported.name, "Roundtrip Test");
        assert_eq!(imported.requests.len(), 1);
        assert_eq!(imported.requests[0].name, "Test Request");
    }

    #[test]
    fn import_postman_environment() {
        let json = r#"{
            "name": "Production",
            "values": [
                { "key": "base_url", "value": "https://api.prod.com", "enabled": true },
                { "key": "token", "value": "secret", "enabled": true }
            ]
        }"#;
        let result = super::import_postman_environment(json).unwrap();
        assert_eq!(result.name, "Production");
        assert_eq!(result.variables.len(), 2);
        assert_eq!(result.variables[0].key, "base_url");
    }
}
