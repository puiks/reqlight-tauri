use std::time::{Duration, Instant};

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::models::{HeaderPair, HttpMethod, KeyValuePair, RequestBody, ResponseRecord};

const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Execute an HTTP request and return a ResponseRecord.
pub async fn execute(
    method: &HttpMethod,
    url: &str,
    headers: &[KeyValuePair],
    query_params: &[KeyValuePair],
    body: &RequestBody,
    timeout_secs: Option<u64>,
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

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(
            timeout_secs.unwrap_or(DEFAULT_TIMEOUT_SECS),
        ))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let reqwest_method = match method {
        HttpMethod::Get => reqwest::Method::GET,
        HttpMethod::Post => reqwest::Method::POST,
        HttpMethod::Put => reqwest::Method::PUT,
        HttpMethod::Patch => reqwest::Method::PATCH,
        HttpMethod::Delete => reqwest::Method::DELETE,
    };

    let mut request = client.request(reqwest_method, parsed_url);

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

    // Set body and auto-set Content-Type if not specified
    match body {
        RequestBody::Json(text) => {
            if !header_map.contains_key("content-type") {
                header_map.insert(
                    reqwest::header::CONTENT_TYPE,
                    HeaderValue::from_static("application/json"),
                );
            }
            request = request.body(text.clone());
        }
        RequestBody::FormData(pairs) => {
            if !header_map.contains_key("content-type") {
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
        RequestBody::None => {}
    }

    request = request.headers(header_map);

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
    let body_string = String::from_utf8(body_bytes.to_vec()).ok();

    Ok(ResponseRecord {
        status_code,
        headers: resp_headers,
        body_string,
        elapsed_time: elapsed,
        body_size,
        is_json,
    })
}
