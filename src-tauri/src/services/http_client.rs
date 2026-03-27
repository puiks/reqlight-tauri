use std::time::{Duration, Instant};

use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::models::{
    ApiKeyLocation, AuthConfig, HeaderPair, HttpMethod, KeyValuePair, RequestBody, ResponseRecord,
};

const DEFAULT_TIMEOUT_SECS: u64 = 30;
/// Max response body size we'll read into memory (5 MB)
const MAX_RESPONSE_BODY_BYTES: usize = 5 * 1024 * 1024;

/// Execute an HTTP request and return a ResponseRecord.
/// Accepts a shared `reqwest::Client` (with cookie_store enabled) so that
/// cookies persist across requests within the same session.
#[allow(clippy::too_many_arguments)]
pub async fn execute(
    client: &reqwest::Client,
    method: &HttpMethod,
    url: &str,
    headers: &[KeyValuePair],
    query_params: &[KeyValuePair],
    body: &RequestBody,
    auth: &AuthConfig,
    timeout_secs: Option<u64>,
    follow_redirects: Option<bool>,
) -> Result<ResponseRecord, String> {
    // Build URL with query params
    let mut parsed_url = reqwest::Url::parse(url).map_err(|e| format!("Invalid URL: {e}"))?;

    {
        let enabled_params: Vec<_> = query_params
            .iter()
            .filter(|p| p.is_enabled && !p.key.is_empty())
            .collect();
        if !enabled_params.is_empty() {
            let mut pairs = parsed_url.query_pairs_mut();
            for p in &enabled_params {
                pairs.append_pair(&p.key, &p.value);
            }
        }
    }

    let reqwest_method = match method {
        HttpMethod::Get => reqwest::Method::GET,
        HttpMethod::Post => reqwest::Method::POST,
        HttpMethod::Put => reqwest::Method::PUT,
        HttpMethod::Patch => reqwest::Method::PATCH,
        HttpMethod::Delete => reqwest::Method::DELETE,
    };

    // Set headers
    let mut header_map = HeaderMap::new();
    for h in headers.iter().filter(|h| h.is_enabled && !h.key.is_empty()) {
        if let (Ok(name), Ok(value)) = (
            HeaderName::from_bytes(h.key.as_bytes()),
            HeaderValue::from_str(&h.value),
        ) {
            header_map.insert(name, value);
        }
    }

    // Apply auth configuration (does not override manually set Authorization header)
    match auth {
        AuthConfig::BearerToken { token } if !token.is_empty() => {
            if !header_map.contains_key(reqwest::header::AUTHORIZATION) {
                if let Ok(val) = HeaderValue::from_str(&format!("Bearer {token}")) {
                    header_map.insert(reqwest::header::AUTHORIZATION, val);
                }
            }
        }
        AuthConfig::BasicAuth { username, password } => {
            if !header_map.contains_key(reqwest::header::AUTHORIZATION) {
                let encoded = STANDARD.encode(format!("{username}:{password}"));
                if let Ok(val) = HeaderValue::from_str(&format!("Basic {encoded}")) {
                    header_map.insert(reqwest::header::AUTHORIZATION, val);
                }
            }
        }
        AuthConfig::ApiKey {
            key,
            value,
            location,
        } if !key.is_empty() => match location {
            ApiKeyLocation::Header => {
                if let (Ok(name), Ok(val)) = (
                    HeaderName::from_bytes(key.as_bytes()),
                    HeaderValue::from_str(value),
                ) {
                    header_map.insert(name, val);
                }
            }
            ApiKeyLocation::Query => {
                parsed_url.query_pairs_mut().append_pair(key, value);
            }
        },
        _ => {}
    }

    // If redirects are disabled, build a temporary client with no-redirect policy.
    // reqwest::Client is cheap to construct and redirect policy is per-client.
    let no_redirect_client;
    let effective_client = if follow_redirects == Some(false) {
        no_redirect_client = reqwest::Client::builder()
            .cookie_store(true)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {e}"))?;
        &no_redirect_client
    } else {
        client
    };

    // Build request AFTER auth (ApiKey::Query may modify parsed_url)
    let mut request = effective_client
        .request(reqwest_method, parsed_url)
        .timeout(Duration::from_secs(
            timeout_secs.unwrap_or(DEFAULT_TIMEOUT_SECS),
        ));

    // Set body and auto-set Content-Type if not specified
    let mut is_multipart = false;
    match body {
        RequestBody::Json(text) => {
            if !header_map.contains_key(reqwest::header::CONTENT_TYPE) {
                header_map.insert(
                    reqwest::header::CONTENT_TYPE,
                    HeaderValue::from_static("application/json"),
                );
            }
            request = request.body(text.clone());
        }
        RequestBody::FormData(pairs) => {
            if !header_map.contains_key(reqwest::header::CONTENT_TYPE) {
                header_map.insert(
                    reqwest::header::CONTENT_TYPE,
                    HeaderValue::from_static("application/x-www-form-urlencoded"),
                );
            }
            let encoded: String = pairs
                .iter()
                .filter(|p| p.is_enabled && !p.key.is_empty())
                .map(|p| {
                    format!(
                        "{}={}",
                        urlencoding::encode(&p.key),
                        urlencoding::encode(&p.value)
                    )
                })
                .collect::<Vec<_>>()
                .join("&");
            request = request.body(encoded);
        }
        RequestBody::RawText(text) => {
            request = request.body(text.clone());
        }
        RequestBody::Multipart(fields) => {
            is_multipart = true;
            let mut form = reqwest::multipart::Form::new();
            for field in fields.iter().filter(|f| f.is_enabled && !f.name.is_empty()) {
                if let Some(ref path) = field.file_path {
                    let file_bytes = tokio::fs::read(path)
                        .await
                        .map_err(|e| format!("Failed to read file {path}: {e}"))?;
                    let file_name = std::path::Path::new(path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let part = reqwest::multipart::Part::bytes(file_bytes).file_name(file_name);
                    form = form.part(field.name.clone(), part);
                } else {
                    form = form.text(field.name.clone(), field.value.clone());
                }
            }
            // multipart sets its own Content-Type with boundary; don't set headers before
            request = request.multipart(form);
        }
        RequestBody::None => {}
    }

    // For multipart, headers must be set AFTER .multipart() to avoid overwriting Content-Type
    if !is_multipart {
        request = request.headers(header_map);
    } else {
        // Add headers except Content-Type (multipart sets its own)
        header_map.remove(reqwest::header::CONTENT_TYPE);
        request = request.headers(header_map);
    }

    // Execute with timing
    let start = Instant::now();
    let response = request
        .send()
        .await
        .map_err(|e| format!("Request failed: {e}"))?;
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;

    let status_code = response.status().as_u16() as i32;

    // Collect response headers
    let resp_headers: Vec<HeaderPair> = response
        .headers()
        .iter()
        .map(|(k, v)| HeaderPair {
            key: k.to_string(),
            value: v.to_str().unwrap_or("").to_string(),
        })
        .collect();

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let is_json = content_type.contains("json");

    let body_bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response body: {e}"))?;
    let body_size = body_bytes.len();

    let (body_string, is_truncated) = if body_size > MAX_RESPONSE_BODY_BYTES {
        let truncated = &body_bytes[..MAX_RESPONSE_BODY_BYTES];
        let s = String::from_utf8_lossy(truncated).into_owned();
        (Some(s), true)
    } else {
        (String::from_utf8(body_bytes.to_vec()).ok(), false)
    };

    Ok(ResponseRecord {
        status_code,
        headers: resp_headers,
        body_string,
        elapsed_time: elapsed,
        body_size,
        is_json,
        is_truncated,
        content_type,
    })
}

#[cfg(test)]
#[path = "http_client_tests.rs"]
mod tests;
