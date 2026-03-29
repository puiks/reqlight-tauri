use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use rquickjs::{Function, Object};

use crate::error::AppError;
use crate::services::scripting::{ScriptRequestData, ScriptResponseData, ScriptTestResult};

pub(crate) fn register_environment<'js>(
    ctx: &rquickjs::Ctx<'js>,
    rl: &Object<'js>,
    env_state: &Rc<RefCell<HashMap<String, String>>>,
    env_updates: &Rc<RefCell<Vec<(String, String)>>>,
) -> Result<(), AppError> {
    let env_obj = Object::new(ctx.clone())
        .map_err(|e| AppError::Script(format!("Failed to create environment: {e}")))?;

    let get_state = env_state.clone();
    env_obj
        .set(
            "get",
            Function::new(ctx.clone(), move |key: String| -> String {
                get_state.borrow().get(&key).cloned().unwrap_or_default()
            })
            .map_err(|e| AppError::Script(format!("Failed to set env.get: {e}")))?,
        )
        .map_err(|e| AppError::Script(format!("Failed to set env.get: {e}")))?;

    let set_state = env_state.clone();
    let set_updates = env_updates.clone();
    env_obj
        .set(
            "set",
            Function::new(ctx.clone(), move |key: String, value: String| {
                set_state.borrow_mut().insert(key.clone(), value.clone());
                set_updates.borrow_mut().push((key, value));
            })
            .map_err(|e| AppError::Script(format!("Failed to set env.set: {e}")))?,
        )
        .map_err(|e| AppError::Script(format!("Failed to set env.set: {e}")))?;

    rl.set("environment", env_obj)
        .map_err(|e| AppError::Script(format!("Failed to set rl.environment: {e}")))?;
    Ok(())
}

pub(crate) fn register_request<'js>(
    ctx: &rquickjs::Ctx<'js>,
    rl: &Object<'js>,
    req: &ScriptRequestData,
) -> Result<(), AppError> {
    let req_obj = Object::new(ctx.clone())
        .map_err(|e| AppError::Script(format!("Failed to create request: {e}")))?;

    req_obj
        .set("method", req.method.as_str())
        .map_err(|e| AppError::Script(format!("set request.method: {e}")))?;
    req_obj
        .set("url", req.url.as_str())
        .map_err(|e| AppError::Script(format!("set request.url: {e}")))?;
    req_obj
        .set("body", req.body.as_str())
        .map_err(|e| AppError::Script(format!("set request.body: {e}")))?;

    let headers_obj = Object::new(ctx.clone())
        .map_err(|e| AppError::Script(format!("create request.headers: {e}")))?;
    for (k, v) in &req.headers {
        headers_obj
            .set(k.as_str(), v.as_str())
            .map_err(|e| AppError::Script(format!("set header {k}: {e}")))?;
    }
    req_obj
        .set("headers", headers_obj)
        .map_err(|e| AppError::Script(format!("set request.headers: {e}")))?;

    rl.set("request", req_obj)
        .map_err(|e| AppError::Script(format!("set rl.request: {e}")))?;
    Ok(())
}

pub(crate) fn register_response<'js>(
    ctx: &rquickjs::Ctx<'js>,
    rl: &Object<'js>,
    resp: &ScriptResponseData,
) -> Result<(), AppError> {
    let resp_obj = Object::new(ctx.clone())
        .map_err(|e| AppError::Script(format!("Failed to create response: {e}")))?;

    resp_obj
        .set("status", resp.status)
        .map_err(|e| AppError::Script(format!("set response.status: {e}")))?;
    resp_obj
        .set("body", resp.body.as_str())
        .map_err(|e| AppError::Script(format!("set response.body: {e}")))?;
    resp_obj
        .set("time", resp.time)
        .map_err(|e| AppError::Script(format!("set response.time: {e}")))?;

    let headers_obj = Object::new(ctx.clone())
        .map_err(|e| AppError::Script(format!("create response.headers: {e}")))?;
    for (k, v) in &resp.headers {
        headers_obj
            .set(k.as_str(), v.as_str())
            .map_err(|e| AppError::Script(format!("set resp header {k}: {e}")))?;
    }
    resp_obj
        .set("headers", headers_obj)
        .map_err(|e| AppError::Script(format!("set response.headers: {e}")))?;

    rl.set("response", resp_obj)
        .map_err(|e| AppError::Script(format!("set rl.response: {e}")))?;

    Ok(())
}

pub(crate) fn register_test_fn<'js>(
    ctx: &rquickjs::Ctx<'js>,
    rl: &Object<'js>,
    test_results: &Rc<RefCell<Vec<ScriptTestResult>>>,
) -> Result<(), AppError> {
    let results = test_results.clone();
    rl.set(
        "test",
        Function::new(ctx.clone(), move |name: String, func: Function<'_>| {
            let call_result: Result<(), rquickjs::Error> = func.call(());
            let (passed, message) = match call_result {
                Ok(()) => (true, None),
                Err(e) => (false, Some(format!("{e}"))),
            };
            results.borrow_mut().push(ScriptTestResult {
                name,
                passed,
                message,
            });
        })
        .map_err(|e| AppError::Script(format!("Failed to set rl.test: {e}")))?,
    )
    .map_err(|e| AppError::Script(format!("Failed to set rl.test: {e}")))?;

    Ok(())
}

