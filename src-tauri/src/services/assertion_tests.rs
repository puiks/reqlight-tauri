use super::*;
use crate::models::assertion::{AssertionOperator, AssertionRule, AssertionSource};
use crate::models::response::{HeaderPair, ResponseRecord};
use uuid::Uuid;

fn make_response() -> ResponseRecord {
    ResponseRecord {
        status_code: 200,
        headers: vec![
            HeaderPair {
                key: "content-type".to_string(),
                value: "application/json".to_string(),
            },
            HeaderPair {
                key: "x-request-id".to_string(),
                value: "abc123".to_string(),
            },
        ],
        body_string: Some(
            r#"{"data":{"id":42,"name":"Alice","tags":["admin","user"]}}"#.to_string(),
        ),
        elapsed_time: 150.0,
        body_size: 100,
        is_json: true,
        is_truncated: false,
        content_type: "application/json".to_string(),
    }
}

fn make_rule(
    source: AssertionSource,
    operator: AssertionOperator,
    expected: Option<&str>,
) -> AssertionRule {
    AssertionRule {
        id: Uuid::new_v4(),
        source,
        operator,
        expected: expected.map(String::from),
        is_enabled: true,
    }
}

#[test]
fn skips_disabled_rules() {
    let mut rule = make_rule(
        AssertionSource::StatusCode,
        AssertionOperator::Equals,
        Some("200"),
    );
    rule.is_enabled = false;
    let results = evaluate(&[rule], &make_response());
    assert!(results.is_empty());
}

#[test]
fn status_code_equals() {
    let rule = make_rule(
        AssertionSource::StatusCode,
        AssertionOperator::Equals,
        Some("200"),
    );
    let results = evaluate(&[rule], &make_response());
    assert!(results[0].passed);
}

#[test]
fn status_code_not_equals() {
    let rule = make_rule(
        AssertionSource::StatusCode,
        AssertionOperator::Equals,
        Some("404"),
    );
    let results = evaluate(&[rule], &make_response());
    assert!(!results[0].passed);
    assert_eq!(results[0].actual.as_deref(), Some("200"));
}

#[test]
fn response_time_less_than() {
    let rule = make_rule(
        AssertionSource::ResponseTime,
        AssertionOperator::LessThan,
        Some("2000"),
    );
    let results = evaluate(&[rule], &make_response());
    assert!(results[0].passed);
}

#[test]
fn response_time_exceeds() {
    let rule = make_rule(
        AssertionSource::ResponseTime,
        AssertionOperator::LessThan,
        Some("100"),
    );
    let results = evaluate(&[rule], &make_response());
    assert!(!results[0].passed);
}

#[test]
fn header_contains() {
    let rule = make_rule(
        AssertionSource::Header("content-type".to_string()),
        AssertionOperator::Contains,
        Some("json"),
    );
    let results = evaluate(&[rule], &make_response());
    assert!(results[0].passed);
}

#[test]
fn header_case_insensitive() {
    let rule = make_rule(
        AssertionSource::Header("Content-Type".to_string()),
        AssertionOperator::Contains,
        Some("json"),
    );
    let results = evaluate(&[rule], &make_response());
    assert!(results[0].passed);
}

#[test]
fn header_exists() {
    let rule = make_rule(
        AssertionSource::Header("x-request-id".to_string()),
        AssertionOperator::Exists,
        None,
    );
    let results = evaluate(&[rule], &make_response());
    assert!(results[0].passed);
}

#[test]
fn header_not_exists() {
    let rule = make_rule(
        AssertionSource::Header("x-nonexistent".to_string()),
        AssertionOperator::NotExists,
        None,
    );
    let results = evaluate(&[rule], &make_response());
    assert!(results[0].passed);
}

#[test]
fn json_path_equals() {
    let rule = make_rule(
        AssertionSource::JsonPath("$.data.name".to_string()),
        AssertionOperator::Equals,
        Some("Alice"),
    );
    let results = evaluate(&[rule], &make_response());
    assert!(results[0].passed);
}

#[test]
fn json_path_exists() {
    let rule = make_rule(
        AssertionSource::JsonPath("$.data.id".to_string()),
        AssertionOperator::Exists,
        None,
    );
    let results = evaluate(&[rule], &make_response());
    assert!(results[0].passed);
}

#[test]
fn json_path_not_exists() {
    let rule = make_rule(
        AssertionSource::JsonPath("$.data.missing".to_string()),
        AssertionOperator::NotExists,
        None,
    );
    let results = evaluate(&[rule], &make_response());
    assert!(results[0].passed);
}

#[test]
fn json_path_numeric_greater_than() {
    let rule = make_rule(
        AssertionSource::JsonPath("$.data.id".to_string()),
        AssertionOperator::GreaterThan,
        Some("10"),
    );
    let results = evaluate(&[rule], &make_response());
    assert!(results[0].passed);
}

#[test]
fn json_path_type_is_number() {
    let rule = make_rule(
        AssertionSource::JsonPath("$.data.id".to_string()),
        AssertionOperator::TypeIs,
        Some("number"),
    );
    let results = evaluate(&[rule], &make_response());
    assert!(results[0].passed);
}

#[test]
fn json_path_array_index() {
    let rule = make_rule(
        AssertionSource::JsonPath("$.data.tags[0]".to_string()),
        AssertionOperator::Equals,
        Some("admin"),
    );
    let results = evaluate(&[rule], &make_response());
    assert!(results[0].passed);
}

#[test]
fn body_contains_match() {
    let rule = make_rule(
        AssertionSource::BodyContains("Alice".to_string()),
        AssertionOperator::Contains,
        Some("Alice"),
    );
    let results = evaluate(&[rule], &make_response());
    assert!(results[0].passed);
}

#[test]
fn body_contains_no_match() {
    let rule = make_rule(
        AssertionSource::BodyContains("Bob".to_string()),
        AssertionOperator::Contains,
        Some("Bob"),
    );
    let results = evaluate(&[rule], &make_response());
    assert!(!results[0].passed);
}

#[test]
fn multiple_rules_evaluated_independently() {
    let rules = vec![
        make_rule(
            AssertionSource::StatusCode,
            AssertionOperator::Equals,
            Some("200"),
        ),
        make_rule(
            AssertionSource::ResponseTime,
            AssertionOperator::LessThan,
            Some("2000"),
        ),
        make_rule(
            AssertionSource::JsonPath("$.data.name".to_string()),
            AssertionOperator::Equals,
            Some("Bob"),
        ),
    ];
    let results = evaluate(&rules, &make_response());
    assert_eq!(results.len(), 3);
    assert!(results[0].passed);
    assert!(results[1].passed);
    assert!(!results[2].passed);
}
