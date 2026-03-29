use super::*;

#[test]
fn extract_code_from_valid_request() {
    let req = "GET /callback?code=abc123&state=xyz HTTP/1.1\r\nHost: 127.0.0.1";
    let result = extract_code_from_request(req).unwrap();
    assert_eq!(result.code, "abc123");
    assert_eq!(result.state.as_deref(), Some("xyz"));
}

#[test]
fn extract_code_url_decoded() {
    let req = "GET /callback?code=abc%20123 HTTP/1.1\r\n";
    let result = extract_code_from_request(req).unwrap();
    assert_eq!(result.code, "abc 123");
    assert!(result.state.is_none());
}

#[test]
fn extract_code_error_response() {
    let req = "GET /callback?error=access_denied&error_description=User%20denied HTTP/1.1\r\n";
    let result = extract_code_from_request(req);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("User denied"));
}

#[test]
fn extract_code_missing_code() {
    let req = "GET /callback?state=xyz HTTP/1.1\r\n";
    let result = extract_code_from_request(req);
    assert!(result.is_err());
}

#[test]
fn pkce_challenge_generates_valid_pair() {
    let pkce = generate_pkce_challenge();
    // Verifier should be non-empty base64url
    assert!(!pkce.code_verifier.is_empty());
    assert!(pkce.code_verifier.len() >= 43);
    // Challenge should be S256 of verifier
    let expected = URL_SAFE_NO_PAD.encode(Sha256::digest(pkce.code_verifier.as_bytes()));
    assert_eq!(pkce.code_challenge, expected);
}

#[test]
fn pkce_challenges_are_unique() {
    let a = generate_pkce_challenge();
    let b = generate_pkce_challenge();
    assert_ne!(a.code_verifier, b.code_verifier);
}

#[test]
fn extract_code_with_state_validation() {
    let req = "GET /callback?code=mycode&state=expected_state HTTP/1.1\r\n";
    let result = extract_code_from_request(req).unwrap();
    assert_eq!(result.code, "mycode");
    assert_eq!(result.state.as_deref(), Some("expected_state"));
}

#[test]
fn extract_code_no_query_string() {
    let req = "GET /callback HTTP/1.1\r\n";
    let result = extract_code_from_request(req);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No authorization code"));
}

#[test]
fn extract_code_empty_request() {
    let result = extract_code_from_request("");
    assert!(result.is_err());
}

#[test]
fn extract_code_error_without_description() {
    let req = "GET /callback?error=server_error HTTP/1.1\r\n";
    let result = extract_code_from_request(req);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("server_error"));
}

#[tokio::test]
async fn client_credentials_exchange_http_error() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/token")
        .with_status(401)
        .with_body(r#"{"error":"invalid_client"}"#)
        .create_async()
        .await;

    let client = reqwest::Client::new();
    let result = client_credentials_exchange(
        &client,
        &format!("{}/token", server.url()),
        "id",
        "secret",
        "",
    )
    .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("401"), "got: {err}");
    mock.assert_async().await;
}

#[tokio::test]
async fn client_credentials_exchange_invalid_json() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/token")
        .with_status(200)
        .with_body("not json at all")
        .create_async()
        .await;

    let client = reqwest::Client::new();
    let result = client_credentials_exchange(
        &client,
        &format!("{}/token", server.url()),
        "id",
        "secret",
        "",
    )
    .await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("Failed to parse token response"));
    mock.assert_async().await;
}

#[tokio::test]
async fn refresh_token_exchange_http_error() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/token")
        .with_status(400)
        .with_body(r#"{"error":"invalid_grant"}"#)
        .create_async()
        .await;

    let client = reqwest::Client::new();
    let result = refresh_token_exchange(
        &client,
        &format!("{}/token", server.url()),
        "old_refresh",
        "id",
        "secret",
    )
    .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("400"), "got: {err}");
    mock.assert_async().await;
}

#[tokio::test]
async fn authorization_code_exchange_with_pkce() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"access_token":"at_123","refresh_token":"rt_456","expires_in":3600}"#)
        .create_async()
        .await;

    let client = reqwest::Client::new();
    let result = authorization_code_exchange(
        &client,
        &format!("{}/token", server.url()),
        "auth_code_xyz",
        "client-id",
        "client-secret",
        "http://localhost/callback",
        Some("my_code_verifier"),
    )
    .await;

    assert!(result.is_ok());
    let token = result.unwrap();
    assert_eq!(token.access_token, "at_123");
    assert_eq!(token.refresh_token.as_deref(), Some("rt_456"));
    assert_eq!(token.expires_in, Some(3600));
    mock.assert_async().await;
}
