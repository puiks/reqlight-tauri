use super::*;
use crate::models::{ApiKeyLocation, AuthConfig, HttpMethod, RequestBody, SavedRequest};
use crate::test_utils::make_kv;
use uuid::Uuid;

fn base_request() -> SavedRequest {
    SavedRequest {
        id: Uuid::new_v4(),
        name: "Test".to_string(),
        method: HttpMethod::Get,
        url: "https://example.com".to_string(),
        query_params: vec![],
        headers: vec![],
        body: RequestBody::None,
        auth: AuthConfig::None,
        sort_order: 0,
        created_at: String::new(),
        updated_at: String::new(),
        response_extractions: vec![],
        timeout_secs: None,
    }
}

#[test]
fn export_simple_get() {
    let req = base_request();
    let result = export(&req, None);
    assert_eq!(result, "curl \\\n  'https://example.com'");
}

#[test]
fn export_post_with_method() {
    let mut req = base_request();
    req.method = HttpMethod::Post;
    let result = export(&req, None);
    assert!(result.contains("-X POST"));
}

#[test]
fn export_with_headers() {
    let mut req = base_request();
    req.headers = vec![make_kv("Content-Type", "application/json")];
    let result = export(&req, None);
    assert!(result.contains("-H 'Content-Type: application/json'"));
}

#[test]
fn export_with_query_params() {
    let mut req = base_request();
    req.query_params = vec![make_kv("q", "hello world")];
    let result = export(&req, None);
    assert!(result.contains("q=hello%20world"));
}

#[test]
fn export_with_json_body() {
    let mut req = base_request();
    req.method = HttpMethod::Post;
    req.body = RequestBody::Json(r#"{"name":"test"}"#.to_string());
    let result = export(&req, None);
    assert!(result.contains(r#"-d '{"name":"test"}'"#));
}

#[test]
fn export_with_form_data() {
    let mut req = base_request();
    req.method = HttpMethod::Post;
    req.body = RequestBody::FormData(vec![make_kv("foo", "bar"), make_kv("baz", "qux")]);
    let result = export(&req, None);
    assert!(result.contains("-d 'foo=bar&baz=qux'"));
}

#[test]
fn export_with_environment_interpolation() {
    let mut req = base_request();
    req.url = "https://{{host}}/api".to_string();
    let env = RequestEnvironment {
        id: Uuid::new_v4(),
        name: "test".to_string(),
        variables: vec![make_kv("host", "api.example.com")],
    };
    let result = export(&req, Some(&env));
    assert!(result.contains("https://api.example.com/api"));
    assert!(!result.contains("{{host}}"));
}

#[test]
fn export_disabled_header_excluded() {
    let mut req = base_request();
    req.headers = vec![KeyValuePair {
        id: Uuid::new_v4(),
        key: "X-Skip".to_string(),
        value: "yes".to_string(),
        is_enabled: false,
        is_secret: false,
    }];
    let result = export(&req, None);
    assert!(!result.contains("X-Skip"));
}

#[test]
fn export_body_with_single_quotes_escaped() {
    let mut req = base_request();
    req.method = HttpMethod::Post;
    req.body = RequestBody::Json(r#"{"msg":"it's a test"}"#.to_string());
    let result = export(&req, None);
    assert!(result.contains(r#"-d '{"msg":"it'\''s a test"}'"#));
}

#[test]
fn export_header_with_single_quotes_escaped() {
    let mut req = base_request();
    req.headers = vec![make_kv("X-Msg", "it's")];
    let result = export(&req, None);
    assert!(result.contains(r#"-H 'X-Msg: it'\''s'"#));
}

#[test]
fn export_disabled_param_excluded() {
    let mut req = base_request();
    req.query_params = vec![KeyValuePair {
        id: Uuid::new_v4(),
        key: "skip".to_string(),
        value: "true".to_string(),
        is_enabled: false,
        is_secret: false,
    }];
    let result = export(&req, None);
    assert!(!result.contains("skip=true"));
}

#[test]
fn export_bearer_token_auth() {
    let mut req = base_request();
    req.auth = AuthConfig::BearerToken {
        token: "my-token".to_string(),
    };
    let result = export(&req, None);
    assert!(result.contains("-H 'Authorization: Bearer my-token'"));
}

#[test]
fn export_basic_auth() {
    let mut req = base_request();
    req.auth = AuthConfig::BasicAuth {
        username: "user".to_string(),
        password: "pass".to_string(),
    };
    let result = export(&req, None);
    assert!(result.contains("-u 'user:pass'"));
}

#[test]
fn export_api_key_header_auth() {
    let mut req = base_request();
    req.auth = AuthConfig::ApiKey {
        key: "X-API-Key".to_string(),
        value: "secret".to_string(),
        location: ApiKeyLocation::Header,
    };
    let result = export(&req, None);
    assert!(result.contains("-H 'X-API-Key: secret'"));
}

#[test]
fn export_api_key_query_auth() {
    let mut req = base_request();
    req.auth = AuthConfig::ApiKey {
        key: "api_key".to_string(),
        value: "abc123".to_string(),
        location: ApiKeyLocation::Query,
    };
    let result = export(&req, None);
    assert!(result.contains("api_key=abc123"));
}
