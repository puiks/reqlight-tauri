use crate::models::{
    ApiKeyLocation, AuthConfig, HttpMethod, KeyValuePair, RequestBody, RequestEnvironment,
    SavedRequest,
};
use crate::services::interpolator;

use base64::Engine;

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

    let enabled_headers: Vec<&KeyValuePair> = headers
        .iter()
        .filter(|h| h.is_enabled && !h.key.is_empty())
        .collect();
    let enabled_params: Vec<&KeyValuePair> = params
        .iter()
        .filter(|p| p.is_enabled && !p.key.is_empty())
        .collect();

    let m = &request.method;
    let h = &enabled_headers;
    let p = &enabled_params;
    match language {
        "javascript-fetch" => Ok(generate_js(m, &url, h, p, &body, &auth, true)),
        "javascript-axios" => Ok(generate_js(m, &url, h, p, &body, &auth, false)),
        "python-requests" => Ok(generate_python(m, &url, h, p, &body, &auth)),
        "curl" => Ok(crate::services::curl_exporter::export(request, environment)),
        _ => Err(format!("Unsupported language: {language}")),
    }
}

/// Build the full URL with query params and optional API key query param.
fn build_full_url(url: &str, params: &[&KeyValuePair], auth: &AuthConfig) -> String {
    let mut full = if params.is_empty() {
        url.to_string()
    } else {
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
    };
    if let AuthConfig::ApiKey {
        key,
        value,
        location: ApiKeyLocation::Query,
    } = auth
    {
        if !key.is_empty() {
            let sep = if full.contains('?') { "&" } else { "?" };
            full = format!(
                "{full}{sep}{}={}",
                urlencoding::encode(key),
                urlencoding::encode(value)
            );
        }
    }
    full
}

fn escape_str(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
}

fn collect_auth_headers(auth: &AuthConfig) -> Vec<(String, String)> {
    match auth {
        AuthConfig::BearerToken { token } if !token.is_empty() => {
            vec![("Authorization".into(), format!("Bearer {token}"))]
        }
        AuthConfig::BasicAuth { username, password } => {
            let enc =
                base64::engine::general_purpose::STANDARD.encode(format!("{username}:{password}"));
            vec![("Authorization".into(), format!("Basic {enc}"))]
        }
        AuthConfig::ApiKey {
            key,
            value,
            location: ApiKeyLocation::Header,
        } if !key.is_empty() => {
            vec![(key.clone(), value.clone())]
        }
        AuthConfig::OAuth2 { access_token, .. } if !access_token.is_empty() => {
            vec![("Authorization".into(), format!("Bearer {access_token}"))]
        }
        _ => vec![],
    }
}

fn body_content(body: &RequestBody) -> Option<(String, Option<String>)> {
    match body {
        RequestBody::Json(s) => Some((s.clone(), Some("application/json".into()))),
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
            Some((encoded, Some("application/x-www-form-urlencoded".into())))
        }
        RequestBody::GraphQL { query, variables } => {
            let vars_value: serde_json::Value = if variables.trim().is_empty() {
                serde_json::Value::Null
            } else {
                serde_json::from_str(variables).unwrap_or_else(|e| {
                    eprintln!("Warning: failed to parse GraphQL variables as JSON: {e}");
                    serde_json::Value::Null
                })
            };
            let gql = serde_json::json!({ "query": query, "variables": vars_value });
            Some((
                serde_json::to_string_pretty(&gql).unwrap_or_default(),
                Some("application/json".into()),
            ))
        }
        _ => None,
    }
}

/// Push escaped header key-value lines (with given indent) and auto-add Content-Type if needed.
fn push_header_entries(
    lines: &mut Vec<String>,
    indent: &str,
    headers: &[&KeyValuePair],
    auth_headers: &[(String, String)],
    body: &RequestBody,
) {
    for h in headers {
        lines.push(format!(
            "{indent}\"{}\": \"{}\",",
            escape_str(&h.key),
            escape_str(&h.value)
        ));
    }
    for (k, v) in auth_headers {
        lines.push(format!(
            "{indent}\"{}\": \"{}\",",
            escape_str(k),
            escape_str(v)
        ));
    }
    if let Some((_, Some(ref ct))) = body_content(body) {
        let already_set = headers
            .iter()
            .any(|h| h.key.eq_ignore_ascii_case("content-type"))
            || auth_headers
                .iter()
                .any(|(k, _)| k.eq_ignore_ascii_case("content-type"));
        if !already_set {
            lines.push(format!("{indent}\"Content-Type\": \"{ct}\","));
        }
    }
}

fn has_header_content(
    headers: &[&KeyValuePair],
    auth_headers: &[(String, String)],
    body: &RequestBody,
) -> bool {
    !headers.is_empty() || !auth_headers.is_empty() || body_content(body).is_some()
}

/// Shared generator for fetch and axios (same structure, different opening/body-key/suffix).
fn generate_js(
    method: &HttpMethod,
    url: &str,
    headers: &[&KeyValuePair],
    params: &[&KeyValuePair],
    body: &RequestBody,
    auth: &AuthConfig,
    is_fetch: bool,
) -> String {
    let full_url = build_full_url(url, params, auth);
    let escaped_url = escape_str(&full_url);
    let auth_headers = collect_auth_headers(auth);
    let mut lines = if is_fetch {
        vec![
            format!("const response = await fetch(\"{escaped_url}\", {{"),
            format!("  method: \"{}\",", method.as_str()),
        ]
    } else {
        let m = method.as_str().to_lowercase();
        vec![format!(
            "const response = await axios.{m}(\"{escaped_url}\", {{"
        )]
    };
    if has_header_content(headers, &auth_headers, body) {
        lines.push("  headers: {".into());
        push_header_entries(&mut lines, "    ", headers, &auth_headers, body);
        lines.push("  },".into());
    }
    if let Some((ref content, _)) = body_content(body) {
        let key = if is_fetch { "body" } else { "data" };
        lines.push(format!("  {key}: \"{}\",", escape_str(content)));
    }
    lines.push("});".into());
    if is_fetch {
        lines.push("const data = await response.text();".into());
    }
    lines.join("\n")
}

fn generate_python(
    method: &HttpMethod,
    url: &str,
    headers: &[&KeyValuePair],
    params: &[&KeyValuePair],
    body: &RequestBody,
    auth: &AuthConfig,
) -> String {
    let full_url = build_full_url(url, params, auth);
    let method_lower = method.as_str().to_lowercase();
    let mut lines = vec!["import requests".into(), String::new()];
    let auth_headers = collect_auth_headers(auth);
    let has_hdrs = has_header_content(headers, &auth_headers, body);
    if has_hdrs {
        lines.push("headers = {".into());
        push_header_entries(&mut lines, "    ", headers, &auth_headers, body);
        lines.push("}".into());
        lines.push(String::new());
    }
    let has_body = body_content(body).is_some();
    if let Some((ref content, _)) = body_content(body) {
        lines.push(format!("data = \"{}\"", escape_str(content)));
        lines.push(String::new());
    }
    let mut call = vec![format!(
        "response = requests.{method_lower}(\"{}\"",
        escape_str(&full_url)
    )];
    if has_hdrs {
        call.push("headers=headers".into());
    }
    if has_body {
        call.push("data=data".into());
    }
    if call.len() == 1 {
        lines.push(format!("{})", call[0]));
    } else {
        let first = call.remove(0);
        lines.push(format!("{}, {}", first, call.join(", ") + ")"));
    }
    lines.push("print(response.text)".into());
    lines.join("\n")
}

#[cfg(test)]
#[path = "code_generator_tests.rs"]
mod tests;
