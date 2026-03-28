use serde::Deserialize;

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

/// Exchange client credentials for a token (Client Credentials Grant).
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
            eprintln!("Warning: failed to read token error response body: {e}");
            format!("<failed to read body: {e}>")
        });
        return Err(format!("Token endpoint returned {status}: {body}"));
    }

    resp.json::<TokenResponse>()
        .await
        .map_err(|e| format!("Failed to parse token response: {e}"))
}

/// Exchange an authorization code for a token (Authorization Code Grant).
pub async fn authorization_code_exchange(
    client: &reqwest::Client,
    token_url: &str,
    code: &str,
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
) -> Result<TokenResponse, String> {
    let params = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("redirect_uri", redirect_uri),
    ];

    let resp = client
        .post(token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_else(|e| {
            eprintln!("Warning: failed to read token error response body: {e}");
            format!("<failed to read body: {e}>")
        });
        return Err(format!("Token endpoint returned {status}: {body}"));
    }

    resp.json::<TokenResponse>()
        .await
        .map_err(|e| format!("Failed to parse token response: {e}"))
}

/// Refresh an expired token.
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

/// Extract an authorization code from an HTTP request string.
/// Parses "GET /callback?code=xxx&state=yyy HTTP/1.1" format.
pub fn extract_code_from_request(request: &str) -> Result<String, String> {
    // Parse "GET /callback?code=xxx&state=yyy HTTP/1.1"
    let first_line = request.lines().next().unwrap_or("");
    let path = first_line.split_whitespace().nth(1).unwrap_or("");

    // Check for error response
    if let Some(query) = path.split('?').nth(1) {
        for param in query.split('&') {
            let mut kv = param.splitn(2, '=');
            let key = kv.next().unwrap_or("");
            let val = kv.next().unwrap_or("");
            if key == "error" {
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
            if key == "code" {
                return Ok(urlencoding::decode(val).unwrap_or_default().to_string());
            }
        }
    }

    Err("No authorization code received in callback".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_code_from_valid_request() {
        let req = "GET /callback?code=abc123&state=xyz HTTP/1.1\r\nHost: 127.0.0.1";
        let code = extract_code_from_request(req).unwrap();
        assert_eq!(code, "abc123");
    }

    #[test]
    fn extract_code_url_decoded() {
        let req = "GET /callback?code=abc%20123 HTTP/1.1\r\n";
        let code = extract_code_from_request(req).unwrap();
        assert_eq!(code, "abc 123");
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
}
