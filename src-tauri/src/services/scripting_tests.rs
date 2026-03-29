use super::*;

fn empty_env() -> HashMap<String, String> {
    HashMap::new()
}

fn test_env() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("host".to_string(), "example.com".to_string());
    m.insert("token".to_string(), "abc123".to_string());
    m
}

fn dummy_request() -> ScriptRequestData {
    ScriptRequestData {
        method: "GET".to_string(),
        url: "https://example.com/api".to_string(),
        headers: HashMap::new(),
        body: String::new(),
    }
}

fn dummy_response() -> ScriptResponseData {
    ScriptResponseData {
        status: 200,
        headers: {
            let mut h = HashMap::new();
            h.insert("content-type".to_string(), "application/json".to_string());
            h
        },
        body: r#"{"data":{"id":42,"name":"Alice"}}"#.to_string(),
        time: 150.0,
    }
}

#[test]
fn env_get_returns_value() {
    let result = execute_pre_request(
        r#"console.log(rl.environment.get("host"));"#,
        &test_env(),
        &dummy_request(),
    )
    .unwrap();
    assert_eq!(result.console_output, vec!["example.com"]);
    assert!(result.error.is_none());
}

#[test]
fn env_set_records_update() {
    let result = execute_pre_request(
        r#"rl.environment.set("new_var", "hello");"#,
        &empty_env(),
        &dummy_request(),
    )
    .unwrap();
    assert_eq!(result.env_updates, vec![("new_var".into(), "hello".into())]);
}

#[test]
fn env_set_is_readable_in_same_script() {
    let result = execute_pre_request(
        r#"
        rl.environment.set("x", "42");
        console.log(rl.environment.get("x"));
        "#,
        &empty_env(),
        &dummy_request(),
    )
    .unwrap();
    assert_eq!(result.console_output, vec!["42"]);
}

#[test]
fn request_data_accessible() {
    let result = execute_pre_request(
        r#"console.log(rl.request.method + " " + rl.request.url);"#,
        &empty_env(),
        &dummy_request(),
    )
    .unwrap();
    assert_eq!(result.console_output, vec!["GET https://example.com/api"]);
}

#[test]
fn response_data_accessible() {
    let result = execute_test(
        r#"console.log(rl.response.status);"#,
        &empty_env(),
        &dummy_request(),
        &dummy_response(),
    )
    .unwrap();
    assert_eq!(result.console_output, vec!["200"]);
}

#[test]
fn response_json_works() {
    let result = execute_test(
        r#"
        let data = rl.response.json();
        console.log(data.data.name);
        "#,
        &empty_env(),
        &dummy_request(),
        &dummy_response(),
    )
    .unwrap();
    assert_eq!(result.console_output, vec!["Alice"]);
}

#[test]
fn test_passing_assertion() {
    let result = execute_test(
        r#"
        rl.test("Status is 200", function() {
            rl.expect(rl.response.status).toBe(200);
        });
        "#,
        &empty_env(),
        &dummy_request(),
        &dummy_response(),
    )
    .unwrap();
    assert_eq!(result.test_results.len(), 1);
    assert!(result.test_results[0].passed);
    assert_eq!(result.test_results[0].name, "Status is 200");
}

#[test]
fn test_failing_assertion() {
    let result = execute_test(
        r#"
        rl.test("Status is 404", function() {
            rl.expect(rl.response.status).toBe(404);
        });
        "#,
        &empty_env(),
        &dummy_request(),
        &dummy_response(),
    )
    .unwrap();
    assert_eq!(result.test_results.len(), 1);
    assert!(!result.test_results[0].passed);
    assert!(result.test_results[0].message.is_some());
}

#[test]
fn multiple_tests() {
    let result = execute_test(
        r#"
        rl.test("Has correct status", function() {
            rl.expect(rl.response.status).toBe(200);
        });
        rl.test("Response is fast", function() {
            rl.expect(rl.response.time).toBeLessThan(500);
        });
        rl.test("Has name", function() {
            let data = rl.response.json();
            rl.expect(data.data.name).toBe("Alice");
        });
        "#,
        &empty_env(),
        &dummy_request(),
        &dummy_response(),
    )
    .unwrap();
    assert_eq!(result.test_results.len(), 3);
    assert!(result.test_results.iter().all(|r| r.passed));
}

#[test]
fn crypto_sha256() {
    let result = execute_pre_request(
        r#"console.log(crypto.sha256("hello"));"#,
        &empty_env(),
        &dummy_request(),
    )
    .unwrap();
    assert_eq!(
        result.console_output[0],
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
}

#[test]
fn crypto_hmac_sha256() {
    let result = execute_pre_request(
        r#"console.log(crypto.hmacSHA256("message", "secret"));"#,
        &empty_env(),
        &dummy_request(),
    )
    .unwrap();
    assert!(!result.console_output[0].is_empty());
    assert!(result.error.is_none());
}

#[test]
fn crypto_md5() {
    let result = execute_pre_request(
        r#"console.log(crypto.md5("hello"));"#,
        &empty_env(),
        &dummy_request(),
    )
    .unwrap();
    assert_eq!(result.console_output[0], "5d41402abc4b2a76b9719d911017c592");
}

#[test]
fn script_error_captured() {
    let result =
        execute_pre_request(r#"undefinedFunction();"#, &empty_env(), &dummy_request()).unwrap();
    assert!(result.error.is_some());
}

#[test]
fn expect_to_contain() {
    let result = execute_test(
        r#"
        rl.test("Body contains Alice", function() {
            rl.expect(rl.response.body).toContain("Alice");
        });
        "#,
        &empty_env(),
        &dummy_request(),
        &dummy_response(),
    )
    .unwrap();
    assert!(result.test_results[0].passed);
}

#[test]
fn empty_script_succeeds() {
    let result = execute_pre_request("", &empty_env(), &dummy_request()).unwrap();
    assert!(result.error.is_none());
    assert!(result.console_output.is_empty());
}

#[test]
fn response_json_with_non_json_body() {
    let mut resp = dummy_response();
    resp.body = "not json at all".to_string();
    let result = execute_test(
        r#"
        try { rl.response.json(); } catch(e) { console.log("parse_error"); }
        "#,
        &empty_env(),
        &dummy_request(),
        &resp,
    )
    .unwrap();
    assert_eq!(result.console_output, vec!["parse_error"]);
}

#[test]
fn infinite_loop_times_out() {
    let result = execute_pre_request(r#"while(true) {}"#, &empty_env(), &dummy_request()).unwrap();
    assert!(result.error.is_some());
}

#[test]
fn expect_to_be_undefined() {
    let result = execute_test(
        r#"
        rl.test("undefined check", function() {
            rl.expect(undefined).toBeUndefined();
        });
        "#,
        &empty_env(),
        &dummy_request(),
        &dummy_response(),
    )
    .unwrap();
    assert!(result.test_results[0].passed);
}

#[test]
fn console_log_multiple_types() {
    let result = execute_pre_request(
        r#"console.log(42, true, "hello");"#,
        &empty_env(),
        &dummy_request(),
    )
    .unwrap();
    assert_eq!(result.console_output, vec!["42 true hello"]);
}

#[test]
fn env_updates_across_pre_request() {
    let result = execute_pre_request(
        r#"
        rl.environment.set("a", "1");
        rl.environment.set("b", "2");
        console.log(rl.environment.get("a") + rl.environment.get("b"));
        "#,
        &empty_env(),
        &dummy_request(),
    )
    .unwrap();
    assert_eq!(result.console_output, vec!["12"]);
    assert_eq!(result.env_updates.len(), 2);
}
