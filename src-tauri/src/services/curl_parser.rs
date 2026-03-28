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
                        "HEAD" => HttpMethod::Head,
                        "OPTIONS" => HttpMethod::Options,
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
#[path = "curl_parser_tests.rs"]
mod tests;
