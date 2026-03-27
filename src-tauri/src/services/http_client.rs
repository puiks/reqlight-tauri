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
pub async fn execute(
    method: &HttpMethod,
    url: &str,
    headers: &[KeyValuePair],
    query_params: &[KeyValuePair],
    body: &RequestBody,
    auth: &AuthConfig,
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

    let mut request = client.request(reqwest_method, parsed_url.clone());

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

    // Set body and auto-set Content-Type if not specified
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AuthConfig, KeyValuePair};
    use uuid::Uuid;

    fn make_kv(key: &str, value: &str) -> KeyValuePair {
        KeyValuePair {
            id: Uuid::new_v4(),
            key: key.to_string(),
            value: value.to_string(),
            is_enabled: true,
            is_secret: false,
        }
    }

    #[tokio::test]
    async fn get_request_returns_200() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/test")
            .with_status(200)
            .with_body("ok")
            .create_async()
            .await;

        let result = execute(
            &HttpMethod::Get,
            &format!("{}/test", server.url()),
            &[],
            &[],
            &RequestBody::None,
            &AuthConfig::None,
            Some(5),
        )
        .await
        .unwrap();

        assert_eq!(result.status_code, 200);
        assert_eq!(result.body_string.as_deref(), Some("ok"));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn post_json_auto_content_type() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/api")
            .match_header("content-type", "application/json")
            .with_status(201)
            .create_async()
            .await;

        let result = execute(
            &HttpMethod::Post,
            &format!("{}/api", server.url()),
            &[],
            &[],
            &RequestBody::Json(r#"{"key":"val"}"#.to_string()),
            &AuthConfig::None,
            Some(5),
        )
        .await
        .unwrap();

        assert_eq!(result.status_code, 201);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn query_params_appended_to_url() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/search")
            .match_query(mockito::Matcher::UrlEncoded("q".into(), "hello".into()))
            .with_status(200)
            .create_async()
            .await;

        execute(
            &HttpMethod::Get,
            &format!("{}/search", server.url()),
            &[],
            &[make_kv("q", "hello")],
            &RequestBody::None,
            &AuthConfig::None,
            Some(5),
        )
        .await
        .unwrap();

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn custom_headers_sent() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/")
            .match_header("x-custom", "value")
            .with_status(200)
            .create_async()
            .await;

        execute(
            &HttpMethod::Get,
            &server.url(),
            &[make_kv("X-Custom", "value")],
            &[],
            &RequestBody::None,
            &AuthConfig::None,
            Some(5),
        )
        .await
        .unwrap();

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn bearer_auth_injects_header() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/")
            .match_header("authorization", "Bearer my-token")
            .with_status(200)
            .create_async()
            .await;

        execute(
            &HttpMethod::Get,
            &server.url(),
            &[],
            &[],
            &RequestBody::None,
            &AuthConfig::BearerToken {
                token: "my-token".to_string(),
            },
            Some(5),
        )
        .await
        .unwrap();

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn basic_auth_injects_header() {
        let mut server = mockito::Server::new_async().await;
        let expected = format!("Basic {}", STANDARD.encode("user:pass"));
        let mock = server
            .mock("GET", "/")
            .match_header("authorization", expected.as_str())
            .with_status(200)
            .create_async()
            .await;

        execute(
            &HttpMethod::Get,
            &server.url(),
            &[],
            &[],
            &RequestBody::None,
            &AuthConfig::BasicAuth {
                username: "user".to_string(),
                password: "pass".to_string(),
            },
            Some(5),
        )
        .await
        .unwrap();

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn auth_does_not_override_manual_authorization() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/")
            .match_header("authorization", "Manual value")
            .with_status(200)
            .create_async()
            .await;

        execute(
            &HttpMethod::Get,
            &server.url(),
            &[make_kv("Authorization", "Manual value")],
            &[],
            &RequestBody::None,
            &AuthConfig::BearerToken {
                token: "should-not-appear".to_string(),
            },
            Some(5),
        )
        .await
        .unwrap();

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn api_key_header_auth() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/")
            .match_header("x-api-key", "secret")
            .with_status(200)
            .create_async()
            .await;

        execute(
            &HttpMethod::Get,
            &server.url(),
            &[],
            &[],
            &RequestBody::None,
            &AuthConfig::ApiKey {
                key: "X-API-Key".to_string(),
                value: "secret".to_string(),
                location: ApiKeyLocation::Header,
            },
            Some(5),
        )
        .await
        .unwrap();

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn invalid_url_returns_error() {
        let result = execute(
            &HttpMethod::Get,
            "not-a-url",
            &[],
            &[],
            &RequestBody::None,
            &AuthConfig::None,
            Some(5),
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid URL"));
    }

    #[tokio::test]
    async fn json_response_detected() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/")
            .with_header("content-type", "application/json")
            .with_body(r#"{"ok":true}"#)
            .with_status(200)
            .create_async()
            .await;

        let result = execute(
            &HttpMethod::Get,
            &server.url(),
            &[],
            &[],
            &RequestBody::None,
            &AuthConfig::None,
            Some(5),
        )
        .await
        .unwrap();

        assert!(result.is_json);
        mock.assert_async().await;
    }
}
