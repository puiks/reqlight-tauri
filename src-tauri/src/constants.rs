/// Application name used for keychain service and log directories.
pub const APP_NAME: &str = "Reqlight";

/// Default HTTP request timeout in seconds.
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Maximum response body size read into memory (5 MB).
pub const MAX_RESPONSE_BODY_BYTES: usize = 5 * 1024 * 1024;

/// OAuth authorization code callback timeout in seconds.
pub const OAUTH_CALLBACK_TIMEOUT_SECS: u64 = 120;

/// Persisted state file name.
pub const DATA_FILE_NAME: &str = "data.json";
