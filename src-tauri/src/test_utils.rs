use crate::models::KeyValuePair;
use uuid::Uuid;

/// Create an enabled `KeyValuePair` for use in tests.
pub fn make_kv(key: &str, value: &str) -> KeyValuePair {
    KeyValuePair {
        id: Uuid::new_v4(),
        key: key.to_string(),
        value: value.to_string(),
        is_enabled: true,
        is_secret: false,
    }
}

/// Create a disabled `KeyValuePair` for use in tests.
pub fn make_kv_disabled(key: &str, value: &str) -> KeyValuePair {
    KeyValuePair {
        id: Uuid::new_v4(),
        key: key.to_string(),
        value: value.to_string(),
        is_enabled: false,
        is_secret: false,
    }
}
