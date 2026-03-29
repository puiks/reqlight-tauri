use crate::services::scripting::{
    execute_pre_request, execute_test, ScriptRequestData, ScriptResponseData, ScriptResult,
};
use std::collections::HashMap;

#[tauri::command]
pub fn execute_script(
    script: String,
    script_type: String,
    env_vars: HashMap<String, String>,
    request: ScriptRequestData,
    response: Option<ScriptResponseData>,
) -> Result<ScriptResult, String> {
    match script_type.as_str() {
        "pre-request" => {
            execute_pre_request(&script, &env_vars, &request).map_err(|e| e.to_string())
        }
        "test" => {
            let resp = response.unwrap_or_default();
            execute_test(&script, &env_vars, &request, &resp).map_err(|e| e.to_string())
        }
        _ => Err(format!("Unknown script type: {script_type}")),
    }
}
