use std::sync::Arc;

use tauri::State;
use tokio::sync::Notify;

use crate::models::{HttpMethod, KeyValuePair, RequestBody, RequestEnvironment, ResponseRecord};
use crate::services::{http_client, interpolator};

/// Shared cancellation signal. When `notify_waiters()` is called,
/// any in-flight request will be aborted.
pub struct RequestCanceller(pub Arc<Notify>);

#[tauri::command]
pub async fn send_request(
    method: HttpMethod,
    url: String,
    headers: Vec<KeyValuePair>,
    query_params: Vec<KeyValuePair>,
    body: RequestBody,
    timeout_secs: Option<u64>,
    environment: Option<RequestEnvironment>,
    canceller: State<'_, RequestCanceller>,
) -> Result<ResponseRecord, String> {
    // Interpolate variables if environment is provided
    let (final_url, final_headers, final_params, final_body) = if let Some(ref env) = environment {
        interpolator::interpolate_request(&url, &headers, &query_params, &body, &env.variables)
    } else {
        (url, headers, query_params, body)
    };

    let cancel = canceller.0.clone();

    tokio::select! {
        result = http_client::execute(
            &method,
            &final_url,
            &final_headers,
            &final_params,
            &final_body,
            timeout_secs,
        ) => result,
        _ = cancel.notified() => Err("Request cancelled".to_string()),
    }
}

#[tauri::command]
pub async fn cancel_request(canceller: State<'_, RequestCanceller>) -> Result<(), String> {
    canceller.0.notify_waiters();
    Ok(())
}
