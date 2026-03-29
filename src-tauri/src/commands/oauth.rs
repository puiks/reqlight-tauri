use crate::constants::OAUTH_CALLBACK_TIMEOUT_SECS;
use crate::services::oauth;
use crate::SharedHttpClient;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthTokenResult {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: Option<u64>,
}

/// IPC: Exchange client credentials for a token.
#[tauri::command]
pub async fn oauth_client_credentials(
    client: State<'_, SharedHttpClient>,
    token_url: String,
    client_id: String,
    client_secret: String,
    scopes: String,
) -> Result<OAuthTokenResult, String> {
    let resp = oauth::client_credentials_exchange(
        &client.0,
        &token_url,
        &client_id,
        &client_secret,
        &scopes,
    )
    .await?;

    Ok(OAuthTokenResult {
        access_token: resp.access_token,
        refresh_token: resp.refresh_token.unwrap_or_default(),
        expires_in: resp.expires_in,
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthCodeFlowParams {
    pub auth_url: String,
    pub token_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub scopes: String,
}

/// IPC: Start the Authorization Code flow with PKCE.
/// Opens the system browser and waits for the callback.
#[tauri::command]
pub async fn oauth_authorization_code(
    client: State<'_, SharedHttpClient>,
    params: AuthCodeFlowParams,
) -> Result<OAuthTokenResult, String> {
    // Generate PKCE challenge
    let pkce = oauth::generate_pkce_challenge();

    // Generate state parameter for CSRF protection
    let state = uuid::Uuid::new_v4().to_string();

    // Start local callback server
    let (redirect_uri, code) = {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|e| format!("Failed to start callback server: {e}"))?;
        let port = listener
            .local_addr()
            .map_err(|e| format!("Failed to get address: {e}"))?
            .port();
        let redirect_uri = format!("http://127.0.0.1:{port}/callback");

        // Build authorization URL with PKCE and state
        let auth_url = build_auth_url(
            &params.auth_url,
            &params.client_id,
            &redirect_uri,
            &params.scopes,
            &pkce.code_challenge,
            &state,
        );

        // Open browser
        open::that(&auth_url).map_err(|e| format!("Failed to open browser: {e}"))?;

        // Wait for callback (120s timeout)
        let expected_state = state.clone();
        let accept_future = async {
            let (stream, _) = listener
                .accept()
                .await
                .map_err(|e| format!("Accept error: {e}"))?;

            let mut buf = vec![0u8; 4096];
            stream
                .readable()
                .await
                .map_err(|e| format!("Read error: {e}"))?;
            let n = stream
                .try_read(&mut buf)
                .map_err(|e| format!("Read error: {e}"))?;
            let request_str = String::from_utf8_lossy(&buf[..n]);

            let result = oauth::extract_code_from_request(&request_str)?;

            // Validate state parameter to prevent CSRF
            match result.state.as_deref() {
                Some(returned_state) if returned_state == expected_state => {}
                Some(_) => {
                    return Err(
                        "State mismatch: possible CSRF attack. Please try again.".to_string()
                    );
                }
                None => {
                    return Err("Missing state parameter in callback".to_string());
                }
            }

            let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n\
                <html><body><h2>Authorization successful!</h2>\
                <p>You can close this tab and return to Reqlight.</p>\
                <script>window.close()</script></body></html>";
            let _ = stream.writable().await;
            let _ = stream.try_write(response.as_bytes());

            Ok::<String, String>(result.code)
        };

        let code = tokio::time::timeout(
            std::time::Duration::from_secs(OAUTH_CALLBACK_TIMEOUT_SECS),
            accept_future,
        )
        .await
        .map_err(|_| {
            format!("Authorization timed out ({OAUTH_CALLBACK_TIMEOUT_SECS}s). Please try again.")
        })?
        .map_err(|e: String| e)?;

        (redirect_uri, code)
    };

    // Exchange code for token with PKCE code_verifier
    let resp = oauth::authorization_code_exchange(
        &client.0,
        &params.token_url,
        &code,
        &params.client_id,
        &params.client_secret,
        &redirect_uri,
        Some(&pkce.code_verifier),
    )
    .await?;

    Ok(OAuthTokenResult {
        access_token: resp.access_token,
        refresh_token: resp.refresh_token.unwrap_or_default(),
        expires_in: resp.expires_in,
    })
}

/// IPC: Refresh an expired token.
#[tauri::command]
pub async fn oauth_refresh_token(
    client: State<'_, SharedHttpClient>,
    token_url: String,
    refresh_token: String,
    client_id: String,
    client_secret: String,
) -> Result<OAuthTokenResult, String> {
    let resp = oauth::refresh_token_exchange(
        &client.0,
        &token_url,
        &refresh_token,
        &client_id,
        &client_secret,
    )
    .await?;

    Ok(OAuthTokenResult {
        access_token: resp.access_token,
        refresh_token: resp.refresh_token.unwrap_or_default(),
        expires_in: resp.expires_in,
    })
}

fn build_auth_url(
    auth_url: &str,
    client_id: &str,
    redirect_uri: &str,
    scopes: &str,
    code_challenge: &str,
    state: &str,
) -> String {
    let mut url = format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&code_challenge={}&code_challenge_method=S256&state={}",
        auth_url,
        urlencoding::encode(client_id),
        urlencoding::encode(redirect_uri),
        urlencoding::encode(code_challenge),
        urlencoding::encode(state),
    );
    if !scopes.is_empty() {
        url.push_str(&format!("&scope={}", urlencoding::encode(scopes)));
    }
    url
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_auth_url_includes_all_required_params() {
        let url = build_auth_url(
            "https://auth.example.com/authorize",
            "my-client",
            "http://127.0.0.1:8080/callback",
            "openid profile",
            "abc123challenge",
            "state-xyz",
        );

        assert!(url.starts_with("https://auth.example.com/authorize?"));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("client_id=my-client"));
        assert!(url.contains("redirect_uri=http%3A%2F%2F127.0.0.1%3A8080%2Fcallback"));
        assert!(url.contains("code_challenge=abc123challenge"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("state=state-xyz"));
        assert!(url.contains("scope=openid%20profile"));
    }

    #[test]
    fn build_auth_url_omits_scope_when_empty() {
        let url = build_auth_url(
            "https://auth.example.com/authorize",
            "client-id",
            "http://localhost/cb",
            "",
            "challenge",
            "state",
        );

        assert!(!url.contains("scope="));
    }

    #[test]
    fn build_auth_url_encodes_special_characters() {
        let url = build_auth_url(
            "https://auth.example.com/authorize",
            "client with spaces",
            "http://localhost/cb",
            "read write",
            "ch+all/enge=",
            "state&param",
        );

        assert!(url.contains("client_id=client%20with%20spaces"));
        assert!(url.contains("scope=read%20write"));
        assert!(url.contains("code_challenge=ch%2Ball%2Fenge%3D"));
        assert!(url.contains("state=state%26param"));
    }
}
