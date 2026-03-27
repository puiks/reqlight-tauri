use crate::models::{
    ApiKeyLocation, AuthConfig, HttpMethod, KeyValuePair, RequestBody, RequestEnvironment,
    SavedRequest,
};
use crate::services::interpolator;

/// Export a SavedRequest as a cURL command string.
/// If an environment is provided, variables are interpolated first.
pub fn export(request: &SavedRequest, environment: Option<&RequestEnvironment>) -> String {
    let (url, headers, params, body, auth) = if let Some(env) = environment {
        let (u, h, p, b) = interpolator::interpolate_request(
            &request.url,
            &request.headers,
            &request.query_params,
            &request.body,
            &env.variables,
        );
        let a = interpolator::interpolate_auth(&request.auth, &env.variables);
        (u, h, p, b, a)
    } else {
        (
            request.url.clone(),
            request.headers.clone(),
            request.query_params.clone(),
            request.body.clone(),
            request.auth.clone(),
        )
    };

    let mut parts: Vec<String> = vec!["curl".to_string()];

    if request.method != HttpMethod::Get {
        parts.push(format!("-X {}", request.method.as_str()));
    }

    // URL with query params
    let mut url_string = url;
    let enabled_params: Vec<&KeyValuePair> = params
        .iter()
        .filter(|p| p.is_enabled && !p.key.is_empty())
        .collect();
    if !enabled_params.is_empty() {
        let qs: String = enabled_params
            .iter()
            .map(|p| format!("{}={}", percent_encode(&p.key), percent_encode(&p.value)))
            .collect::<Vec<_>>()
            .join("&");
        let sep = if url_string.contains('?') { "&" } else { "?" };
        url_string = format!("{url_string}{sep}{qs}");
    }

    // Auth — process before URL is finalized (ApiKey Query appends to URL)
    match &auth {
        AuthConfig::ApiKey {
            key,
            value,
            location: ApiKeyLocation::Query,
        } if !key.is_empty() => {
            let sep = if url_string.contains('?') { "&" } else { "?" };
            url_string = format!(
                "{url_string}{sep}{}={}",
                percent_encode(key),
                percent_encode(value)
            );
        }
        _ => {}
    }

    parts.push(format!("'{}'", shell_escape(&url_string)));

    // Headers
    for h in headers.iter().filter(|h| h.is_enabled && !h.key.is_empty()) {
        parts.push(format!(
            "-H '{}'",
            shell_escape(&format!("{}: {}", h.key, h.value))
        ));
    }

    // Auth headers
    match &auth {
        AuthConfig::BearerToken { token } if !token.is_empty() => {
            parts.push(format!(
                "-H '{}'",
                shell_escape(&format!("Authorization: Bearer {token}"))
            ));
        }
        AuthConfig::BasicAuth { username, password } => {
            parts.push(format!(
                "-u '{}'",
                shell_escape(&format!("{username}:{password}"))
            ));
        }
        AuthConfig::ApiKey {
            key,
            value,
            location: ApiKeyLocation::Header,
        } if !key.is_empty() => {
            parts.push(format!("-H '{}'", shell_escape(&format!("{key}: {value}"))));
        }
        _ => {}
    }

    // Body
    match &body {
        RequestBody::Json(s) => {
            parts.push(format!("-d '{}'", shell_escape(s)));
        }
        RequestBody::RawText(s) => {
            parts.push(format!("-d '{}'", shell_escape(s)));
        }
        RequestBody::FormData(pairs) => {
            let encoded: String = pairs
                .iter()
                .filter(|p| p.is_enabled && !p.key.is_empty())
                .map(|p| format!("{}={}", percent_encode(&p.key), percent_encode(&p.value)))
                .collect::<Vec<_>>()
                .join("&");
            parts.push(format!("-d '{}'", shell_escape(&encoded)));
        }
        RequestBody::None => {}
    }

    parts.join(" \\\n  ")
}

fn percent_encode(s: &str) -> String {
    urlencoding::encode(s).into_owned()
}

/// Escape a string for safe embedding inside single quotes in a shell command.
/// Single quotes cannot be escaped inside single quotes, so we break out of the
/// single-quoted string, add an escaped single quote, then re-enter.
/// e.g. "it's" → 'it'\''s'
fn shell_escape(s: &str) -> String {
    s.replace('\'', "'\\''")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        ApiKeyLocation, AuthConfig, HttpMethod, KeyValuePair, RequestBody, SavedRequest,
    };
    use uuid::Uuid;

    fn make_kv(key: &str, value: &str) -> KeyValuePair {
        KeyValuePair {
            id: Uuid::new_v4(),
            key: key.to_string(),
            value: value.to_string(),
            is_enabled: true,
            is_secret: false,
        }
    }

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
}
