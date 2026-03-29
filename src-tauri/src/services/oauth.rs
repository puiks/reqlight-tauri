use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::Deserialize;
use sha2::{Digest, Sha256};

/// OAuth2 token response from the token endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub expires_in: Option<u64>,
    #[serde(default)]
    #[allow(dead_code)]
    pub token_type: Option<String>,
}

/// PKCE code verifier and challenge pair.
pub struct PkceChallenge {
    pub code_verifier: String,
    pub code_challenge: String,
}

/// Generate a PKCE code verifier (128 random bytes, base64url-encoded)
/// and its corresponding S256 challenge.
pub fn generate_pkce_challenge() -> PkceChallenge {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Generate 32 random bytes using a simple approach (uuid + timestamp mixing)
    let id1 = uuid::Uuid::new_v4();
    let id2 = uuid::Uuid::new_v4();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();

    let mut bytes = Vec::with_capacity(48);
    bytes.extend_from_slice(id1.as_bytes());
    bytes.extend_from_slice(id2.as_bytes());
    bytes.extend_from_slice(&nanos.to_le_bytes());

    let code_verifier = URL_SAFE_NO_PAD.encode(&bytes);
    let code_challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()));

    PkceChallenge {
        code_verifier,
        code_challenge,
    }
}

/// Exchange client credentials for a token (Client Credentials Grant).
#[tracing::instrument(skip(client, client_secret))]
pub async fn client_credentials_exchange(
    client: &reqwest::Client,
    token_url: &str,
    client_id: &str,
    client_secret: &str,
    scopes: &str,
) -> Result<TokenResponse, String> {
    let mut params = vec![
        ("grant_type", "client_credentials"),
        ("client_id", client_id),
        ("client_secret", client_secret),
    ];
    if !scopes.is_empty() {
        params.push(("scope", scopes));
    }

    let resp = client
        .post(token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_else(|e| {
            tracing::warn!("Failed to read token error response body: {e}");
            format!("<failed to read body: {e}>")
        });
        return Err(format!("Token endpoint returned {status}: {body}"));
    }

    resp.json::<TokenResponse>()
        .await
        .map_err(|e| format!("Failed to parse token response: {e}"))
}

/// Exchange an authorization code for a token (Authorization Code Grant).
/// When PKCE is used, `code_verifier` must be provided.
#[tracing::instrument(skip(client, client_secret, code_verifier))]
pub async fn authorization_code_exchange(
    client: &reqwest::Client,
    token_url: &str,
    code: &str,
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
    code_verifier: Option<&str>,
) -> Result<TokenResponse, String> {
    let mut params = vec![
        ("grant_type", "authorization_code"),
        ("code", code),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("redirect_uri", redirect_uri),
    ];
    if let Some(verifier) = code_verifier {
        params.push(("code_verifier", verifier));
    }

    let resp = client
        .post(token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_else(|e| {
            tracing::warn!("Failed to read token error response body: {e}");
            format!("<failed to read body: {e}>")
        });
        return Err(format!("Token endpoint returned {status}: {body}"));
    }

    resp.json::<TokenResponse>()
        .await
        .map_err(|e| format!("Failed to parse token response: {e}"))
}

/// Refresh an expired token.
#[tracing::instrument(skip(client, refresh_token, client_secret))]
pub async fn refresh_token_exchange(
    client: &reqwest::Client,
    token_url: &str,
    refresh_token: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<TokenResponse, String> {
    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", client_id),
        ("client_secret", client_secret),
    ];

    let resp = client
        .post(token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token refresh failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_else(|e| {
            eprintln!("Warning: failed to read token refresh error response body: {e}");
            format!("<failed to read body: {e}>")
        });
        return Err(format!("Token refresh error {status}: {body}"));
    }

    resp.json::<TokenResponse>()
        .await
        .map_err(|e| format!("Failed to parse refresh response: {e}"))
}

/// Result of extracting authorization callback parameters.
#[derive(Debug)]
pub struct AuthCallbackResult {
    pub code: String,
    pub state: Option<String>,
}

/// Extract an authorization code and state from an HTTP request string.
/// Parses "GET /callback?code=xxx&state=yyy HTTP/1.1" format.
pub fn extract_code_from_request(request: &str) -> Result<AuthCallbackResult, String> {
    let first_line = request.lines().next().unwrap_or("");
    let path = first_line.split_whitespace().nth(1).unwrap_or("");

    if let Some(query) = path.split('?').nth(1) {
        let mut code = None;
        let mut state = None;

        for param in query.split('&') {
            let mut kv = param.splitn(2, '=');
            let key = kv.next().unwrap_or("");
            let val = kv.next().unwrap_or("");
            match key {
                "error" => {
                    let desc = query
                        .split('&')
                        .find_map(|p| {
                            let mut kv = p.splitn(2, '=');
                            if kv.next() == Some("error_description") {
                                kv.next()
                                    .map(|v| urlencoding::decode(v).unwrap_or_default().to_string())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| val.to_string());
                    return Err(format!("Authorization error: {desc}"));
                }
                "code" => {
                    code = Some(urlencoding::decode(val).unwrap_or_default().to_string());
                }
                "state" => {
                    state = Some(urlencoding::decode(val).unwrap_or_default().to_string());
                }
                _ => {}
            }
        }

        if let Some(code) = code {
            return Ok(AuthCallbackResult { code, state });
        }
    }

    Err("No authorization code received in callback".to_string())
}

#[cfg(test)]
#[path = "oauth_tests.rs"]
mod tests;
