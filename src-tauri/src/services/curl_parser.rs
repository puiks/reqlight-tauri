use crate::models::{AuthConfig, HttpMethod, KeyValuePair, RequestBody, SavedRequest};

/// Parse a cURL command string into a SavedRequest.
/// 1:1 port of Swift's CurlParser.
pub fn parse(curl: &str) -> Result<SavedRequest, String> {
    let tokens = tokenize(curl);
    if tokens.is_empty() {
        return Err("Empty cURL command".to_string());
    }

    let mut request = SavedRequest::default();
    let mut i = 0;
    let mut explicit_method = false; // true if -X was used
    let mut has_data = false;
    let mut force_get = false; // true if -G was used

    // Skip "curl" prefix
    if tokens.first().map(|s| s.to_lowercase()) == Some("curl".to_string()) {
        i = 1;
    }

    while i < tokens.len() {
        let token = &tokens[i];
        match token.as_str() {
            "-X" | "--request" => {
                i += 1;
                if i < tokens.len() {
                    explicit_method = true;
                    request.method = match tokens[i].to_uppercase().as_str() {
                        "GET" => HttpMethod::Get,
                        "POST" => HttpMethod::Post,
                        "PUT" => HttpMethod::Put,
                        "PATCH" => HttpMethod::Patch,
                        "DELETE" => HttpMethod::Delete,
                        _ => HttpMethod::Get,
                    };
                }
            }
            "-H" | "--header" => {
                i += 1;
                if i < tokens.len() {
                    if let Some((key, value)) = parse_header(&tokens[i]) {
                        // Detect Bearer token from Authorization header
                        if key.eq_ignore_ascii_case("authorization")
                            && value.to_lowercase().starts_with("bearer ")
                        {
                            let token = value["bearer ".len()..].trim().to_string();
                            request.auth = AuthConfig::BearerToken { token };
                        } else {
                            request.headers.push(KeyValuePair {
                                id: uuid::Uuid::new_v4(),
                                key,
                                value,
                                is_enabled: true,
                                is_secret: false,
                            });
                        }
                    }
                }
            }
            "-u" | "--user" => {
                i += 1;
                if i < tokens.len() {
                    let cred = &tokens[i];
                    let (username, password) = match cred.find(':') {
                        Some(idx) => (cred[..idx].to_string(), cred[idx + 1..].to_string()),
                        None => (cred.clone(), String::new()),
                    };
                    request.auth = AuthConfig::BasicAuth { username, password };
                }
            }
            "-d" | "--data" | "--data-raw" | "--data-binary" => {
                i += 1;
                has_data = true;
                if i < tokens.len() {
                    let body_text = &tokens[i];
                    let trimmed = body_text.trim();
                    if trimmed.starts_with('{') || trimmed.starts_with('[') {
                        request.body = RequestBody::Json(body_text.clone());
                    } else if body_text.contains('=') && !body_text.contains(' ') {
                        // Looks like form data
                        let pairs: Vec<KeyValuePair> = body_text
                            .split('&')
                            .filter_map(|part| {
                                let mut kv = part.splitn(2, '=');
                                let key = kv.next()?.to_string();
                                let value = kv.next().unwrap_or("").to_string();
                                if key.is_empty() {
                                    return None;
                                }
                                Some(KeyValuePair {
                                    id: uuid::Uuid::new_v4(),
                                    key,
                                    value,
                                    is_enabled: true,
                                    is_secret: false,
                                })
                            })
                            .collect();
                        request.body = RequestBody::FormData(pairs);
                    } else {
                        request.body = RequestBody::RawText(body_text.clone());
                    }
                }
            }
            "-G" | "--get" => {
                // -G forces GET and tells curl to append -d data as query string.
                // Only override method if -X was not explicitly given.
                force_get = true;
                if !explicit_method {
                    request.method = HttpMethod::Get;
                }
            }
            _ => {
                if !token.starts_with('-') && request.url.is_empty() {
                    request.url = token.clone();
                }
            }
        }
        i += 1;
    }

    if request.url.is_empty() {
        return Err("No URL found in cURL command".to_string());
    }

    // Auto-promote to POST if body data was provided, no explicit method, and no -G flag
    if has_data && !explicit_method && !force_get && request.method == HttpMethod::Get {
        request.method = HttpMethod::Post;
    }

    Ok(request)
}

/// Tokenize a cURL command string, handling quotes and line continuations.
fn tokenize(input: &str) -> Vec<String> {
    // Remove line continuations
    let cleaned = input
        .replace("\\\n", " ")
        .replace("\\\r\n", " ")
        .trim()
        .to_string();

    let mut tokens: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;

    for ch in cleaned.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }

        if ch == '\\' && !in_single_quote {
            escaped = true;
            continue;
        }

        if ch == '\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
            continue;
        }

        if ch == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
            continue;
        }

        if ch.is_whitespace() && !in_single_quote && !in_double_quote {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            continue;
        }

        current.push(ch);
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn parse_header(raw: &str) -> Option<(String, String)> {
    let colon_idx = raw.find(':')?;
    let key = raw[..colon_idx].trim().to_string();
    let value = raw[colon_idx + 1..].trim().to_string();
    Some((key, value))
}

#[cfg(test)]
mod tests {
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
}
