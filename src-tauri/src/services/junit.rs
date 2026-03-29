use crate::services::assertion::AssertionResult;

/// A single test result for JUnit output.
pub struct TestResult {
    pub name: String,
    pub method: String,
    pub url: String,
    pub elapsed_secs: f64,
    pub passed: bool,
    pub error_message: Option<String>,
    pub assertion_results: Vec<AssertionResult>,
}

/// Generate JUnit XML from a list of test results.
pub fn generate_junit_xml(suite_name: &str, results: &[TestResult]) -> String {
    let total = results.len();
    let failures = results.iter().filter(|r| !r.passed).count();
    let total_time: f64 = results.iter().map(|r| r.elapsed_secs).sum();

    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str(&format!(
        "<testsuite name=\"{}\" tests=\"{}\" failures=\"{}\" time=\"{:.3}\">\n",
        escape_xml(suite_name),
        total,
        failures,
        total_time
    ));

    for result in results {
        xml.push_str(&format!(
            "  <testcase name=\"{} {}\" classname=\"{}\" time=\"{:.3}\"",
            escape_xml(&result.method),
            escape_xml(&result.name),
            escape_xml(suite_name),
            result.elapsed_secs
        ));

        if result.passed {
            xml.push_str(" />\n");
        } else {
            xml.push_str(">\n");
            let msg = build_failure_message(result);
            xml.push_str(&format!(
                "    <failure message=\"{}\">{}</failure>\n",
                escape_xml(&short_message(result)),
                escape_xml(&msg)
            ));
            xml.push_str("  </testcase>\n");
        }
    }

    xml.push_str("</testsuite>\n");
    xml
}

fn short_message(result: &TestResult) -> String {
    if let Some(ref err) = result.error_message {
        return err.clone();
    }
    let failed: Vec<_> = result
        .assertion_results
        .iter()
        .filter(|a| !a.passed)
        .map(|a| a.message.as_str())
        .collect();
    if failed.is_empty() {
        "Test failed".to_string()
    } else {
        failed.join("; ")
    }
}

fn build_failure_message(result: &TestResult) -> String {
    let mut lines = Vec::new();
    lines.push(format!("URL: {} {}", result.method, result.url));
    if let Some(ref err) = result.error_message {
        lines.push(format!("Error: {err}"));
    }
    for ar in &result.assertion_results {
        let status = if ar.passed { "PASS" } else { "FAIL" };
        lines.push(format!(
            "[{status}] {} (actual: {})",
            ar.message,
            ar.actual.as_deref().unwrap_or("N/A")
        ));
    }
    lines.join("\n")
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn generates_valid_xml_for_passing_tests() {
        let results = vec![TestResult {
            name: "Get Users".to_string(),
            method: "GET".to_string(),
            url: "https://api.example.com/users".to_string(),
            elapsed_secs: 0.15,
            passed: true,
            error_message: None,
            assertion_results: vec![],
        }];
        let xml = generate_junit_xml("My API", &results);
        assert!(xml.contains("tests=\"1\""));
        assert!(xml.contains("failures=\"0\""));
        assert!(xml.contains("GET Get Users"));
        assert!(xml.contains("/>"));
    }

    #[test]
    fn generates_failure_element_for_failed_tests() {
        let results = vec![TestResult {
            name: "Create User".to_string(),
            method: "POST".to_string(),
            url: "https://api.example.com/users".to_string(),
            elapsed_secs: 0.3,
            passed: false,
            error_message: None,
            assertion_results: vec![AssertionResult {
                rule_id: Uuid::new_v4(),
                passed: false,
                actual: Some("500".to_string()),
                expected: Some("201".to_string()),
                message: "Expected == 201, got 500".to_string(),
            }],
        }];
        let xml = generate_junit_xml("My API", &results);
        assert!(xml.contains("failures=\"1\""));
        assert!(xml.contains("<failure"));
        assert!(xml.contains("Expected == 201, got 500"));
    }

    #[test]
    fn escapes_special_xml_characters() {
        let results = vec![TestResult {
            name: "Test <special> & \"chars\"".to_string(),
            method: "GET".to_string(),
            url: "https://example.com?a=1&b=2".to_string(),
            elapsed_secs: 0.1,
            passed: true,
            error_message: None,
            assertion_results: vec![],
        }];
        let xml = generate_junit_xml("Suite", &results);
        assert!(xml.contains("&lt;special&gt;"));
        assert!(xml.contains("&amp;"));
    }
}
