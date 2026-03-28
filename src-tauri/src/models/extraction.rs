use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A rule for extracting a value from a JSON response into an environment variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractionRule {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    #[serde(default)]
    pub variable_name: String,
    #[serde(default)]
    pub json_path: String,
    #[serde(default = "default_true")]
    pub is_enabled: bool,
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_roundtrip() {
        let rule = ExtractionRule {
            id: Uuid::new_v4(),
            variable_name: "token".to_string(),
            json_path: "$.data.token".to_string(),
            is_enabled: true,
        };
        let json = serde_json::to_string(&rule).unwrap();
        let parsed: ExtractionRule = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.variable_name, "token");
        assert_eq!(parsed.json_path, "$.data.token");
        assert!(parsed.is_enabled);
    }

    #[test]
    fn deserialize_empty_defaults() {
        let json = r#"{}"#;
        let rule: ExtractionRule = serde_json::from_str(json).unwrap();
        assert!(rule.variable_name.is_empty());
        assert!(rule.json_path.is_empty());
        assert!(rule.is_enabled);
    }
}
