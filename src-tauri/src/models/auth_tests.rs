use super::*;

#[test]
fn serde_none_roundtrip() {
    let auth = AuthConfig::None;
    let json = serde_json::to_string(&auth).unwrap();
    let parsed: AuthConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, AuthConfig::None);
}

#[test]
fn serde_bearer_roundtrip() {
    let auth = AuthConfig::BearerToken {
        token: "my-secret-token".to_string(),
    };
    let json = serde_json::to_string(&auth).unwrap();
    assert!(json.contains("bearerToken"));
    let parsed: AuthConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, auth);
}

#[test]
fn serde_basic_auth_roundtrip() {
    let auth = AuthConfig::BasicAuth {
        username: "admin".to_string(),
        password: "p@ss".to_string(),
    };
    let json = serde_json::to_string(&auth).unwrap();
    assert!(json.contains("basicAuth"));
    let parsed: AuthConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, auth);
}

#[test]
fn serde_api_key_header_roundtrip() {
    let auth = AuthConfig::ApiKey {
        key: "X-API-Key".to_string(),
        value: "abc123".to_string(),
        location: ApiKeyLocation::Header,
    };
    let json = serde_json::to_string(&auth).unwrap();
    assert!(json.contains("apiKey"));
    let parsed: AuthConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, auth);
}

#[test]
fn serde_api_key_query_roundtrip() {
    let auth = AuthConfig::ApiKey {
        key: "api_key".to_string(),
        value: "xyz".to_string(),
        location: ApiKeyLocation::Query,
    };
    let json = serde_json::to_string(&auth).unwrap();
    let parsed: AuthConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, auth);
}

#[test]
fn backward_compatible_missing_auth_defaults_to_none() {
    // Simulates existing SavedRequest JSON without "auth" field
    let json = r#"{"none":{}}"#;
    let parsed: AuthConfig = serde_json::from_str(json).unwrap();
    assert_eq!(parsed, AuthConfig::None);
}

#[test]
fn serde_oauth2_roundtrip() {
    let auth = AuthConfig::OAuth2 {
        grant_type: "authorization_code".to_string(),
        client_id: "my-app".to_string(),
        client_secret: "secret123".to_string(),
        auth_url: "https://auth.example.com/authorize".to_string(),
        token_url: "https://auth.example.com/token".to_string(),
        scopes: "read write".to_string(),
        access_token: "tok_abc".to_string(),
        refresh_token: "ref_xyz".to_string(),
        token_expiry: Some("2026-04-01T00:00:00Z".to_string()),
    };
    let json = serde_json::to_string(&auth).unwrap();
    assert!(json.contains("oauth2"));
    assert!(json.contains("grantType"));
    let parsed: AuthConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, auth);
}

#[test]
fn unknown_variant_defaults_to_none() {
    let json = r#"{"unknownType":{"_0":{}}}"#;
    let parsed: AuthConfig = serde_json::from_str(json).unwrap();
    assert_eq!(parsed, AuthConfig::None);
}
