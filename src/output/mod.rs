//! Output utilities for CLI output.
//!
//! Supports both human-friendly (default) and JSON output formats.
//! Human output includes colored formatting when writing to a terminal.

mod attachment;
mod cycle;
mod issue;
mod label;
mod milestone;
mod project;
mod team;
mod user;

use colored::Colorize;
use serde::Serialize;

use crate::error::LinError;

// Note: Submodules contain HumanDisplay implementations for domain types.
// The implementations are automatically available when the HumanDisplay trait is in scope.

/// Initialize color support based on terminal capabilities.
///
/// Respects the NO_COLOR environment variable (https://no-color.org/).
/// Disables colors when stdout is not a TTY (e.g., when piped).
pub fn init_colors() {
    // Disable colors if stdout is not a TTY (piped output)
    if !atty::is(atty::Stream::Stdout) {
        colored::control::set_override(false);
    }
    // The colored crate automatically respects NO_COLOR env var
}

/// Output format for CLI responses.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-friendly output (default)
    #[default]
    Human,
    /// JSON output for scriptability
    Json,
}

impl OutputFormat {
    /// Create an OutputFormat from a boolean flag.
    pub fn from_json_flag(json: bool) -> Self {
        if json {
            OutputFormat::Json
        } else {
            OutputFormat::Human
        }
    }
}

/// JSON wrapper for successful responses.
#[derive(Serialize)]
struct SuccessResponse<T: Serialize> {
    success: bool,
    data: T,
}

/// JSON wrapper for error responses.
#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error: ErrorDetail,
}

/// Error details for JSON output.
#[derive(Serialize)]
struct ErrorDetail {
    kind: &'static str,
    message: String,
}

/// Trait for types that can be displayed in human-friendly format.
pub trait HumanDisplay {
    /// Format the value for human-friendly output.
    fn human_fmt(&self) -> String;
}

impl<T: HumanDisplay> HumanDisplay for Vec<T> {
    fn human_fmt(&self) -> String {
        if self.is_empty() {
            "No results found.".dimmed().to_string()
        } else {
            self.iter()
                .map(|item| item.human_fmt())
                .collect::<Vec<_>>()
                .join("\n\n")
        }
    }
}

/// Output a successful result.
///
/// # Arguments
///
/// * `data` - The data to output. Must implement `Serialize` and `HumanDisplay`.
/// * `format` - The output format (Human or Json).
pub fn output<T: Serialize + HumanDisplay>(data: &T, format: OutputFormat) {
    match format {
        OutputFormat::Human => {
            println!("{}", data.human_fmt());
        }
        OutputFormat::Json => {
            let response = SuccessResponse {
                success: true,
                data,
            };
            let json = serde_json::to_string_pretty(&response)
                .expect("Failed to serialize success response");
            println!("{}", json);
        }
    }
}

/// Output a successful result as JSON to stdout (legacy function for compatibility).
///
/// # Arguments
///
/// * `data` - The data to serialize and output. Must implement `Serialize`.
pub fn output_success<T: Serialize>(data: &T) {
    let response = SuccessResponse {
        success: true,
        data,
    };

    // Unwrap is safe here because we control the types being serialized
    let json =
        serde_json::to_string_pretty(&response).expect("Failed to serialize success response");
    println!("{}", json);
}

/// Output an error as JSON to stderr and exit with code 1.
///
/// This function will terminate the program after printing the error.
///
/// # Arguments
///
/// * `error` - The error to format and output.
pub fn output_error(error: &LinError) -> ! {
    output_error_with_format(error, OutputFormat::Human)
}

/// Output an error and exit with code 1.
///
/// This function will terminate the program after printing the error.
///
/// # Arguments
///
/// * `error` - The error to format and output.
/// * `format` - The output format.
pub fn output_error_with_format(error: &LinError, format: OutputFormat) -> ! {
    match format {
        OutputFormat::Human => {
            eprintln!("{}: {}", "Error".red().bold(), error);
        }
        OutputFormat::Json => {
            let response = ErrorResponse {
                success: false,
                error: ErrorDetail {
                    kind: error.kind(),
                    message: error.to_string(),
                },
            };
            let json = serde_json::to_string_pretty(&response)
                .expect("Failed to serialize error response");
            eprintln!("{}", json);
        }
    }
    std::process::exit(1);
}

/// Output an error as JSON to stderr without exiting.
///
/// Use this when you need to handle the error programmatically after output.
///
/// # Arguments
///
/// * `error` - The error to format and output.
pub fn print_error(error: &LinError) {
    let response = ErrorResponse {
        success: false,
        error: ErrorDetail {
            kind: error.kind(),
            message: error.to_string(),
        },
    };

    let json = serde_json::to_string_pretty(&response).expect("Failed to serialize error response");
    eprintln!("{}", json);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize)]
    struct TestData {
        id: String,
        value: i32,
    }

    #[test]
    fn test_success_response_serialization() {
        let data = TestData {
            id: "test-123".into(),
            value: 42,
        };
        let response = SuccessResponse {
            success: true,
            data: &data,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"id\":\"test-123\""));
        assert!(json.contains("\"value\":42"));
    }

    #[test]
    fn test_error_response_serialization() {
        let err = LinError::config("test error");
        let response = ErrorResponse {
            success: false,
            error: ErrorDetail {
                kind: err.kind(),
                message: err.to_string(),
            },
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":false"));
        assert!(json.contains("\"kind\":\"config\""));
        assert!(json.contains("test error"));
    }

    #[test]
    fn test_output_format_from_json_flag() {
        assert_eq!(OutputFormat::from_json_flag(true), OutputFormat::Json);
        assert_eq!(OutputFormat::from_json_flag(false), OutputFormat::Human);
    }

    #[test]
    fn test_vec_human_display_empty() {
        let users: Vec<crate::models::User> = vec![];
        let output = users.human_fmt();
        assert_eq!(output, "No results found.");
    }
}
