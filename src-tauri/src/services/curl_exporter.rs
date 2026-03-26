use crate::models::{HttpMethod, KeyValuePair, RequestBody, RequestEnvironment, SavedRequest};
use crate::services::interpolator;

/// Export a SavedRequest as a cURL command string.
/// If an environment is provided, variables are interpolated first.
pub fn export(request: &SavedRequest, environment: Option<&RequestEnvironment>) -> String {
    let (url, headers, params, body) = if let Some(env) = environment {
        interpolator::interpolate_request(
            &request.url,
            &request.headers,
            &request.query_params,
            &request.body,
            &env.variables,
        )
    } else {
        (
            request.url.clone(),
            request.headers.clone(),
            request.query_params.clone(),
            request.body.clone(),
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
    parts.push(format!("'{url_string}'"));

    // Headers
    for h in headers.iter().filter(|h| h.is_enabled && !h.key.is_empty()) {
        parts.push(format!("-H '{}: {}'", h.key, h.value));
    }

    // Body
    match &body {
        RequestBody::Json(s) => {
            parts.push(format!("-d '{s}'"));
        }
        RequestBody::RawText(s) => {
            parts.push(format!("-d '{s}'"));
        }
        RequestBody::FormData(pairs) => {
            let encoded: String = pairs
                .iter()
                .filter(|p| p.is_enabled && !p.key.is_empty())
                .map(|p| format!("{}={}", percent_encode(&p.key), percent_encode(&p.value)))
                .collect::<Vec<_>>()
                .join("&");
            parts.push(format!("-d '{encoded}'"));
        }
        RequestBody::None => {}
    }

    parts.join(" \\\n  ")
}

fn percent_encode(s: &str) -> String {
    urlencoding::encode(s).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{HttpMethod, KeyValuePair, RequestBody, SavedRequest};
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
}
