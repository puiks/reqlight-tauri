use keyring::Entry;

const SERVICE_NAME: &str = "Reqlight";

// Unit tests are intentionally omitted for this module.
//
// `keychain.rs` is a thin wrapper around the `keyring` crate with no business logic —
// each function creates an `Entry` and calls a single method on it. Writing tests would
// either (a) test the `keyring` library itself (not our responsibility), or (b) require
// mocking `Entry` which would need a trait abstraction that adds complexity without value.
//
// The keychain integration is covered indirectly through the `persistence.rs` tests
// (which verify secret sanitization) and through manual/integration testing.

/// Save a secret value to the system credential store.
pub fn save(key: &str, value: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, key).map_err(|e| e.to_string())?;
    entry.set_password(value).map_err(|e| e.to_string())
}

/// Load a secret value from the system credential store.
/// Returns None if the key doesn't exist.
pub fn load(key: &str) -> Result<Option<String>, String> {
    let entry = Entry::new(SERVICE_NAME, key).map_err(|e| e.to_string())?;
    match entry.get_password() {
        Ok(v) => Ok(Some(v)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// Delete a secret value from the system credential store.
pub fn delete(key: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, key).map_err(|e| e.to_string())?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}
