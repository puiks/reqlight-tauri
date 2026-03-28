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
        AuthConfig::OAuth2 { access_token, .. } if !access_token.is_empty() => {
            parts.push(format!(
                "-H '{}'",
                shell_escape(&format!("Authorization: Bearer {access_token}"))
            ));
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
        RequestBody::Multipart(fields) => {
            for field in fields.iter().filter(|f| f.is_enabled && !f.name.is_empty()) {
                if let Some(ref path) = field.file_path {
                    parts.push(format!(
                        "-F '{}=@{}'",
                        shell_escape(&field.name),
                        shell_escape(path)
                    ));
                } else {
                    parts.push(format!(
                        "-F '{}={}'",
                        shell_escape(&field.name),
                        shell_escape(&field.value)
                    ));
                }
            }
        }
        RequestBody::GraphQL { query, variables } => {
            let vars_value: serde_json::Value = if variables.trim().is_empty() {
                serde_json::Value::Null
            } else {
                serde_json::from_str(variables).unwrap_or(serde_json::Value::Null)
            };
            let gql = serde_json::json!({ "query": query, "variables": vars_value });
            parts.push(format!("-d '{}'", shell_escape(&gql.to_string())));
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
#[path = "curl_exporter_tests.rs"]
mod tests;
