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
