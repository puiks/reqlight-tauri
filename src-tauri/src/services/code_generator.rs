use crate::models::{
    ApiKeyLocation, AuthConfig, HttpMethod, KeyValuePair, RequestBody, RequestEnvironment,
    SavedRequest,
};
use crate::services::interpolator;

/// Generate a code snippet for the given request in the specified language.
pub fn generate(
    request: &SavedRequest,
    environment: Option<&RequestEnvironment>,
    language: &str,
) -> Result<String, String> {
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

    let method = &request.method;
    let enabled_headers: Vec<&KeyValuePair> = headers
        .iter()
        .filter(|h| h.is_enabled && !h.key.is_empty())
        .collect();
    let enabled_params: Vec<&KeyValuePair> = params
        .iter()
        .filter(|p| p.is_enabled && !p.key.is_empty())
        .collect();

    match language {
        "javascript-fetch" => Ok(generate_fetch(
            method,
            &url,
            &enabled_headers,
            &enabled_params,
            &body,
            &auth,
        )),
        "javascript-axios" => Ok(generate_axios(
            method,
            &url,
            &enabled_headers,
            &enabled_params,
            &body,
            &auth,
        )),
        "python-requests" => Ok(generate_python(
            method,
            &url,
            &enabled_headers,
            &enabled_params,
            &body,
            &auth,
        )),
        "curl" => Ok(crate::services::curl_exporter::export(request, environment)),
        _ => Err(format!("Unsupported language: {language}")),
    }
}

fn build_url_with_params(url: &str, params: &[&KeyValuePair]) -> String {
    if params.is_empty() {
        return url.to_string();
    }
    let qs: String = params
        .iter()
        .map(|p| {
            format!(
                "{}={}",
                urlencoding::encode(&p.key),
                urlencoding::encode(&p.value)
            )
        })
        .collect::<Vec<_>>()
        .join("&");
    let sep = if url.contains('?') { "&" } else { "?" };
    format!("{url}{sep}{qs}")
}

fn js_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
}

fn py_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
}

fn collect_auth_headers(auth: &AuthConfig) -> Vec<(String, String)> {
    match auth {
        AuthConfig::BearerToken { token } if !token.is_empty() => {
            vec![("Authorization".to_string(), format!("Bearer {token}"))]
        }
        AuthConfig::BasicAuth { username, password } => {
            let encoded =
                base64::engine::general_purpose::STANDARD.encode(format!("{username}:{password}"));
            vec![("Authorization".to_string(), format!("Basic {encoded}"))]
        }
        AuthConfig::ApiKey {
            key,
            value,
            location: ApiKeyLocation::Header,
        } if !key.is_empty() => {
            vec![(key.clone(), value.clone())]
        }
        AuthConfig::OAuth2 { access_token, .. } if !access_token.is_empty() => {
            vec![(
                "Authorization".to_string(),
                format!("Bearer {access_token}"),
            )]
        }
        _ => vec![],
    }
}

fn api_key_query_param(auth: &AuthConfig) -> Option<(String, String)> {
    if let AuthConfig::ApiKey {
        key,
        value,
        location: ApiKeyLocation::Query,
    } = auth
    {
        if !key.is_empty() {
            return Some((key.clone(), value.clone()));
        }
    }
    None
}

fn body_content(body: &RequestBody) -> Option<(String, Option<String>)> {
    match body {
        RequestBody::Json(s) => Some((s.clone(), Some("application/json".to_string()))),
        RequestBody::RawText(s) => Some((s.clone(), None)),
        RequestBody::FormData(pairs) => {
            let encoded: String = pairs
                .iter()
                .filter(|p| p.is_enabled && !p.key.is_empty())
                .map(|p| {
                    format!(
                        "{}={}",
                        urlencoding::encode(&p.key),
                        urlencoding::encode(&p.value)
                    )
                })
                .collect::<Vec<_>>()
                .join("&");
            Some((
                encoded,
                Some("application/x-www-form-urlencoded".to_string()),
            ))
        }
        RequestBody::GraphQL { query, variables } => {
            let vars_value: serde_json::Value = if variables.trim().is_empty() {
                serde_json::Value::Null
            } else {
                serde_json::from_str(variables).unwrap_or(serde_json::Value::Null)
            };
            let gql = serde_json::json!({ "query": query, "variables": vars_value });
            Some((
                serde_json::to_string_pretty(&gql).unwrap_or_default(),
                Some("application/json".to_string()),
            ))
        }
        _ => None,
    }
}

