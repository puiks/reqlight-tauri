use std::sync::Arc;

use tauri::State;
use tokio::sync::Notify;

use crate::models::{
    AuthConfig, HttpMethod, KeyValuePair, RequestBody, RequestEnvironment, ResponseRecord,
};
use crate::services::{http_client, interpolator};
use crate::SharedHttpClient;

/// Shared cancellation signal. When `notify_waiters()` is called,
/// any in-flight request will be aborted.
pub struct RequestCanceller(pub Arc<Notify>);

/// IPC: Execute an HTTP request with optional environment variable interpolation.
/// Supports cancellation via the shared `RequestCanceller` signal.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn send_request(
    method: HttpMethod,
    url: String,
    headers: Vec<KeyValuePair>,
    query_params: Vec<KeyValuePair>,
    body: RequestBody,
    #[allow(unused_mut)] mut auth: AuthConfig,
    timeout_secs: Option<u64>,
    environment: Option<RequestEnvironment>,
    http_client_state: State<'_, SharedHttpClient>,
    canceller: State<'_, RequestCanceller>,
) -> Result<ResponseRecord, String> {
    // Interpolate variables if environment is provided
    let (final_url, final_headers, final_params, final_body, final_auth) = if let Some(ref env) =
        environment
    {
        let (u, h, p, b) =
            interpolator::interpolate_request(&url, &headers, &query_params, &body, &env.variables);
        let a = interpolator::interpolate_auth(&auth, &env.variables);
        (u, h, p, b, a)
    } else {
        (url, headers, query_params, body, auth)
    };

    let client = http_client_state.0.clone();
    let cancel = canceller.0.clone();

    tokio::select! {
        result = http_client::execute(
            &client,
            &method,
            &final_url,
            &final_headers,
            &final_params,
            &final_body,
            &final_auth,
            timeout_secs,
        ) => result,
        _ = cancel.notified() => Err("Request cancelled".to_string()),
    }
}

/// IPC: Cancel any in-flight HTTP request by notifying the cancellation signal.
#[tauri::command]
pub async fn cancel_request(canceller: State<'_, RequestCanceller>) -> Result<(), String> {
    canceller.0.notify_waiters();
    Ok(())
}
