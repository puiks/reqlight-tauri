use crate::models::{HttpMethod, KeyValuePair, RequestBody, RequestEnvironment, ResponseRecord};
use crate::services::{http_client, interpolator};

#[tauri::command]
pub async fn send_request(
    method: HttpMethod,
    url: String,
    headers: Vec<KeyValuePair>,
    query_params: Vec<KeyValuePair>,
    body: RequestBody,
    timeout_secs: Option<u64>,
    environment: Option<RequestEnvironment>,
) -> Result<ResponseRecord, String> {
    // Interpolate variables if environment is provided
    let (final_url, final_headers, final_params, final_body) = if let Some(ref env) = environment {
        interpolator::interpolate_request(&url, &headers, &query_params, &body, &env.variables)
    } else {
        (url, headers, query_params, body)
    };

    http_client::execute(
        &method,
        &final_url,
        &final_headers,
        &final_params,
        &final_body,
        timeout_secs,
    )
    .await
}