// --- JavaScript fetch ---
fn generate_fetch(
    method: &HttpMethod,
    url: &str,
    headers: &[&KeyValuePair],
    params: &[&KeyValuePair],
    body: &RequestBody,
    auth: &AuthConfig,
) -> String {
    let mut full_url = build_url_with_params(url, params);
    if let Some((k, v)) = api_key_query_param(auth) {
        let sep = if full_url.contains('?') { "&" } else { "?" };
        full_url = format!(
            "{full_url}{sep}{}={}",
            urlencoding::encode(&k),
            urlencoding::encode(&v)
        );
    }

    let mut lines = vec![format!(
        "const response = await fetch(\"{}\", {{",
        js_escape(&full_url)
    )];
    lines.push(format!("  method: \"{}\",", method.as_str()));

    let auth_headers = collect_auth_headers(auth);
    if !headers.is_empty() || !auth_headers.is_empty() || body_content(body).is_some() {
        lines.push("  headers: {".to_string());
        for h in headers {
            lines.push(format!(
                "    \"{}\": \"{}\",",
                js_escape(&h.key),
                js_escape(&h.value)
            ));
        }
        for (k, v) in &auth_headers {
            lines.push(format!("    \"{}\": \"{}\",", js_escape(k), js_escape(v)));
        }
        if let Some((_, Some(ref ct))) = body_content(body) {
            let already_set = headers
                .iter()
                .any(|h| h.key.eq_ignore_ascii_case("content-type"))
                || auth_headers
                    .iter()
                    .any(|(k, _)| k.eq_ignore_ascii_case("content-type"));
            if !already_set {
                lines.push(format!("    \"Content-Type\": \"{ct}\","));
            }
        }
        lines.push("  },".to_string());
    }

    if let Some((ref content, _)) = body_content(body) {
        lines.push(format!("  body: \"{}\",", js_escape(content)));
    }

    lines.push("});".to_string());
    lines.push("const data = await response.text();".to_string());
    lines.join("\n")
}

// --- JavaScript axios ---
fn generate_axios(
    method: &HttpMethod,
    url: &str,
    headers: &[&KeyValuePair],
    params: &[&KeyValuePair],
    body: &RequestBody,
    auth: &AuthConfig,
) -> String {
    let mut full_url = build_url_with_params(url, params);
    if let Some((k, v)) = api_key_query_param(auth) {
        let sep = if full_url.contains('?') { "&" } else { "?" };
        full_url = format!(
            "{full_url}{sep}{}={}",
            urlencoding::encode(&k),
            urlencoding::encode(&v)
        );
    }

    let method_lower = method.as_str().to_lowercase();
    let mut lines = vec![format!(
        "const response = await axios.{method_lower}(\"{}\", {{",
        js_escape(&full_url)
    )];

    let auth_headers = collect_auth_headers(auth);
    if !headers.is_empty() || !auth_headers.is_empty() || body_content(body).is_some() {
        lines.push("  headers: {".to_string());
        for h in headers {
            lines.push(format!(
                "    \"{}\": \"{}\",",
                js_escape(&h.key),
                js_escape(&h.value)
            ));
        }
        for (k, v) in &auth_headers {
            lines.push(format!("    \"{}\": \"{}\",", js_escape(k), js_escape(v)));
        }
        if let Some((_, Some(ref ct))) = body_content(body) {
            let already_set = headers
                .iter()
                .any(|h| h.key.eq_ignore_ascii_case("content-type"));
            if !already_set {
                lines.push(format!("    \"Content-Type\": \"{ct}\","));
            }
        }
        lines.push("  },".to_string());
    }

    if let Some((ref content, _)) = body_content(body) {
        lines.push(format!("  data: \"{}\",", js_escape(content)));
    }

    lines.push("});".to_string());
    lines.join("\n")
}

// --- Python requests ---
fn generate_python(
    method: &HttpMethod,
    url: &str,
    headers: &[&KeyValuePair],
    params: &[&KeyValuePair],
    body: &RequestBody,
    auth: &AuthConfig,
) -> String {
    let mut full_url = build_url_with_params(url, params);
    if let Some((k, v)) = api_key_query_param(auth) {
        let sep = if full_url.contains('?') { "&" } else { "?" };
        full_url = format!(
            "{full_url}{sep}{}={}",
            urlencoding::encode(&k),
            urlencoding::encode(&v)
        );
    }

    let method_lower = method.as_str().to_lowercase();
    let mut lines = vec!["import requests".to_string(), String::new()];

    let auth_headers = collect_auth_headers(auth);
    if !headers.is_empty() || !auth_headers.is_empty() || body_content(body).is_some() {
        lines.push("headers = {".to_string());
        for h in headers {
            lines.push(format!(
                "    \"{}\": \"{}\",",
                py_escape(&h.key),
                py_escape(&h.value)
            ));
        }
        for (k, v) in &auth_headers {
            lines.push(format!("    \"{}\": \"{}\",", py_escape(k), py_escape(v)));
        }
        if let Some((_, Some(ref ct))) = body_content(body) {
            let already_set = headers
                .iter()
                .any(|h| h.key.eq_ignore_ascii_case("content-type"));
            if !already_set {
                lines.push(format!("    \"Content-Type\": \"{ct}\","));
            }
        }
        lines.push("}".to_string());
        lines.push(String::new());
    }

    let has_headers =
        !headers.is_empty() || !auth_headers.is_empty() || body_content(body).is_some();
    let has_body = body_content(body).is_some();

    if has_body {
        if let Some((ref content, _)) = body_content(body) {
            lines.push(format!("data = \"{}\"", py_escape(content)));
            lines.push(String::new());
        }
    }

    let mut call_parts = vec![format!(
        "response = requests.{method_lower}(\"{}\"",
        py_escape(&full_url)
    )];
    if has_headers {
        call_parts.push("headers=headers".to_string());
    }
    if has_body {
        call_parts.push("data=data".to_string());
    }

    if call_parts.len() == 1 {
        lines.push(format!("{})", call_parts[0]));
    } else {
        let first = call_parts.remove(0);
        lines.push(format!("{}, {}", first, call_parts.join(", ") + ")"));
    }

    lines.push("print(response.text)".to_string());
    lines.join("\n")
}

use base64::Engine;

#[cfg(test)]
#[path = "code_generator_tests.rs"]
mod tests;
