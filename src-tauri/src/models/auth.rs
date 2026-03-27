use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// API Key injection location: header or query parameter.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ApiKeyLocation {
    #[default]
    Header,
    Query,
}

impl Serialize for ApiKeyLocation {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(match self {
            ApiKeyLocation::Header => "header",
            ApiKeyLocation::Query => "query",
        })
    }
}

impl<'de> Deserialize<'de> for ApiKeyLocation {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "query" => Ok(ApiKeyLocation::Query),
            _ => Ok(ApiKeyLocation::Header),
        }
    }
}

/// Authentication configuration for a request.
///
/// Serializes in Swift Codable compatible format:
///   AuthConfig::None        → {"none": {}}
///   AuthConfig::BearerToken → {"bearerToken": {"_0": {"token": "..."}}}
///   AuthConfig::BasicAuth   → {"basicAuth": {"_0": {"username": "...", "password": "..."}}}
///   AuthConfig::ApiKey      → {"apiKey": {"_0": {"key": "...", "value": "...", "location": "..."}}}
#[derive(Debug, Clone, Default, PartialEq)]
pub enum AuthConfig {
    #[default]
    None,
    BearerToken {
        token: String,
    },
    BasicAuth {
        username: String,
        password: String,
    },
    ApiKey {
        key: String,
        value: String,
        location: ApiKeyLocation,
    },
}

impl Serialize for AuthConfig {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(1))?;
        match self {
            AuthConfig::None => {
                map.serialize_entry("none", &serde_json::json!({}))?;
            }
            AuthConfig::BearerToken { token } => {
                map.serialize_entry("bearerToken", &serde_json::json!({"_0": {"token": token}}))?;
            }
            AuthConfig::BasicAuth { username, password } => {
                map.serialize_entry(
                    "basicAuth",
                    &serde_json::json!({"_0": {"username": username, "password": password}}),
                )?;
            }
            AuthConfig::ApiKey {
                key,
                value,
                location,
            } => {
                let loc_str = match location {
                    ApiKeyLocation::Header => "header",
                    ApiKeyLocation::Query => "query",
                };
                map.serialize_entry(
                    "apiKey",
                    &serde_json::json!({"_0": {"key": key, "value": value, "location": loc_str}}),
                )?;
            }
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for AuthConfig {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        let obj = value
            .as_object()
            .ok_or_else(|| serde::de::Error::custom("AuthConfig must be a JSON object"))?;

        if obj.contains_key("none") {
            Ok(AuthConfig::None)
        } else if let Some(inner) = obj.get("bearerToken") {
            let data = inner.get("_0").unwrap_or(inner);
            let token = data
                .get("token")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            Ok(AuthConfig::BearerToken { token })
        } else if let Some(inner) = obj.get("basicAuth") {
            let data = inner.get("_0").unwrap_or(inner);
            let username = data
                .get("username")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let password = data
                .get("password")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            Ok(AuthConfig::BasicAuth { username, password })
        } else if let Some(inner) = obj.get("apiKey") {
            let data = inner.get("_0").unwrap_or(inner);
            let key = data
                .get("key")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let value = data
                .get("value")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let location = match data
                .get("location")
                .and_then(|v| v.as_str())
                .unwrap_or("header")
            {
                "query" => ApiKeyLocation::Query,
                _ => ApiKeyLocation::Header,
            };
            Ok(AuthConfig::ApiKey {
                key,
                value,
                location,
            })
        } else {
            Ok(AuthConfig::None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_none_roundtrip() {
        let auth = AuthConfig::None;
        let json = serde_json::to_string(&auth).unwrap();
        let parsed: AuthConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, AuthConfig::None);
    }

    #[test]
    fn serde_bearer_roundtrip() {
        let auth = AuthConfig::BearerToken {
            token: "my-secret-token".to_string(),
        };
        let json = serde_json::to_string(&auth).unwrap();
        assert!(json.contains("bearerToken"));
        let parsed: AuthConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, auth);
    }

    #[test]
    fn serde_basic_auth_roundtrip() {
        let auth = AuthConfig::BasicAuth {
            username: "admin".to_string(),
            password: "p@ss".to_string(),
        };
        let json = serde_json::to_string(&auth).unwrap();
        assert!(json.contains("basicAuth"));
        let parsed: AuthConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, auth);
    }

    #[test]
    fn serde_api_key_header_roundtrip() {
        let auth = AuthConfig::ApiKey {
            key: "X-API-Key".to_string(),
            value: "abc123".to_string(),
            location: ApiKeyLocation::Header,
        };
        let json = serde_json::to_string(&auth).unwrap();
        assert!(json.contains("apiKey"));
        let parsed: AuthConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, auth);
    }

    #[test]
    fn serde_api_key_query_roundtrip() {
        let auth = AuthConfig::ApiKey {
            key: "api_key".to_string(),
            value: "xyz".to_string(),
            location: ApiKeyLocation::Query,
        };
        let json = serde_json::to_string(&auth).unwrap();
        let parsed: AuthConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, auth);
    }

    #[test]
    fn backward_compatible_missing_auth_defaults_to_none() {
        // Simulates existing SavedRequest JSON without "auth" field
        let json = r#"{"none":{}}"#;
        let parsed: AuthConfig = serde_json::from_str(json).unwrap();
        assert_eq!(parsed, AuthConfig::None);
    }

    #[test]
    fn unknown_variant_defaults_to_none() {
        let json = r#"{"unknownType":{"_0":{}}}"#;
        let parsed: AuthConfig = serde_json::from_str(json).unwrap();
        assert_eq!(parsed, AuthConfig::None);
    }
}
