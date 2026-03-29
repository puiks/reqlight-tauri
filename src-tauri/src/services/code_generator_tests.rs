use super::*;
use crate::models::{AuthConfig, HttpMethod, RequestBody, SavedRequest};
use crate::test_utils::make_kv;
use uuid::Uuid;

fn base_request() -> SavedRequest {
    SavedRequest {
        id: Uuid::new_v4(),
        name: "Test".to_string(),
        method: HttpMethod::Get,
        url: "https://api.example.com/data".to_string(),
        query_params: vec![],
        headers: vec![],
        body: RequestBody::None,
        auth: AuthConfig::None,
        sort_order: 0,
        created_at: String::new(),
        updated_at: String::new(),
        response_extractions: vec![],
        assertions: vec![],
        timeout_secs: None,
        pre_request_script: None,
        test_script: None,
    }
}

// --- fetch tests ---

#[test]
fn fetch_simple_get() {
    let req = base_request();
    let result = generate(&req, None, "javascript-fetch").unwrap();
    assert!(result.contains("fetch(\"https://api.example.com/data\""));
    assert!(result.contains("method: \"GET\""));
}

#[test]
fn fetch_post_with_json() {
    let mut req = base_request();
    req.method = HttpMethod::Post;
    req.body = RequestBody::Json(r#"{"key":"val"}"#.to_string());
    let result = generate(&req, None, "javascript-fetch").unwrap();
    assert!(result.contains("method: \"POST\""));
    assert!(result.contains("application/json"));
    assert!(result.contains("body:"));
}

#[test]
fn fetch_with_bearer_auth() {
    let mut req = base_request();
    req.auth = AuthConfig::BearerToken {
        token: "tok123".to_string(),
    };
    let result = generate(&req, None, "javascript-fetch").unwrap();
    assert!(result.contains("Bearer tok123"));
}

#[test]
fn fetch_with_query_params() {
    let mut req = base_request();
    req.query_params = vec![make_kv("q", "hello")];
    let result = generate(&req, None, "javascript-fetch").unwrap();
    assert!(result.contains("q=hello"));
}

// --- axios tests ---

#[test]
fn axios_simple_get() {
    let req = base_request();
    let result = generate(&req, None, "javascript-axios").unwrap();
    assert!(result.contains("axios.get("));
}

#[test]
fn axios_post_with_json() {
    let mut req = base_request();
    req.method = HttpMethod::Post;
    req.body = RequestBody::Json(r#"{"key":"val"}"#.to_string());
    let result = generate(&req, None, "javascript-axios").unwrap();
    assert!(result.contains("axios.post("));
    assert!(result.contains("data:"));
}

#[test]
fn axios_with_basic_auth() {
    let mut req = base_request();
    req.auth = AuthConfig::BasicAuth {
        username: "user".to_string(),
        password: "pass".to_string(),
    };
    let result = generate(&req, None, "javascript-axios").unwrap();
    assert!(result.contains("Authorization"));
    assert!(result.contains("Basic"));
}

// --- Python tests ---

#[test]
fn python_simple_get() {
    let req = base_request();
    let result = generate(&req, None, "python-requests").unwrap();
    assert!(result.contains("import requests"));
    assert!(result.contains("requests.get("));
}

#[test]
fn python_post_with_json() {
    let mut req = base_request();
    req.method = HttpMethod::Post;
    req.body = RequestBody::Json(r#"{"key":"val"}"#.to_string());
    let result = generate(&req, None, "python-requests").unwrap();
    assert!(result.contains("requests.post("));
    assert!(result.contains("headers=headers"));
    assert!(result.contains("data=data"));
}

#[test]
fn python_with_headers() {
    let mut req = base_request();
    req.headers = vec![make_kv("X-Custom", "value")];
    let result = generate(&req, None, "python-requests").unwrap();
    assert!(result.contains("\"X-Custom\": \"value\""));
}

#[test]
fn python_with_api_key_query() {
    let mut req = base_request();
    req.auth = AuthConfig::ApiKey {
        key: "api_key".to_string(),
        value: "secret".to_string(),
        location: ApiKeyLocation::Query,
    };
    let result = generate(&req, None, "python-requests").unwrap();
    assert!(result.contains("api_key=secret"));
}

// --- curl delegation ---

#[test]
fn curl_delegates_to_exporter() {
    let req = base_request();
    let result = generate(&req, None, "curl").unwrap();
    assert!(result.starts_with("curl"));
}

// --- env interpolation ---

#[test]
fn interpolation_applied() {
    let mut req = base_request();
    req.url = "https://{{host}}/api".to_string();
    let env = RequestEnvironment {
        id: Uuid::new_v4(),
        name: "test".to_string(),
        variables: vec![make_kv("host", "api.example.com")],
    };
    let result = generate(&req, Some(&env), "javascript-fetch").unwrap();
    assert!(result.contains("api.example.com"));
    assert!(!result.contains("{{host}}"));
}

// --- unsupported language ---

#[test]
fn unsupported_language_returns_error() {
    let req = base_request();
    let result = generate(&req, None, "go");
    assert!(result.is_err());
}
