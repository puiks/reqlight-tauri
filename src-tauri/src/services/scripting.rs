use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

use rquickjs::{Context, Function, Object, Runtime};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

use super::scripting_runtime::*;

/// Result returned after executing a script.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScriptResult {
    pub env_updates: Vec<(String, String)>,
    pub test_results: Vec<ScriptTestResult>,
    pub console_output: Vec<String>,
    pub error: Option<String>,
}

/// A single test assertion result from a script.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptTestResult {
    pub name: String,
    pub passed: bool,
    pub message: Option<String>,
}

/// Request data exposed to scripts.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScriptRequestData {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

/// Response data exposed to scripts.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScriptResponseData {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub time: f64,
}

/// Execute a pre-request script. Only env vars and request data are available.
pub fn execute_pre_request(
    script: &str,
    env_vars: &HashMap<String, String>,
    request: &ScriptRequestData,
) -> Result<ScriptResult, AppError> {
    execute_script(script, env_vars, Some(request), None)
}

/// Execute a test script. Response data is also available.
pub fn execute_test(
    script: &str,
    env_vars: &HashMap<String, String>,
    request: &ScriptRequestData,
    response: &ScriptResponseData,
) -> Result<ScriptResult, AppError> {
    execute_script(script, env_vars, Some(request), Some(response))
}

fn execute_script(
    script: &str,
    env_vars: &HashMap<String, String>,
    request: Option<&ScriptRequestData>,
    response: Option<&ScriptResponseData>,
) -> Result<ScriptResult, AppError> {
    let rt = Runtime::new().map_err(|e| AppError::Script(format!("JS runtime error: {e}")))?;
    rt.set_max_stack_size(512 * 1024); // 512KB stack limit

    // 5-second execution timeout to prevent infinite loops
    let deadline = Instant::now() + std::time::Duration::from_secs(5);
    rt.set_interrupt_handler(Some(Box::new(move || Instant::now() > deadline)));
    let ctx = Context::full(&rt).map_err(|e| AppError::Script(format!("JS context error: {e}")))?;

    // Shared state between Rust and JS closures
    let env_state: Rc<RefCell<HashMap<String, String>>> = Rc::new(RefCell::new(env_vars.clone()));
    let env_updates: Rc<RefCell<Vec<(String, String)>>> = Rc::new(RefCell::new(Vec::new()));
    let test_results: Rc<RefCell<Vec<ScriptTestResult>>> = Rc::new(RefCell::new(Vec::new()));
    let console_output: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));

    let result = ctx.with(|ctx| -> Result<ScriptResult, AppError> {
        let globals = ctx.globals();

        // Register console.log — use a Rust string sink + JS wrapper for type coercion
        let log_output = console_output.clone();
        globals
            .set(
                "__log",
                Function::new(ctx.clone(), move |msg: String| {
                    log_output.borrow_mut().push(msg);
                })
                .map_err(|e| AppError::Script(format!("Failed to set __log: {e}")))?,
            )
            .map_err(|e| AppError::Script(format!("Failed to set __log: {e}")))?;
        ctx.eval::<(), _>(
            "var console = { log: function() { __log(Array.prototype.map.call(arguments, String).join(' ')); } };",
        ).map_err(|e| AppError::Script(format!("Failed to init console: {e}")))?;

        // Register `rl` global object
        let rl = Object::new(ctx.clone())
            .map_err(|e| AppError::Script(format!("Failed to create rl: {e}")))?;

        // rl.environment.get / rl.environment.set
        register_environment(&ctx, &rl, &env_state, &env_updates)?;

        // rl.request
        if let Some(req) = request {
            register_request(&ctx, &rl, req)?;
        }

        // rl.response (only for test scripts) — Rust object setup only
        if let Some(resp) = response {
            register_response(&ctx, &rl, resp)?;
        }

        // rl.test(name, fn) — Rust callback only
        register_test_fn(&ctx, &rl, &test_results)?;

        // Set rl on globals BEFORE JS evals that reference `rl` by name
        globals
            .set("rl", rl)
            .map_err(|e| AppError::Script(format!("Failed to set rl global: {e}")))?;

        // Register crypto global
        register_crypto(&ctx, &globals)?;

        // Now inject JS wrappers that reference `rl` by name
        if let Some(resp) = response {
            inject_response_json(&ctx, resp)?;
        }
        inject_expect_api(&ctx)?;

        // Execute the script
        let eval_result: Result<(), rquickjs::Error> = ctx.eval(script);
        let error = eval_result.err().map(|e| format!("{e}"));

        Ok(ScriptResult {
            env_updates: env_updates.borrow().clone(),
            test_results: test_results.borrow().clone(),
            console_output: console_output.borrow().clone(),
            error,
        })
    });

    result
}

#[cfg(test)]
#[path = "scripting_tests.rs"]
mod tests;
