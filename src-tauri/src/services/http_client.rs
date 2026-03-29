use std::time::{Duration, Instant};

use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::constants::{DEFAULT_TIMEOUT_SECS, MAX_RESPONSE_BODY_BYTES};
use crate::error::AppError;
use crate::models::{
    ApiKeyLocation, AuthConfig, HeaderPair, HttpMethod, KeyValuePair, ProxyConfig, RequestBody,
    ResponseRecord,
};

/// Parse URL and append enabled query parameters.
fn build_url(url: &str, query_params: &[KeyValuePair]) -> Result<reqwest::Url, AppError> {
    let mut parsed =
        reqwest::Url::parse(url).map_err(|e| AppError::Validation(format!("Invalid URL: {e}")))?;

    let enabled: Vec<_> = query_params
        .iter()
        .filter(|p| p.is_enabled && !p.key.is_empty())
        .collect();
    if !enabled.is_empty() {
        let mut pairs = parsed.query_pairs_mut();
        for p in &enabled {
            pairs.append_pair(&p.key, &p.value);
        }
    }

    Ok(parsed)
}

/// Convert user headers into a reqwest HeaderMap.
fn build_headers(headers: &[KeyValuePair]) -> HeaderMap {
    let mut map = HeaderMap::new();
    for h in headers.iter().filter(|h| h.is_enabled && !h.key.is_empty()) {
        if let (Ok(name), Ok(value)) = (
            HeaderName::from_bytes(h.key.as_bytes()),
            HeaderValue::from_str(&h.value),
        ) {
            map.insert(name, value);
        }
    }
    map
}

/// Apply auth configuration to headers and URL.
/// Does not override a manually set Authorization header.
fn apply_auth(auth: &AuthConfig, header_map: &mut HeaderMap, parsed_url: &mut reqwest::Url) {
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
        AuthConfig::OAuth2 { access_token, .. } if !access_token.is_empty() => {
            if !header_map.contains_key(reqwest::header::AUTHORIZATION) {
                if let Ok(val) = HeaderValue::from_str(&format!("Bearer {access_token}")) {
                    header_map.insert(reqwest::header::AUTHORIZATION, val);
                }
            }
        }
        _ => {}
    }
}

/// Build an effective HTTP client, creating a custom one when proxy or
/// no-redirect settings require it.
fn build_client<'a>(
    default_client: &'a reqwest::Client,
    follow_redirects: Option<bool>,
    proxy_config: Option<&ProxyConfig>,
) -> Result<std::borrow::Cow<'a, reqwest::Client>, AppError> {
    let needs_custom = follow_redirects == Some(false)
        || proxy_config.is_some_and(|p| p.enabled && !p.proxy_url.is_empty());

    if !needs_custom {
        return Ok(std::borrow::Cow::Borrowed(default_client));
    }

    let mut builder = reqwest::Client::builder().cookie_store(true);
    if follow_redirects == Some(false) {
        builder = builder.redirect(reqwest::redirect::Policy::none());
    }
    if let Some(proxy) = proxy_config.filter(|p| p.enabled && !p.proxy_url.is_empty()) {
        let mut p = reqwest::Proxy::all(&proxy.proxy_url)
            .map_err(|e| AppError::Network(format!("Invalid proxy URL: {e}")))?;
        if !proxy.no_proxy.is_empty() {
            if let Some(no_proxy) = reqwest::NoProxy::from_string(&proxy.no_proxy) {
                p = p.no_proxy(Some(no_proxy));
            }
        }
        builder = builder.proxy(p);
    }

    let client = builder
        .build()
        .map_err(|e| AppError::Network(format!("Failed to create HTTP client: {e}")))?;
    Ok(std::borrow::Cow::Owned(client))
}

/// Attach request body and set appropriate Content-Type headers.
/// Returns `(request_builder, is_multipart)`.
async fn attach_body(
    mut request: reqwest::RequestBuilder,
    body: &RequestBody,
    header_map: &mut HeaderMap,
) -> Result<(reqwest::RequestBuilder, bool), AppError> {
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
        RequestBody::GraphQL { query, variables } => {
            if !header_map.contains_key(reqwest::header::CONTENT_TYPE) {
                header_map.insert(
                    reqwest::header::CONTENT_TYPE,
                    HeaderValue::from_static("application/json"),
                );
            }
            let vars_value: serde_json::Value = if variables.trim().is_empty() {
                serde_json::Value::Null
            } else {
                serde_json::from_str(variables).unwrap_or(serde_json::Value::Null)
            };
            let gql_body = serde_json::json!({
                "query": query,
                "variables": vars_value,
            });
            request = request.body(gql_body.to_string());
        }
        RequestBody::Multipart(fields) => {
            is_multipart = true;
            let mut form = reqwest::multipart::Form::new();
            for field in fields.iter().filter(|f| f.is_enabled && !f.name.is_empty()) {
                if let Some(ref path) = field.file_path {
                    let file_bytes = tokio::fs::read(path)
                        .await
                        .map_err(|e| AppError::Io(format!("Failed to read file {path}: {e}")))?;
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
            request = request.multipart(form);
        }
        RequestBody::None => {}
    }

    Ok((request, is_multipart))
}

/// Read the response into a ResponseRecord, handling truncation for large bodies.
async fn read_response(
    response: reqwest::Response,
    elapsed: f64,
) -> Result<ResponseRecord, AppError> {
    let status_code = response.status().as_u16() as i32;

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

    let body_bytes = response.bytes().await?;
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
    proxy_config: Option<&ProxyConfig>,
) -> Result<ResponseRecord, AppError> {
    let mut parsed_url = build_url(url, query_params)?;
    let mut header_map = build_headers(headers);
    apply_auth(auth, &mut header_map, &mut parsed_url);

    let effective_client = build_client(client, follow_redirects, proxy_config)?;

    let reqwest_method = match method {
        HttpMethod::Get => reqwest::Method::GET,
        HttpMethod::Post => reqwest::Method::POST,
        HttpMethod::Put => reqwest::Method::PUT,
        HttpMethod::Patch => reqwest::Method::PATCH,
        HttpMethod::Delete => reqwest::Method::DELETE,
        HttpMethod::Head => reqwest::Method::HEAD,
        HttpMethod::Options => reqwest::Method::OPTIONS,
    };

    let request = effective_client
        .request(reqwest_method, parsed_url)
        .timeout(Duration::from_secs(
            timeout_secs.unwrap_or(DEFAULT_TIMEOUT_SECS),
        ));

    let (request, is_multipart) = attach_body(request, body, &mut header_map).await?;

    // For multipart, headers must be set AFTER .multipart() to avoid overwriting Content-Type
    let request = if is_multipart {
        header_map.remove(reqwest::header::CONTENT_TYPE);
        request.headers(header_map)
    } else {
        request.headers(header_map)
    };

    let start = Instant::now();
    tracing::debug!(?method, %url, "Sending HTTP request");
    let response = request.send().await?;
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    tracing::info!(
        status = response.status().as_u16(),
        elapsed_ms = format!("{elapsed:.1}"),
        "HTTP response received"
    );

    read_response(response, elapsed).await
}

#[cfg(test)]
#[path = "http_client_tests.rs"]
mod tests;
