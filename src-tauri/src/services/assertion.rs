use crate::models::assertion::{AssertionOperator, AssertionRule, AssertionSource};
use crate::models::response::ResponseRecord;

/// Result of evaluating a single assertion.
#[derive(Debug, Clone)]
pub struct AssertionResult {
    pub rule_id: uuid::Uuid,
    pub passed: bool,
    pub actual: Option<String>,
    pub expected: Option<String>,
    pub message: String,
}

/// Evaluate all enabled assertion rules against a response.
pub fn evaluate(rules: &[AssertionRule], response: &ResponseRecord) -> Vec<AssertionResult> {
    rules
        .iter()
        .filter(|r| r.is_enabled)
        .map(|rule| evaluate_one(rule, response))
        .collect()
}

fn evaluate_one(rule: &AssertionRule, response: &ResponseRecord) -> AssertionResult {
    let expected = rule.expected.clone();

    let actual = resolve_source(&rule.source, response);
    let passed = compare(actual.as_deref(), &rule.operator, expected.as_deref());

    let message = if passed {
        "Passed".to_string()
    } else {
        format!(
            "Expected {} {}, got {}",
            format_operator(&rule.operator),
            expected.as_deref().unwrap_or(""),
            actual.as_deref().unwrap_or("undefined")
        )
    };

    AssertionResult {
        rule_id: rule.id,
        passed,
        actual,
        expected,
        message,
    }
}

fn resolve_source(source: &AssertionSource, response: &ResponseRecord) -> Option<String> {
    match source {
        AssertionSource::StatusCode => Some(response.status_code.to_string()),
        AssertionSource::ResponseTime => Some(response.elapsed_time.round().to_string()),
        AssertionSource::Header(name) => response
            .headers
            .iter()
            .find(|h| h.key.eq_ignore_ascii_case(name))
            .map(|h| h.value.clone()),
        AssertionSource::JsonPath(path) => {
            let body = response.body_string.as_deref()?;
            let parsed: serde_json::Value = serde_json::from_str(body).ok()?;
            extract_json_path(&parsed, path)
        }
        AssertionSource::BodyContains(_) => response.body_string.clone(),
    }
}

fn compare(actual: Option<&str>, operator: &AssertionOperator, expected: Option<&str>) -> bool {
    match operator {
        AssertionOperator::Exists => actual.is_some(),
        AssertionOperator::NotExists => actual.is_none(),
        AssertionOperator::Equals => actual == expected,
        AssertionOperator::NotEquals => actual != expected,
        AssertionOperator::Contains => {
            actual.is_some_and(|a| expected.is_some_and(|e| a.contains(e)))
        }
        AssertionOperator::NotContains => {
            actual.is_some_and(|a| expected.is_none_or(|e| !a.contains(e)))
        }
        AssertionOperator::GreaterThan => compare_numeric(actual, expected, |a, e| a > e),
        AssertionOperator::LessThan => compare_numeric(actual, expected, |a, e| a < e),
        AssertionOperator::TypeIs => {
            let t = actual.map(value_type).unwrap_or("null");
            expected.is_some_and(|e| e == t)
        }
    }
}

fn compare_numeric(actual: Option<&str>, expected: Option<&str>, op: fn(f64, f64) -> bool) -> bool {
    let a = actual.and_then(|s| s.parse::<f64>().ok());
    let e = expected.and_then(|s| s.parse::<f64>().ok());
    matches!((a, e), (Some(a), Some(e)) if op(a, e))
}

fn value_type(s: &str) -> &'static str {
    if s == "null" {
        return "null";
    }
    if s == "true" || s == "false" {
        return "boolean";
    }
    if s.parse::<f64>().is_ok() {
        return "number";
    }
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(s) {
        return match v {
            serde_json::Value::Array(_) => "array",
            serde_json::Value::Object(_) => "object",
            _ => "string",
        };
    }
    "string"
}

fn format_operator(op: &AssertionOperator) -> &'static str {
    match op {
        AssertionOperator::Equals => "==",
        AssertionOperator::NotEquals => "!=",
        AssertionOperator::Contains => "contains",
        AssertionOperator::NotContains => "not contains",
        AssertionOperator::GreaterThan => ">",
        AssertionOperator::LessThan => "<",
        AssertionOperator::Exists => "exists",
        AssertionOperator::NotExists => "not exists",
        AssertionOperator::TypeIs => "type is",
    }
}

/// Minimal JSONPath extraction: supports $.foo.bar, $.items[0].name
fn extract_json_path(value: &serde_json::Value, path: &str) -> Option<String> {
    let normalized = path.strip_prefix("$.").or_else(|| path.strip_prefix('$'))?;
    if normalized.is_empty() {
        return Some(stringify_json(value));
    }

    let mut current = value;
    for token in tokenize_path(normalized) {
        match parse_token(&token) {
            Token::Key(key) => {
                current = current.get(key)?;
            }
            Token::Index(key, idx) => {
                if !key.is_empty() {
                    current = current.get(key)?;
                }
                current = current.get(idx)?;
            }
        }
    }

    Some(stringify_json(current))
}

enum Token<'a> {
    Key(&'a str),
    Index(&'a str, usize),
}

fn parse_token(token: &str) -> Token<'_> {
    if let Some(bracket_start) = token.find('[') {
        let key = &token[..bracket_start];
        let idx_str = &token[bracket_start + 1..token.len() - 1]; // strip []
        if let Ok(idx) = idx_str.parse::<usize>() {
            return Token::Index(key, idx);
        }
    }
    Token::Key(token)
}

fn tokenize_path(path: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_bracket = false;
    for ch in path.chars() {
        if ch == '[' {
            in_bracket = true;
        }
        if ch == ']' {
            in_bracket = false;
        }
        if ch == '.' && !in_bracket {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn stringify_json(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
        _ => value.to_string(),
    }
}

#[cfg(test)]
#[path = "assertion_tests.rs"]
mod tests;
