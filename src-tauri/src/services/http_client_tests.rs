use super::*;
use crate::models::AuthConfig;
use crate::test_utils::{make_kv, make_kv_disabled};

fn test_client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap()
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
        None,
    )
    .await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Invalid URL"), "Got: {err}");
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
        None,
    )
    .await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("timed out")
            || err.contains("timeout")
            || err.contains("Network error")
            || err.contains("Request timed out"),
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

    let disabled_header = make_kv_disabled("X-Should-Not-Appear", "nope");
    let disabled_param = make_kv_disabled("skip", "true");

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
        None,
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
        None,
    )
    .await
    .unwrap();

    assert_eq!(result.content_type, "text/html; charset=utf-8");
    mock.assert_async().await;
}
