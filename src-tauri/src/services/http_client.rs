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
mod tests {
    use super::*;
    use crate::models::{AuthConfig, KeyValuePair};
    use uuid::Uuid;

    fn test_client() -> reqwest::Client {
        reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .unwrap()
    }

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
            &test_client(),
            &HttpMethod::Get,
            &format!("{}/test", server.url()),
            &[],
            &[],
            &RequestBody::None,
            &AuthConfig::None,
            Some(5),
            None,
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
            &test_client(),
            &HttpMethod::Post,
            &format!("{}/api", server.url()),
            &[],
            &[],
            &RequestBody::Json(r#"{"key":"val"}"#.to_string()),
            &AuthConfig::None,
            Some(5),
            None,
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
            &test_client(),
            &HttpMethod::Get,
            &format!("{}/search", server.url()),
            &[],
            &[make_kv("q", "hello")],
            &RequestBody::None,
            &AuthConfig::None,
            Some(5),
            None,
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
            &test_client(),
            &HttpMethod::Get,
            &server.url(),
            &[make_kv("X-Custom", "value")],
            &[],
            &RequestBody::None,
            &AuthConfig::None,
            Some(5),
            None,
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
            &test_client(),
            &HttpMethod::Get,
            &server.url(),
            &[],
            &[],
            &RequestBody::None,
            &AuthConfig::BearerToken {
                token: "my-token".to_string(),
            },
            Some(5),
            None,
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
            &test_client(),
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
            None,
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
            &test_client(),
            &HttpMethod::Get,
            &server.url(),
            &[make_kv("Authorization", "Manual value")],
            &[],
            &RequestBody::None,
            &AuthConfig::BearerToken {
                token: "should-not-appear".to_string(),
            },
            Some(5),
            None,
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
            &test_client(),
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
            None,
        )
        .await
        .unwrap();

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn invalid_url_returns_error() {
        let result = execute(
            &test_client(),
            &HttpMethod::Get,
            "not-a-url",
            &[],
            &[],
            &RequestBody::None,
            &AuthConfig::None,
            Some(5),
            None,
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
            &test_client(),
            &HttpMethod::Get,
            &server.url(),
            &[],
            &[],
            &RequestBody::None,
            &AuthConfig::None,
            Some(5),
            None,
        )
        .await
        .unwrap();

        assert!(result.is_json);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn form_data_body_encoded_with_content_type() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/form")
            .match_header("content-type", "application/x-www-form-urlencoded")
            .match_body("name=John&age=30")
            .with_status(200)
            .create_async()
            .await;

        let pairs = vec![make_kv("name", "John"), make_kv("age", "30")];

        execute(
            &test_client(),
            &HttpMethod::Post,
            &format!("{}/form", server.url()),
            &[],
            &[],
            &RequestBody::FormData(pairs),
            &AuthConfig::None,
            Some(5),
            None,
        )
        .await
        .unwrap();

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn raw_text_body_sent_without_auto_content_type() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/raw")
            .match_body("plain text body")
            .with_status(200)
            .create_async()
            .await;

        let result = execute(
            &test_client(),
            &HttpMethod::Post,
            &format!("{}/raw", server.url()),
            &[],
            &[],
            &RequestBody::RawText("plain text body".to_string()),
            &AuthConfig::None,
            Some(5),
            None,
        )
        .await
        .unwrap();

        assert_eq!(result.status_code, 200);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn large_response_truncated() {
        let mut server = mockito::Server::new_async().await;
        // Create a body larger than 5MB
        let big_body = "x".repeat(MAX_RESPONSE_BODY_BYTES + 1024);
        let mock = server
            .mock("GET", "/big")
            .with_status(200)
            .with_body(&big_body)
            .create_async()
            .await;

        let result = execute(
            &test_client(),
            &HttpMethod::Get,
            &format!("{}/big", server.url()),
            &[],
            &[],
            &RequestBody::None,
            &AuthConfig::None,
            Some(30),
            None,
        )
        .await
        .unwrap();

        assert!(result.is_truncated);
        assert!(result.body_string.unwrap().len() <= MAX_RESPONSE_BODY_BYTES);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn timeout_returns_error() {
        let mut server = mockito::Server::new_async().await;
        // Mock that never responds quickly enough
        let _mock = server
            .mock("GET", "/slow")
            .with_status(200)
            .with_chunked_body(|w| {
                std::thread::sleep(std::time::Duration::from_secs(5));
                w.write_all(b"delayed")?;
                Ok(())
            })
            .create_async()
            .await;

        let result = execute(
            &test_client(),
            &HttpMethod::Get,
            &format!("{}/slow", server.url()),
            &[],
            &[],
            &RequestBody::None,
            &AuthConfig::None,
            Some(1),
            None,
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("Request failed")
                || err.contains("timed out")
                || err.contains("timeout")
                || err.contains("Failed to read response body"),
            "Expected timeout error, got: {err}"
        );
    }

    #[tokio::test]
    async fn api_key_query_location() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/data")
            .match_query(mockito::Matcher::UrlEncoded(
                "api_key".into(),
                "my-secret".into(),
            ))
            .with_status(200)
            .create_async()
            .await;

        execute(
            &test_client(),
            &HttpMethod::Get,
            &format!("{}/data", server.url()),
            &[],
            &[],
            &RequestBody::None,
            &AuthConfig::ApiKey {
                key: "api_key".to_string(),
                value: "my-secret".to_string(),
                location: ApiKeyLocation::Query,
            },
            Some(5),
            None,
        )
        .await
        .unwrap();

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn disabled_params_and_headers_ignored() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/")
            .with_status(200)
            .create_async()
            .await;

        let disabled_header = KeyValuePair {
            id: Uuid::new_v4(),
            key: "X-Should-Not-Appear".to_string(),
            value: "nope".to_string(),
            is_enabled: false,
            is_secret: false,
        };
        let disabled_param = KeyValuePair {
            id: Uuid::new_v4(),
            key: "skip".to_string(),
            value: "true".to_string(),
            is_enabled: false,
            is_secret: false,
        };

        let result = execute(
            &test_client(),
            &HttpMethod::Get,
            &server.url(),
            &[disabled_header],
            &[disabled_param],
            &RequestBody::None,
            &AuthConfig::None,
            Some(5),
            None,
        )
        .await
        .unwrap();

        assert_eq!(result.status_code, 200);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn non_json_content_type_detected() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/")
            .with_header("content-type", "text/html")
            .with_body("<h1>Hello</h1>")
            .with_status(200)
            .create_async()
            .await;

        let result = execute(
            &test_client(),
            &HttpMethod::Get,
            &server.url(),
            &[],
            &[],
            &RequestBody::None,
            &AuthConfig::None,
            Some(5),
            None,
        )
        .await
        .unwrap();

        assert!(!result.is_json);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn empty_bearer_token_not_injected() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/")
            .match_header("authorization", mockito::Matcher::Missing)
            .with_status(200)
            .create_async()
            .await;

        execute(
            &test_client(),
            &HttpMethod::Get,
            &server.url(),
            &[],
            &[],
            &RequestBody::None,
            &AuthConfig::BearerToken {
                token: "".to_string(),
            },
            Some(5),
            None,
        )
        .await
        .unwrap();

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn cookie_jar_persists_across_requests() {
        let mut server = mockito::Server::new_async().await;

        // First request: server sets a cookie
        let mock_set = server
            .mock("GET", "/login")
            .with_status(200)
            .with_header("set-cookie", "session=abc123; Path=/")
            .create_async()
            .await;

        // Second request: expect the cookie to be sent back automatically
        let mock_check = server
            .mock("GET", "/dashboard")
            .match_header("cookie", "session=abc123")
            .with_status(200)
            .create_async()
            .await;

        // Use the SAME client for both requests (cookie jar shared)
        let client = test_client();

        execute(
            &client,
            &HttpMethod::Get,
            &format!("{}/login", server.url()),
            &[],
            &[],
            &RequestBody::None,
            &AuthConfig::None,
            Some(5),
            None,
        )
        .await
        .unwrap();

        execute(
            &client,
            &HttpMethod::Get,
            &format!("{}/dashboard", server.url()),
            &[],
            &[],
            &RequestBody::None,
            &AuthConfig::None,
            Some(5),
            None,
        )
        .await
        .unwrap();

        mock_set.assert_async().await;
        mock_check.assert_async().await;
    }

    #[tokio::test]
    async fn follow_redirects_disabled_returns_302() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/old")
            .with_status(302)
            .with_header("location", "/new")
            .create_async()
            .await;

        let result = execute(
            &test_client(),
            &HttpMethod::Get,
            &format!("{}/old", server.url()),
            &[],
            &[],
            &RequestBody::None,
            &AuthConfig::None,
            Some(5),
            Some(false),
        )
        .await
        .unwrap();

        assert_eq!(result.status_code, 302);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn content_type_returned_in_response() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/")
            .with_header("content-type", "text/html; charset=utf-8")
            .with_body("<h1>hi</h1>")
            .with_status(200)
            .create_async()
            .await;

        let result = execute(
            &test_client(),
            &HttpMethod::Get,
            &server.url(),
            &[],
            &[],
            &RequestBody::None,
            &AuthConfig::None,
            Some(5),
            None,
        )
        .await
        .unwrap();

        assert_eq!(result.content_type, "text/html; charset=utf-8");
        mock.assert_async().await;
    }
}