/// Inject `rl.response.json()` wrapper via JS. Must be called AFTER `rl` is on globals.
pub(crate) fn inject_response_json(
    ctx: &rquickjs::Ctx<'_>,
    resp: &ScriptResponseData,
) -> Result<(), AppError> {
    let json_init = format!(
        "rl.response.json = function() {{ return JSON.parse({}); }};",
        serde_json::to_string(&resp.body).unwrap_or_else(|_| "\"\"".to_string())
    );
    ctx.eval::<(), _>(json_init)
        .map_err(|e| AppError::Script(format!("Failed to init response.json: {e}")))?;
    Ok(())
}

/// Inject `rl.expect()` chain via JS. Must be called AFTER `rl` is on globals.
pub(crate) fn inject_expect_api(ctx: &rquickjs::Ctx<'_>) -> Result<(), AppError> {
    let expect_js = r#"
rl.expect = function(actual) {
    return {
        toBe: function(expected) {
            if (actual !== expected) throw new Error("Expected " + JSON.stringify(expected) + " but got " + JSON.stringify(actual));
        },
        toEqual: function(expected) {
            if (JSON.stringify(actual) !== JSON.stringify(expected)) throw new Error("Expected " + JSON.stringify(expected) + " but got " + JSON.stringify(actual));
        },
        toContain: function(expected) {
            if (typeof actual === 'string') {
                if (actual.indexOf(expected) === -1) throw new Error("Expected to contain " + JSON.stringify(expected));
            } else if (Array.isArray(actual)) {
                var found = actual.some(function(item) { return JSON.stringify(item) === JSON.stringify(expected); });
                if (!found) throw new Error("Expected array to contain " + JSON.stringify(expected));
            }
        },
        toBeDefined: function() {
            if (actual === undefined || actual === null) throw new Error("Expected value to be defined");
        },
        toBeUndefined: function() {
            if (actual !== undefined) throw new Error("Expected undefined but got " + JSON.stringify(actual));
        },
        toBeGreaterThan: function(expected) {
            if (!(actual > expected)) throw new Error("Expected " + actual + " > " + expected);
        },
        toBeLessThan: function(expected) {
            if (!(actual < expected)) throw new Error("Expected " + actual + " < " + expected);
        },
        toBeTruthy: function() {
            if (!actual) throw new Error("Expected truthy value but got " + JSON.stringify(actual));
        }
    };
};
"#;
    ctx.eval::<(), _>(expect_js)
        .map_err(|e| AppError::Script(format!("Failed to init expect API: {e}")))?;
    Ok(())
}

pub(crate) fn register_crypto<'js>(
    ctx: &rquickjs::Ctx<'js>,
    globals: &Object<'js>,
) -> Result<(), AppError> {
    let crypto = Object::new(ctx.clone())
        .map_err(|e| AppError::Script(format!("Failed to create crypto: {e}")))?;

    crypto
        .set(
            "sha256",
            Function::new(ctx.clone(), |input: String| -> String {
                use sha2::{Digest, Sha256};
                let hash = Sha256::digest(input.as_bytes());
                hex_encode(&hash)
            })
            .map_err(|e| AppError::Script(format!("set crypto.sha256: {e}")))?,
        )
        .map_err(|e| AppError::Script(format!("set crypto.sha256: {e}")))?;

    crypto
        .set(
            "md5",
            Function::new(ctx.clone(), |input: String| -> String {
                use md5::Digest;
                let hash = md5::Md5::digest(input.as_bytes());
                hex_encode(&hash)
            })
            .map_err(|e| AppError::Script(format!("set crypto.md5: {e}")))?,
        )
        .map_err(|e| AppError::Script(format!("set crypto.md5: {e}")))?;

    crypto
        .set(
            "hmacSHA256",
            Function::new(ctx.clone(), |message: String, secret: String| -> String {
                use hmac::{Hmac, Mac};
                use sha2::Sha256;
                type HmacSha256 = Hmac<Sha256>;
                match HmacSha256::new_from_slice(secret.as_bytes()) {
                    Ok(mut mac) => {
                        mac.update(message.as_bytes());
                        hex_encode(&mac.finalize().into_bytes())
                    }
                    Err(_) => "HMAC_ERROR".to_string(),
                }
            })
            .map_err(|e| AppError::Script(format!("set crypto.hmacSHA256: {e}")))?,
        )
        .map_err(|e| AppError::Script(format!("set crypto.hmacSHA256: {e}")))?;

    globals
        .set("crypto", crypto)
        .map_err(|e| AppError::Script(format!("set crypto global: {e}")))?;
    Ok(())
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
