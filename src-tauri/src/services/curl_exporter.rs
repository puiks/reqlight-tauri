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
            .map(|p| {
                format!(
                    "{}={}",
                    percent_encode(&p.key),
                    percent_encode(&p.value)
                )
            })
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
                .map(|p| {
                    format!(
                        "{}={}",
                        percent_encode(&p.key),
                        percent_encode(&p.value)
                    )
                })
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
