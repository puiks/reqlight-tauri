use super::*;
use crate::models::environment::RequestEnvironment;
use crate::models::request::KeyValuePair;
use std::fs;

#[test]
fn keychain_key_format_uses_uppercased_uuids() {
    let env_id = uuid::Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap();
    let var_id = uuid::Uuid::parse_str("11111111-2222-3333-4444-555555555555").unwrap();

    let key = keychain_key(&env_id, &var_id);

    assert_eq!(
        key,
        "env_A1B2C3D4-E5F6-7890-ABCD-EF1234567890_11111111-2222-3333-4444-555555555555"
    );
}

#[test]
fn load_from_nonexistent_file_returns_default() {
    let path = Path::new("/tmp/reqlight-test-nonexistent/data.json");
    let state = load_state_from_path(path).unwrap();

    assert!(state.collections.is_empty());
    assert!(state.environments.is_empty());
    assert!(state.history.is_empty());
    assert!(state.active_environment_id.is_none());
}

#[test]
fn save_and_load_round_trip_preserves_data() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.json");

    let mut state = AppState::default();
    state.environments.push(RequestEnvironment {
        id: uuid::Uuid::new_v4(),
        name: "Production".to_string(),
        variables: vec![KeyValuePair {
            id: uuid::Uuid::new_v4(),
            key: "BASE_URL".to_string(),
            value: "https://api.example.com".to_string(),
            is_enabled: true,
            is_secret: false,
        }],
    });

    save_state_to_path(&path, &state).unwrap();
    let loaded = load_state_from_path(&path).unwrap();

    assert_eq!(loaded.environments.len(), 1);
    assert_eq!(loaded.environments[0].name, "Production");
    assert_eq!(loaded.environments[0].variables.len(), 1);
    assert_eq!(loaded.environments[0].variables[0].key, "BASE_URL");
    assert_eq!(
        loaded.environments[0].variables[0].value,
        "https://api.example.com"
    );
}

#[test]
fn sanitize_secrets_strips_secret_values() {
    let env_id = uuid::Uuid::new_v4();
    let secret_var_id = uuid::Uuid::new_v4();
    let normal_var_id = uuid::Uuid::new_v4();

    let mut state = AppState::default();
    state.environments.push(RequestEnvironment {
        id: env_id,
        name: "Test".to_string(),
        variables: vec![
            KeyValuePair {
                id: secret_var_id,
                key: "API_KEY".to_string(),
                value: "super-secret-123".to_string(),
                is_enabled: true,
                is_secret: true,
            },
            KeyValuePair {
                id: normal_var_id,
                key: "HOST".to_string(),
                value: "localhost".to_string(),
                is_enabled: true,
                is_secret: false,
            },
        ],
    });

    let (sanitized, secrets) = sanitize_secrets(&state);

    // Secret value should be stripped
    assert_eq!(sanitized.environments[0].variables[0].value, "");
    // Normal value should be preserved
    assert_eq!(sanitized.environments[0].variables[1].value, "localhost");
    // Secret should be returned for keychain storage
    assert_eq!(secrets.len(), 1);
    assert_eq!(secrets[0].1, "super-secret-123");
    assert!(secrets[0].0.contains(&env_id.to_string().to_uppercase()));
}

#[test]
fn secret_values_not_written_to_json_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.json");

    let mut state = AppState::default();
    state.environments.push(RequestEnvironment {
        id: uuid::Uuid::new_v4(),
        name: "Secrets Test".to_string(),
        variables: vec![KeyValuePair {
            id: uuid::Uuid::new_v4(),
            key: "TOKEN".to_string(),
            value: "my-secret-token".to_string(),
            is_enabled: true,
            is_secret: true,
        }],
    });

    // Sanitize and save (simulating what save_state does, minus keychain)
    let (sanitized, _secrets) = sanitize_secrets(&state);
    save_state_to_path(&path, &sanitized).unwrap();

    // Read the raw JSON and verify the secret is not present
    let raw_json = fs::read_to_string(&path).unwrap();
    assert!(
        !raw_json.contains("my-secret-token"),
        "Secret value should NOT appear in the JSON file"
    );
    // The key name should still be there
    assert!(raw_json.contains("TOKEN"));
}

