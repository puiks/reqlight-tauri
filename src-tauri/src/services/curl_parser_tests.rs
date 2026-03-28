use super::*;
use crate::models::HttpMethod;

#[test]
fn parse_simple_get() {
    let req = parse("curl https://example.com").unwrap();
    assert_eq!(req.url, "https://example.com");
    assert_eq!(req.method, HttpMethod::Get);
}

#[test]
fn parse_explicit_method() {
    let req = parse("curl -X POST https://example.com").unwrap();
    assert_eq!(req.method, HttpMethod::Post);
}

#[test]
fn parse_headers() {
    let req = parse(r#"curl -H "Content-Type: application/json" -H "Authorization: Bearer token" https://example.com"#).unwrap();
    // Authorization: Bearer is extracted as auth config, not a header
    assert_eq!(req.headers.len(), 1);
    assert_eq!(req.headers[0].key, "Content-Type");
    assert_eq!(req.headers[0].value, "application/json");
    match &req.auth {
        AuthConfig::BearerToken { token } => assert_eq!(token, "token"),
        _ => panic!("Expected BearerToken auth"),
    }
}

#[test]
fn parse_json_body() {
    let req = parse(r#"curl -d '{"name":"test"}' https://example.com"#).unwrap();
    assert_eq!(req.method, HttpMethod::Post); // auto-promoted from GET
    match &req.body {
        RequestBody::Json(s) => assert_eq!(s, r#"{"name":"test"}"#),
        _ => panic!("Expected Json body"),
    }
}

#[test]
fn parse_form_data() {
    let req = parse("curl -d 'foo=bar&baz=qux' https://example.com").unwrap();
    match &req.body {
        RequestBody::FormData(pairs) => {
            assert_eq!(pairs.len(), 2);
            assert_eq!(pairs[0].key, "foo");
            assert_eq!(pairs[0].value, "bar");
            assert_eq!(pairs[1].key, "baz");
            assert_eq!(pairs[1].value, "qux");
        }
        _ => panic!("Expected FormData body"),
    }
}

#[test]
fn parse_raw_text_body() {
    let req = parse("curl -d 'hello world' https://example.com").unwrap();
    match &req.body {
        RequestBody::RawText(s) => assert_eq!(s, "hello world"),
        _ => panic!("Expected RawText body"),
    }
}

#[test]
fn parse_line_continuations() {
    let input = "curl \\\n  -X PUT \\\n  https://example.com";
    let req = parse(input).unwrap();
    assert_eq!(req.method, HttpMethod::Put);
    assert_eq!(req.url, "https://example.com");
}

#[test]
fn parse_get_flag_without_explicit_method() {
    // -G without -X forces GET
    let req = parse("curl -d 'foo=bar' -G https://example.com").unwrap();
    assert_eq!(req.method, HttpMethod::Get);
}

#[test]
fn parse_get_flag_does_not_override_explicit_method() {
    // -X POST takes precedence over -G
    let req = parse("curl -X POST -G https://example.com").unwrap();
    assert_eq!(req.method, HttpMethod::Post);
}

#[test]
fn parse_data_auto_promotes_to_post() {
    // -d without -X should auto-promote to POST
    let req = parse("curl -d 'test' https://example.com").unwrap();
    assert_eq!(req.method, HttpMethod::Post);
}

#[test]
fn parse_data_with_explicit_get_stays_get() {
    // -X GET with -d should keep GET (user explicitly chose it)
    let req = parse("curl -X GET -d 'test' https://example.com").unwrap();
    assert_eq!(req.method, HttpMethod::Get);
}

#[test]
fn parse_empty_returns_error() {
    assert!(parse("").is_err());
}

#[test]
fn parse_no_url_returns_error() {
    assert!(parse("curl -X GET").is_err());
}

#[test]
fn tokenize_handles_single_quotes() {
    let tokens = tokenize("curl -d 'hello world' http://x.com");
    assert_eq!(tokens, vec!["curl", "-d", "hello world", "http://x.com"]);
}

#[test]
fn tokenize_handles_double_quotes() {
    let tokens = tokenize(r#"curl -H "Content-Type: json" http://x.com"#);
    assert_eq!(
        tokens,
        vec!["curl", "-H", "Content-Type: json", "http://x.com"]
    );
}

#[test]
fn tokenize_handles_escaped_chars() {
    let tokens = tokenize(r#"curl -d "hello \"world\"" http://x.com"#);
    assert_eq!(
        tokens,
        vec!["curl", "-d", r#"hello "world""#, "http://x.com"]
    );
}

#[test]
fn parse_basic_auth_flag() {
    let req = parse("curl -u admin:secret https://example.com").unwrap();
    match &req.auth {
        AuthConfig::BasicAuth { username, password } => {
            assert_eq!(username, "admin");
            assert_eq!(password, "secret");
        }
        _ => panic!("Expected BasicAuth"),
    }
}

#[test]
fn parse_bearer_from_header() {
    let req = parse(r#"curl -H "Authorization: Bearer my-token" https://example.com"#).unwrap();
    match &req.auth {
        AuthConfig::BearerToken { token } => {
            assert_eq!(token, "my-token");
        }
        _ => panic!("Expected BearerToken"),
    }
    // Authorization header should NOT be in the headers list
    assert!(req.headers.iter().all(|h| h.key != "Authorization"));
}
