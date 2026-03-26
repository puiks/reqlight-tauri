use crate::models::{HttpMethod, KeyValuePair, RequestBody, SavedRequest};

/// Parse a cURL command string into a SavedRequest.
/// 1:1 port of Swift's CurlParser.
pub fn parse(curl: &str) -> Result<SavedRequest, String> {
    let tokens = tokenize(curl);
    if tokens.is_empty() {
        return Err("Empty cURL command".to_string());
    }

    let mut request = SavedRequest::default();
    let mut i = 0;

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
            "-d" | "--data" | "--data-raw" | "--data-binary" => {
                i += 1;
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
                    if request.method == HttpMethod::Get {
                        request.method = HttpMethod::Post;
                    }
                }
            }
            "-G" | "--get" => {
                request.method = HttpMethod::Get;
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
        assert_eq!(req.headers.len(), 2);
        assert_eq!(req.headers[0].key, "Content-Type");
        assert_eq!(req.headers[0].value, "application/json");
        assert_eq!(req.headers[1].key, "Authorization");
        assert_eq!(req.headers[1].value, "Bearer token");
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
    fn parse_get_flag_overrides_method() {
        let req = parse("curl -X POST -G https://example.com").unwrap();
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
}
