use crate::models::collection::RequestCollection;
use crate::models::{HttpMethod, KeyValuePair, RequestBody, SavedRequest};
use serde_json::Value;
use uuid::Uuid;

/// Import an OpenAPI 3.x or Swagger 2.x spec (JSON or YAML) into collections.
/// Groups requests by the first tag; untagged requests go into a "Default" collection.
pub fn import_openapi(input: &str) -> Result<Vec<RequestCollection>, String> {
    let doc: Value = parse_input(input)?;

    let base_url = extract_base_url(&doc);
    let paths = doc
        .get("paths")
        .and_then(|v| v.as_object())
        .ok_or("Missing or invalid 'paths' in spec")?;

    let mut tag_map: std::collections::BTreeMap<String, Vec<SavedRequest>> =
        std::collections::BTreeMap::new();
    let mut order = 0i32;

    for (path, path_item) in paths {
        let path_obj = match path_item.as_object() {
            Some(o) => o,
            None => continue,
        };

        for (method_str, operation) in path_obj {
            let method = match parse_method(method_str) {
                Some(m) => m,
                None => continue, // skip "parameters", "summary", etc.
            };

            let op_obj = match operation.as_object() {
                Some(o) => o,
                None => continue,
            };

            let name = op_obj
                .get("summary")
                .and_then(|v| v.as_str())
                .or_else(|| op_obj.get("operationId").and_then(|v| v.as_str()))
                .unwrap_or(path.as_str())
                .to_string();

            let tag = op_obj
                .get("tags")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .unwrap_or("Default")
                .to_string();

            let (query_params, headers) = extract_parameters(op_obj, &doc);
            let body = extract_request_body(op_obj, &doc);
            let url = format!("{}{}", base_url, path);

            let now = crate::models::request::now_iso8601();
            let request = SavedRequest {
                id: Uuid::new_v4(),
                name,
                method,
                url,
                query_params,
                headers,
                body,
                auth: crate::models::AuthConfig::None,
                sort_order: order,
                created_at: now.clone(),
                updated_at: now,
                response_extractions: vec![],
                timeout_secs: None,
            };
            order += 1;

            tag_map.entry(tag).or_default().push(request);
        }
    }

    let collections = tag_map
        .into_iter()
        .enumerate()
        .map(|(i, (tag, requests))| RequestCollection {
            id: Uuid::new_v4(),
            name: tag,
            requests,
            sort_order: i as i32,
            created_at: crate::models::request::now_iso8601(),
        })
        .collect();

    Ok(collections)
}

fn parse_input(input: &str) -> Result<Value, String> {
    // Try JSON first (faster), then YAML
    if let Ok(v) = serde_json::from_str::<Value>(input) {
        return Ok(v);
    }
    serde_yaml::from_str::<Value>(input).map_err(|e| format!("Failed to parse spec: {e}"))
}

fn extract_base_url(doc: &Value) -> String {
    // OpenAPI 3.x: servers[0].url
    if let Some(url) = doc
        .get("servers")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|s| s.get("url"))
        .and_then(|v| v.as_str())
    {
        return url.trim_end_matches('/').to_string();
    }

    // Swagger 2.x: scheme + host + basePath
    let host = doc.get("host").and_then(|v| v.as_str()).unwrap_or("");
    if host.is_empty() {
        return String::new();
    }
    let scheme = doc
        .get("schemes")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .unwrap_or("https");
    let base_path = doc.get("basePath").and_then(|v| v.as_str()).unwrap_or("");

    format!("{}://{}{}", scheme, host, base_path.trim_end_matches('/'))
}

fn parse_method(s: &str) -> Option<HttpMethod> {
    match s.to_lowercase().as_str() {
        "get" => Some(HttpMethod::Get),
        "post" => Some(HttpMethod::Post),
        "put" => Some(HttpMethod::Put),
        "patch" => Some(HttpMethod::Patch),
        "delete" => Some(HttpMethod::Delete),
        "head" => Some(HttpMethod::Head),
        "options" => Some(HttpMethod::Options),
        _ => None,
    }
}

fn extract_parameters(
    operation: &serde_json::Map<String, Value>,
    doc: &Value,
) -> (Vec<KeyValuePair>, Vec<KeyValuePair>) {
    let mut query_params = Vec::new();
    let mut headers = Vec::new();

    let params = match operation.get("parameters").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return (query_params, headers),
    };

    for param_val in params {
        let param = resolve_ref(param_val, doc);
        let name = param.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let location = param.get("in").and_then(|v| v.as_str()).unwrap_or("");
        let example = param
            .get("example")
            .or_else(|| {
                param
                    .get("schema")
                    .and_then(|s| s.get("default").or_else(|| s.get("example")))
            })
            .and_then(value_to_string)
            .unwrap_or_default();

        let kv = KeyValuePair {
            id: Uuid::new_v4(),
            key: name.to_string(),
            value: example,
            is_enabled: param
                .get("required")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            is_secret: false,
        };

        match location {
            "query" => query_params.push(kv),
            "header" => headers.push(kv),
            _ => {} // path params handled via URL template, cookie params ignored
        }
    }

    (query_params, headers)
}

fn extract_request_body(operation: &serde_json::Map<String, Value>, doc: &Value) -> RequestBody {
    // OpenAPI 3.x: requestBody.content
    if let Some(rb) = operation.get("requestBody") {
        let resolved = resolve_ref(rb, doc);
        if let Some(content) = resolved.get("content").and_then(|v| v.as_object()) {
            if content.contains_key("application/json") {
                let example = content
                    .get("application/json")
                    .and_then(|v| v.get("schema"))
                    .and_then(|s| resolve_ref(s, doc).get("example").cloned())
                    .map(|v| serde_json::to_string_pretty(&v).unwrap_or_default())
                    .unwrap_or_default();
                return RequestBody::Json(example);
            }
            if content.contains_key("application/x-www-form-urlencoded") {
                return RequestBody::FormData(vec![]);
            }
        }
    }

    // Swagger 2.x: body parameter with in=body
    if let Some(params) = operation.get("parameters").and_then(|v| v.as_array()) {
        for p in params {
            let resolved = resolve_ref(p, doc);
            if resolved.get("in").and_then(|v| v.as_str()) == Some("body") {
                let example = resolved
                    .get("schema")
                    .and_then(|s| resolve_ref(s, doc).get("example").cloned())
                    .map(|v| serde_json::to_string_pretty(&v).unwrap_or_default())
                    .unwrap_or_default();
                return RequestBody::Json(example);
            }
        }
    }

    RequestBody::None
}

fn resolve_ref<'a>(value: &'a Value, doc: &'a Value) -> &'a Value {
    if let Some(ref_path) = value.get("$ref").and_then(|v| v.as_str()) {
        // Only handle local refs like "#/components/schemas/Foo"
        let parts: Vec<&str> = ref_path.trim_start_matches("#/").split('/').collect();
        let mut current = doc;
        for part in &parts {
            current = match current.get(part) {
                Some(v) => v,
                None => return value, // ref not found, return original
            };
        }
        current
    } else {
        value
    }
}

fn value_to_string(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

#[cfg(test)]
#[path = "openapi_import_tests.rs"]
mod tests;
