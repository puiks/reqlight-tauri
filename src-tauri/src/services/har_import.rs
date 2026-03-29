use crate::models::collection::RequestCollection;
use crate::models::request::now_iso8601;
use crate::models::{AuthConfig, HttpMethod, KeyValuePair, RequestBody, SavedRequest};
use uuid::Uuid;

/// Import a HAR (HTTP Archive) file and return a RequestCollection.
/// HAR spec: http://www.softwareishard.com/blog/har-12-spec/
pub fn import_har(json_str: &str) -> Result<RequestCollection, String> {
    let root: serde_json::Value =
        serde_json::from_str(json_str).map_err(|e| format!("Invalid JSON: {e}"))?;

    let entries = root
        .get("log")
        .and_then(|l| l.get("entries"))
        .and_then(|e| e.as_array())
        .ok_or("Invalid HAR: missing log.entries array")?;

    if entries.is_empty() {
        return Err("HAR file contains no entries".to_string());
    }

    let now = now_iso8601();
    let mut requests = Vec::new();

    for (i, entry) in entries.iter().enumerate() {
        let req = entry.get("request").ok_or("HAR entry missing request")?;
        if let Some(saved) = parse_har_request(req, i as i32, &now) {
            requests.push(saved);
        }
    }

    let collection_name = root
        .get("log")
        .and_then(|l| l.get("pages"))
        .and_then(|p| p.as_array())
        .and_then(|p| p.first())
        .and_then(|p| p.get("title"))
        .and_then(|t| t.as_str())
        .unwrap_or("HAR Import");

    Ok(RequestCollection {
        id: Uuid::new_v4(),
        name: collection_name.to_string(),
        requests,
        sort_order: 0,
        created_at: now,
    })
}

fn parse_har_request(req: &serde_json::Value, index: i32, now: &str) -> Option<SavedRequest> {
    let url_str = req.get("url")?.as_str()?;
    let method_str = req.get("method")?.as_str()?;

    let method = match method_str.to_uppercase().as_str() {
        "GET" => HttpMethod::Get,
        "POST" => HttpMethod::Post,
        "PUT" => HttpMethod::Put,
        "PATCH" => HttpMethod::Patch,
        "DELETE" => HttpMethod::Delete,
        "HEAD" => HttpMethod::Head,
        "OPTIONS" => HttpMethod::Options,
        _ => HttpMethod::Get,
    };

    // Parse URL to separate base URL from query params
    let (base_url, query_params) = parse_url_and_params(url_str, req);

    // Parse headers (skip pseudo-headers and cookie header)
    let headers = parse_headers(req);

    // Parse body
    let body = parse_body(req);

    // Generate name from method + path
    let name = generate_request_name(method_str, url_str);

    Some(SavedRequest {
        id: Uuid::new_v4(),
        name,
        method,
        url: base_url,
        query_params,
        headers,
        body,
        auth: AuthConfig::None,
        sort_order: index,
        created_at: now.to_string(),
        updated_at: now.to_string(),
        response_extractions: vec![],
        assertions: vec![],
        timeout_secs: None,
        pre_request_script: None,
        test_script: None,
    })
}

fn parse_url_and_params(url_str: &str, req: &serde_json::Value) -> (String, Vec<KeyValuePair>) {
    // Prefer HAR queryString array (already parsed)
    let params: Vec<KeyValuePair> = req
        .get("queryString")
        .and_then(|q| q.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|p| {
                    let key = p.get("name")?.as_str()?.to_string();
                    let value = p
                        .get("value")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    Some(KeyValuePair {
                        id: Uuid::new_v4(),
                        key,
                        value,
                        is_enabled: true,
                        is_secret: false,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    // Strip query string from URL
    let base_url = url_str.split('?').next().unwrap_or(url_str).to_string();

    (base_url, params)
}

fn parse_headers(req: &serde_json::Value) -> Vec<KeyValuePair> {
    req.get("headers")
        .and_then(|h| h.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|h| {
                    let key = h.get("name")?.as_str()?.to_string();
                    // Skip pseudo-headers and cookie (browser-managed)
                    if key.starts_with(':') || key.eq_ignore_ascii_case("cookie") {
                        return None;
                    }
                    let value = h
                        .get("value")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    Some(KeyValuePair {
                        id: Uuid::new_v4(),
                        key,
                        value,
                        is_enabled: true,
                        is_secret: false,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_body(req: &serde_json::Value) -> RequestBody {
    let post_data = match req.get("postData") {
        Some(pd) => pd,
        None => return RequestBody::None,
    };

    let mime = post_data
        .get("mimeType")
        .and_then(|m| m.as_str())
        .unwrap_or("");
    let text = post_data.get("text").and_then(|t| t.as_str());

    // Form data
    if mime.contains("application/x-www-form-urlencoded") {
        if let Some(params) = post_data.get("params").and_then(|p| p.as_array()) {
            let pairs: Vec<KeyValuePair> = params
                .iter()
                .filter_map(|p| {
                    let key = p.get("name")?.as_str()?.to_string();
                    let value = p
                        .get("value")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    Some(KeyValuePair {
                        id: Uuid::new_v4(),
                        key,
                        value,
                        is_enabled: true,
                        is_secret: false,
                    })
                })
                .collect();
            if !pairs.is_empty() {
                return RequestBody::FormData(pairs);
            }
        }
    }

    // JSON body
    if mime.contains("application/json") || mime.contains("+json") {
        if let Some(t) = text {
            return RequestBody::Json(t.to_string());
        }
    }

    // Fallback: raw text
    if let Some(t) = text {
        if !t.is_empty() {
            return RequestBody::RawText(t.to_string());
        }
    }

    RequestBody::None
}

fn generate_request_name(method: &str, url: &str) -> String {
    let path = url
        .split("//")
        .nth(1)
        .and_then(|s| s.split('?').next())
        .and_then(|s| s.find('/').map(|i| &s[i..]))
        .unwrap_or("/");

    // Truncate long paths
    let short_path = if path.len() > 50 {
        format!("{}…", &path[..50])
    } else {
        path.to_string()
    };

    format!("{method} {short_path}")
}

#[cfg(test)]
#[path = "har_import_tests.rs"]
mod tests;
