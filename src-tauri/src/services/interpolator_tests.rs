use super::*;
use crate::models::{ApiKeyLocation, AuthConfig};
use crate::test_utils::{make_kv as make_var, make_kv_disabled as make_disabled_var};

#[test]
fn interpolate_simple_variable() {
    let vars = vec![make_var("host", "example.com")];
    assert_eq!(
        interpolate("https://{{host}}/api", &vars),
        "https://example.com/api"
    );
}

#[test]
fn interpolate_multiple_variables() {
    let vars = vec![make_var("host", "example.com"), make_var("version", "v2")];
    assert_eq!(
        interpolate("https://{{host}}/{{version}}/users", &vars),
        "https://example.com/v2/users"
    );
}

#[test]
fn interpolate_unknown_variable_kept_as_is() {
    let vars = vec![make_var("host", "example.com")];
    assert_eq!(
        interpolate("{{host}}/{{unknown}}", &vars),
        "example.com/{{unknown}}"
    );
}

#[test]
fn interpolate_disabled_variable_ignored() {
    let vars = vec![make_disabled_var("host", "example.com")];
    assert_eq!(interpolate("https://{{host}}", &vars), "https://{{host}}");
}

#[test]
fn interpolate_no_variables_returns_original() {
    let vars = vec![make_var("host", "example.com")];
    assert_eq!(interpolate("https://plain.com", &vars), "https://plain.com");
}

#[test]
fn interpolate_empty_string() {
    let vars = vec![make_var("x", "y")];
    assert_eq!(interpolate("", &vars), "");
}

#[test]
fn interpolate_unclosed_braces() {
    let vars = vec![make_var("x", "y")];
    assert_eq!(interpolate("{{x} still here", &vars), "{{x} still here");
}

#[test]
fn interpolate_adjacent_variables() {
    let vars = vec![make_var("a", "hello"), make_var("b", "world")];
    assert_eq!(interpolate("{{a}}{{b}}", &vars), "helloworld");
}

#[test]
fn interpolate_request_replaces_all_parts() {
    let vars = vec![make_var("host", "api.test"), make_var("token", "abc123")];
    let headers = vec![make_var("Authorization", "Bearer {{token}}")];
    let params = vec![make_var("q", "{{host}}")];
    let body = RequestBody::Json(r#"{"host":"{{host}}"}"#.to_string());

    let (url, new_headers, new_params, new_body) =
        interpolate_request("https://{{host}}/api", &headers, &params, &body, &vars);

    assert_eq!(url, "https://api.test/api");
    assert_eq!(new_headers[0].value, "Bearer abc123");
    assert_eq!(new_params[0].value, "api.test");
    match new_body {
        RequestBody::Json(s) => assert_eq!(s, r#"{"host":"api.test"}"#),
        _ => panic!("Expected Json body"),
    }
}

#[test]
fn interpolate_auth_none_unchanged() {
    let vars = vec![make_var("token", "abc")];
    let result = interpolate_auth(&AuthConfig::None, &vars);
    assert_eq!(result, AuthConfig::None);
}

#[test]
fn interpolate_auth_bearer_token() {
    let vars = vec![make_var("token", "secret123")];
    let auth = AuthConfig::BearerToken {
        token: "{{token}}".to_string(),
    };
    let result = interpolate_auth(&auth, &vars);
    assert_eq!(
        result,
        AuthConfig::BearerToken {
            token: "secret123".to_string()
        }
    );
}

#[test]
fn interpolate_auth_basic() {
    let vars = vec![make_var("user", "admin"), make_var("pass", "s3cret")];
    let auth = AuthConfig::BasicAuth {
        username: "{{user}}".to_string(),
        password: "{{pass}}".to_string(),
    };
    let result = interpolate_auth(&auth, &vars);
    assert_eq!(
        result,
        AuthConfig::BasicAuth {
            username: "admin".to_string(),
            password: "s3cret".to_string()
        }
    );
}

#[test]
fn interpolate_auth_api_key() {
    let vars = vec![make_var("key", "my-api-key")];
    let auth = AuthConfig::ApiKey {
        key: "X-API-Key".to_string(),
        value: "{{key}}".to_string(),
        location: ApiKeyLocation::Header,
    };
    let result = interpolate_auth(&auth, &vars);
    assert_eq!(
        result,
        AuthConfig::ApiKey {
            key: "X-API-Key".to_string(),
            value: "my-api-key".to_string(),
            location: ApiKeyLocation::Header,
        }
    );
}
