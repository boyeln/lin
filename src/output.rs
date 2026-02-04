//! Output utilities for CLI output.
//!
//! Supports both human-friendly (default) and JSON output formats.
//! Human output includes colored formatting when writing to a terminal.

use colored::Colorize;
use serde::Serialize;

use crate::error::LinError;
use crate::models::{Issue, Team, User, WorkflowState};

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

impl HumanDisplay for User {
    fn human_fmt(&self) -> String {
        let name = self.name.bold();
        let status = if self.active {
            String::new()
        } else {
            format!(" {}", "(inactive)".red())
        };
        let display = self
            .display_name
            .as_ref()
            .map(|d| format!(" {}", format!("({})", d).dimmed()))
            .unwrap_or_default();
        format!("{}{}{}\n  {}", name, display, status, self.email.dimmed())
    }
}

impl HumanDisplay for Team {
    fn human_fmt(&self) -> String {
        let desc = self
            .description
            .as_ref()
            .map(|d| format!("\n  {}", d.dimmed()))
            .unwrap_or_default();
        format!(
            "{} {}{}",
            format!("[{}]", self.key).cyan(),
            self.name.bold(),
            desc
        )
    }
}

impl HumanDisplay for WorkflowState {
    fn human_fmt(&self) -> String {
        let type_label = match self.type_.as_str() {
            "backlog" => "Backlog",
            "unstarted" => "Unstarted",
            "started" => "Started",
            "completed" => "Completed",
            "canceled" => "Canceled",
            other => other,
        };
        format!(
            "{} [{}]\n  ID: {}\n  Color: {}",
            self.name, type_label, self.id, self.color
        )
    }
}

impl HumanDisplay for Issue {
    fn human_fmt(&self) -> String {
        let identifier = self.identifier.bold().cyan();
        let mut parts = vec![format!("{} {}", identifier, self.title)];

        if let Some(state) = &self.state {
            // Color status based on workflow state type
            let status_colored = match state.type_.as_str() {
                "completed" => state.name.green(),
                "canceled" => state.name.red().dimmed(),
                "started" => state.name.yellow(),
                "backlog" | "unstarted" => state.name.dimmed(),
                _ => state.name.normal(),
            };
            parts.push(format!("  {}: {}", "Status".dimmed(), status_colored));
        }

        // Color priority based on level
        let priority_colored = match self.priority {
            0 => None,
            1 => Some("Urgent".red().bold()),
            2 => Some("High".yellow()),
            3 => Some("Normal".normal()),
            4 => Some("Low".dimmed()),
            _ => Some("Unknown".normal()),
        };
        if let Some(p) = priority_colored {
            parts.push(format!("  {}: {}", "Priority".dimmed(), p));
        }

        if let Some(assignee) = &self.assignee {
            parts.push(format!("  {}: {}", "Assignee".dimmed(), assignee.name));
        }

        if let Some(team) = &self.team {
            parts.push(format!("  {}: {}", "Team".dimmed(), team.name));
        }

        parts.join("\n")
    }
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
    fn test_user_human_display() {
        let user = User {
            id: "user-123".to_string(),
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            display_name: Some("JD".to_string()),
            active: true,
        };
        let output = user.human_fmt();
        assert!(output.contains("John Doe"));
        assert!(output.contains("(JD)"));
        assert!(output.contains("john@example.com"));
    }

    #[test]
    fn test_user_human_display_inactive() {
        let user = User {
            id: "user-123".to_string(),
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            display_name: None,
            active: false,
        };
        let output = user.human_fmt();
        assert!(output.contains("(inactive)"));
    }

    #[test]
    fn test_team_human_display() {
        let team = Team {
            id: "team-123".to_string(),
            key: "ENG".to_string(),
            name: "Engineering".to_string(),
            description: Some("The engineering team".to_string()),
        };
        let output = team.human_fmt();
        assert!(output.contains("[ENG]"));
        assert!(output.contains("Engineering"));
        assert!(output.contains("The engineering team"));
    }

    #[test]
    fn test_issue_human_display() {
        use crate::models::WorkflowState;

        let issue = Issue {
            id: "issue-123".to_string(),
            identifier: "ENG-123".to_string(),
            title: "Fix the bug".to_string(),
            description: None,
            priority: 2,
            state: Some(WorkflowState {
                id: "state-1".to_string(),
                name: "In Progress".to_string(),
                color: "#0066ff".to_string(),
                type_: "started".to_string(),
            }),
            team: None,
            assignee: None,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-02".to_string(),
        };
        let output = issue.human_fmt();
        assert!(output.contains("ENG-123"));
        assert!(output.contains("Fix the bug"));
        assert!(output.contains("Status: In Progress"));
        assert!(output.contains("Priority: High"));
    }

    #[test]
    fn test_workflow_state_human_display() {
        let state = WorkflowState {
            id: "state-123".to_string(),
            name: "In Progress".to_string(),
            color: "#0066ff".to_string(),
            type_: "started".to_string(),
        };
        let output = state.human_fmt();
        assert!(output.contains("In Progress"));
        assert!(output.contains("[Started]"));
        assert!(output.contains("state-123"));
        assert!(output.contains("#0066ff"));
    }

    #[test]
    fn test_vec_human_display_empty() {
        let users: Vec<User> = vec![];
        let output = users.human_fmt();
        assert_eq!(output, "No results found.");
    }
}
