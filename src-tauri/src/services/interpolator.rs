use std::collections::HashMap;

use crate::models::{AuthConfig, KeyValuePair, RequestBody};

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
        RequestBody::Multipart(fields) => RequestBody::Multipart(
            fields
                .iter()
                .map(|f| crate::models::MultipartField {
                    id: f.id,
                    name: interpolate(&f.name, variables),
                    value: interpolate(&f.value, variables),
                    file_path: f.file_path.as_ref().map(|p| interpolate(p, variables)),
                    is_enabled: f.is_enabled,
                })
                .collect(),
        ),
        RequestBody::GraphQL {
            query,
            variables: vars,
        } => RequestBody::GraphQL {
            query: interpolate(query, variables),
            variables: interpolate(vars, variables),
        },
        RequestBody::None => RequestBody::None,
    };

    (new_url, new_headers, new_params, new_body)
}

/// Interpolate variables in auth configuration fields.
pub fn interpolate_auth(auth: &AuthConfig, variables: &[KeyValuePair]) -> AuthConfig {
    match auth {
        AuthConfig::None => AuthConfig::None,
        AuthConfig::BearerToken { token } => AuthConfig::BearerToken {
            token: interpolate(token, variables),
        },
        AuthConfig::BasicAuth { username, password } => AuthConfig::BasicAuth {
            username: interpolate(username, variables),
            password: interpolate(password, variables),
        },
        AuthConfig::ApiKey {
            key,
            value,
            location,
        } => AuthConfig::ApiKey {
            key: interpolate(key, variables),
            value: interpolate(value, variables),
            location: location.clone(),
        },
        AuthConfig::OAuth2 {
            grant_type,
            client_id,
            client_secret,
            auth_url,
            token_url,
            scopes,
            access_token,
            refresh_token,
            token_expiry,
        } => AuthConfig::OAuth2 {
            grant_type: grant_type.clone(),
            client_id: interpolate(client_id, variables),
            client_secret: interpolate(client_secret, variables),
            auth_url: interpolate(auth_url, variables),
            token_url: interpolate(token_url, variables),
            scopes: interpolate(scopes, variables),
            access_token: interpolate(access_token, variables),
            refresh_token: interpolate(refresh_token, variables),
            token_expiry: token_expiry.clone(),
        },
    }
}

#[cfg(test)]
#[path = "interpolator_tests.rs"]
mod tests;
