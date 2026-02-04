//! Custom error types for the lin CLI.

use thiserror::Error;

/// The main error type for the lin CLI.
///
/// All functions in this crate return `Result<T, LinError>` for consistent
/// error handling and JSON-formatted error output.
#[derive(Error, Debug)]
pub enum LinError {
    /// Configuration-related errors (missing token, invalid config file, etc.)
    #[error("Configuration error: {0}")]
    Config(String),

    /// API-related errors (network issues, authentication failures, etc.)
    #[error("API error: {0}")]
    Api(String),

    /// I/O errors (file operations, etc.)
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Parse errors (invalid JSON, malformed responses, etc.)
    #[error("Parse error: {0}")]
    Parse(String),
}

impl LinError {
    /// Create a new configuration error.
    pub fn config(msg: impl Into<String>) -> Self {
        LinError::Config(msg.into())
    }

    /// Create a new API error.
    pub fn api(msg: impl Into<String>) -> Self {
        LinError::Api(msg.into())
    }

    /// Create a new parse error.
    pub fn parse(msg: impl Into<String>) -> Self {
        LinError::Parse(msg.into())
    }

    /// Get the error kind as a string for JSON output.
    pub fn kind(&self) -> &'static str {
        match self {
            LinError::Config(_) => "config",
            LinError::Api(_) => "api",
            LinError::Io(_) => "io",
            LinError::Parse(_) => "parse",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error() {
        let err = LinError::config("missing API token");
        assert_eq!(err.kind(), "config");
        assert!(err.to_string().contains("missing API token"));
    }

    #[test]
    fn test_api_error() {
        let err = LinError::api("authentication failed");
        assert_eq!(err.kind(), "api");
        assert!(err.to_string().contains("authentication failed"));
    }

    #[test]
    fn test_parse_error() {
        let err = LinError::parse("invalid JSON");
        assert_eq!(err.kind(), "parse");
        assert!(err.to_string().contains("invalid JSON"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: LinError = io_err.into();
        assert_eq!(err.kind(), "io");
    }
}
