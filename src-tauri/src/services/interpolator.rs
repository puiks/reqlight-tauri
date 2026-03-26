use std::collections::HashMap;

use crate::models::{KeyValuePair, RequestBody};

/// Replace all {{var}} occurrences with environment variable values.
/// Unmatched variables are kept as-is.
pub fn interpolate(input: &str, variables: &[KeyValuePair]) -> String {
    if !input.contains("{{") {
        return input.to_string();
    }

    let lookup: HashMap<&str, &str> = variables
        .iter()
        .filter(|v| v.is_enabled)
        .map(|v| (v.key.as_str(), v.value.as_str()))
        .collect();

    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        if i + 1 < len && chars[i] == '{' && chars[i + 1] == '{' {
            // Found "{{", look for "}}"
            let var_start = i + 2;
            let mut found_end = false;
            let mut j = var_start;
            while j + 1 < len {
                if chars[j] == '}' && chars[j + 1] == '}' {
                    let var_name: String = chars[var_start..j].iter().collect();
                    if let Some(&value) = lookup.get(var_name.as_str()) {
                        result.push_str(value);
                    } else {
                        result.push_str("{{");
                        result.push_str(&var_name);
                        result.push_str("}}");
                    }
                    i = j + 2;
                    found_end = true;
                    break;
                }
                j += 1;
            }
            if !found_end {
                result.push(chars[i]);
                i += 1;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

/// Interpolate all parts of a request.
pub fn interpolate_request(
    url: &str,
    headers: &[KeyValuePair],
    query_params: &[KeyValuePair],
    body: &RequestBody,
    variables: &[KeyValuePair],
) -> (String, Vec<KeyValuePair>, Vec<KeyValuePair>, RequestBody) {
    let new_url = interpolate(url, variables);

    let new_headers: Vec<KeyValuePair> = headers
        .iter()
        .map(|h| KeyValuePair {
            id: h.id,
            key: interpolate(&h.key, variables),
            value: interpolate(&h.value, variables),
            is_enabled: h.is_enabled,
            is_secret: h.is_secret,
        })
        .collect();

    let new_params: Vec<KeyValuePair> = query_params
        .iter()
        .map(|p| KeyValuePair {
            id: p.id,
            key: interpolate(&p.key, variables),
            value: interpolate(&p.value, variables),
            is_enabled: p.is_enabled,
            is_secret: p.is_secret,
        })
        .collect();

    let new_body = match body {
        RequestBody::Json(s) => RequestBody::Json(interpolate(s, variables)),
        RequestBody::RawText(s) => RequestBody::RawText(interpolate(s, variables)),
        RequestBody::FormData(pairs) => RequestBody::FormData(
            pairs
                .iter()
                .map(|p| KeyValuePair {
                    id: p.id,
                    key: interpolate(&p.key, variables),
                    value: interpolate(&p.value, variables),
                    is_enabled: p.is_enabled,
                    is_secret: p.is_secret,
                })
                .collect(),
        ),
        RequestBody::None => RequestBody::None,
    };

    (new_url, new_headers, new_params, new_body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::KeyValuePair;
    use uuid::Uuid;

    fn make_var(key: &str, value: &str) -> KeyValuePair {
        KeyValuePair {
            id: Uuid::new_v4(),
            key: key.to_string(),
            value: value.to_string(),
            is_enabled: true,
            is_secret: false,
        }
    }

    fn make_disabled_var(key: &str, value: &str) -> KeyValuePair {
        KeyValuePair {
            id: Uuid::new_v4(),
            key: key.to_string(),
            value: value.to_string(),
            is_enabled: false,
            is_secret: false,
        }
    }

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
        let headers = vec![KeyValuePair {
            id: Uuid::new_v4(),
            key: "Authorization".to_string(),
            value: "Bearer {{token}}".to_string(),
            is_enabled: true,
            is_secret: false,
        }];
        let params = vec![KeyValuePair {
            id: Uuid::new_v4(),
            key: "q".to_string(),
            value: "{{host}}".to_string(),
            is_enabled: true,
            is_secret: false,
        }];
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
}
