//! Output utilities for JSON-formatted CLI output.
//!
//! All lin commands output JSON for scriptability and easy parsing.

use serde::Serialize;

use crate::error::LinError;

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

/// Output a successful result as JSON to stdout.
///
/// # Arguments
///
/// * `data` - The data to serialize and output. Must implement `Serialize`.
///
/// # Example
///
/// ```
/// use serde::Serialize;
/// use lin::output::output_success;
///
/// #[derive(Serialize)]
/// struct MyData {
///     id: String,
///     name: String,
/// }
///
/// let data = MyData { id: "123".into(), name: "Test".into() };
/// output_success(&data);
/// ```
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
///
/// # Example
///
/// ```ignore
/// use lin::error::LinError;
/// use lin::output::output_error;
///
/// let err = LinError::config("missing API token");
/// output_error(&err); // Exits the program
/// ```
pub fn output_error(error: &LinError) -> ! {
    let response = ErrorResponse {
        success: false,
        error: ErrorDetail {
            kind: error.kind(),
            message: error.to_string(),
        },
    };

    // Unwrap is safe here because we control the types being serialized
    let json = serde_json::to_string_pretty(&response).expect("Failed to serialize error response");
    eprintln!("{}", json);
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
}
