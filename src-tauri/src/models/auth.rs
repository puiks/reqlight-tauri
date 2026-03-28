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
    OAuth2 {
        grant_type: String, // "authorization_code" or "client_credentials"
        client_id: String,
        client_secret: String,
        auth_url: String,
        token_url: String,
        scopes: String,
        access_token: String,
        refresh_token: String,
        token_expiry: Option<String>, // ISO8601
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
            AuthConfig::OAuth2 {
                grant_type,
                client_id,
                client_secret,
                auth_url,
                token_url,
                scopes,
                access_token,
                refresh_token,
                token_expiry,
            } => {
                map.serialize_entry(
                    "oauth2",
                    &serde_json::json!({
                        "grantType": grant_type,
                        "clientId": client_id,
                        "clientSecret": client_secret,
                        "authUrl": auth_url,
                        "tokenUrl": token_url,
                        "scopes": scopes,
                        "accessToken": access_token,
                        "refreshToken": refresh_token,
                        "tokenExpiry": token_expiry,
                    }),
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
        } else if let Some(inner) = obj.get("oauth2") {
            let s = |key: &str| -> String {
                inner
                    .get(key)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string()
            };
            Ok(AuthConfig::OAuth2 {
                grant_type: s("grantType"),
                client_id: s("clientId"),
                client_secret: s("clientSecret"),
                auth_url: s("authUrl"),
                token_url: s("tokenUrl"),
                scopes: s("scopes"),
                access_token: s("accessToken"),
                refresh_token: s("refreshToken"),
                token_expiry: inner
                    .get("tokenExpiry")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            })
        } else {
            Ok(AuthConfig::None)
        }
    }
}

#[cfg(test)]
#[path = "auth_tests.rs"]
mod tests;
