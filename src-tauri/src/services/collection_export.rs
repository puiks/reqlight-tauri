use crate::models::collection::RequestCollection;
use crate::models::{AuthConfig, RequestBody, RequestEnvironment, SavedRequest};

use super::collection_types::*;

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
#[path = "collection_export_tests.rs"]
mod tests;
