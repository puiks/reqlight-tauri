use thiserror::Error;

/// Structured error type for Reqlight services.
///
/// Commands layer converts these to `String` at the IPC boundary via `Display`.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Request timed out")]
    Timeout,

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Auth error: {0}")]
    Auth(String),

    #[error("Keychain error: {0}")]
    Keychain(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Script error: {0}")]
    Script(String),

    #[error("{0}")]
    Other(String),
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            AppError::Timeout
        } else {
            AppError::Network(e.to_string())
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Serialization(e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}

impl From<AppError> for String {
    fn from(e: AppError) -> Self {
        e.to_string()
    }
}