#[test]
fn sanitize_secrets_ignores_empty_secret_values() {
    let mut state = AppState::default();
    state.environments.push(RequestEnvironment {
        id: uuid::Uuid::new_v4(),
        name: "Test".to_string(),
        variables: vec![KeyValuePair {
            id: uuid::Uuid::new_v4(),
            key: "EMPTY_SECRET".to_string(),
            value: String::new(),
            is_enabled: true,
            is_secret: true,
        }],
    });

    let (_sanitized, secrets) = sanitize_secrets(&state);

    // Empty secret should not be stored in keychain
    assert!(secrets.is_empty());
}

#[test]
fn sanitize_secrets_strips_oauth2_tokens() {
    use crate::models::auth::AuthConfig;
    use crate::models::collection::RequestCollection;
    use crate::models::request::SavedRequest;

    let col_id = uuid::Uuid::new_v4();
    let req_id = uuid::Uuid::new_v4();

    let mut state = AppState::default();
    let mut request = SavedRequest::default();
    request.id = req_id;
    request.auth = AuthConfig::OAuth2 {
        grant_type: "client_credentials".to_string(),
        client_id: "my-app".to_string(),
        client_secret: "super-secret".to_string(),
        auth_url: "https://auth.example.com".to_string(),
        token_url: "https://token.example.com".to_string(),
        scopes: "read".to_string(),
        access_token: "tok_abc123".to_string(),
        refresh_token: "ref_xyz789".to_string(),
        token_expiry: None,
    };

    state.collections.push(RequestCollection {
        id: col_id,
        name: "Test".to_string(),
        requests: vec![request],
        sort_order: 0,
        created_at: "2026-01-01T00:00:00Z".to_string(),
    });

    let (sanitized, secrets) = sanitize_secrets(&state);

    // OAuth2 tokens should be stripped
    if let AuthConfig::OAuth2 {
        access_token,
        refresh_token,
        client_secret,
        client_id,
        ..
    } = &sanitized.collections[0].requests[0].auth
    {
        assert_eq!(access_token, "");
        assert_eq!(refresh_token, "");
        assert_eq!(client_secret, "");
        // client_id is NOT secret, should be preserved
        assert_eq!(client_id, "my-app");
    } else {
        panic!("Expected OAuth2 auth config");
    }

    // 3 secrets: access_token, refresh_token, client_secret
    assert_eq!(secrets.len(), 3);
    let secret_values: Vec<&str> = secrets.iter().map(|(_, v)| v.as_str()).collect();
    assert!(secret_values.contains(&"tok_abc123"));
    assert!(secret_values.contains(&"ref_xyz789"));
    assert!(secret_values.contains(&"super-secret"));
}

#[test]
fn oauth2_secrets_not_written_to_json_file() {
    use crate::models::auth::AuthConfig;
    use crate::models::collection::RequestCollection;
    use crate::models::request::SavedRequest;

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.json");

    let mut state = AppState::default();
    let mut request = SavedRequest::default();
    request.auth = AuthConfig::OAuth2 {
        grant_type: "client_credentials".to_string(),
        client_id: "my-app".to_string(),
        client_secret: "the-client-secret".to_string(),
        auth_url: "".to_string(),
        token_url: "".to_string(),
        scopes: "".to_string(),
        access_token: "secret-access-token".to_string(),
        refresh_token: "secret-refresh-token".to_string(),
        token_expiry: None,
    };
    state.collections.push(RequestCollection {
        id: uuid::Uuid::new_v4(),
        name: "Test".to_string(),
        requests: vec![request],
        sort_order: 0,
        created_at: "2026-01-01T00:00:00Z".to_string(),
    });

    let (sanitized, _) = sanitize_secrets(&state);
    save_state_to_path(&path, &sanitized).unwrap();

    let raw_json = fs::read_to_string(&path).unwrap();
    assert!(
        !raw_json.contains("secret-access-token"),
        "access_token should NOT appear in JSON"
    );
    assert!(
        !raw_json.contains("secret-refresh-token"),
        "refresh_token should NOT appear in JSON"
    );
    assert!(
        !raw_json.contains("the-client-secret"),
        "client_secret should NOT appear in JSON"
    );
    // Non-secret fields should still be present
    assert!(raw_json.contains("my-app"));
}

#[test]
fn load_from_invalid_json_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.json");
    fs::write(&path, "not valid json{{{").unwrap();

    let result = load_state_from_path(&path);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to parse data file"));
}
