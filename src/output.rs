//! Output utilities for CLI output.
//!
//! Supports both human-friendly (default) and JSON output formats.
//! Human output includes colored formatting when writing to a terminal.

use colored::Colorize;
use serde::Serialize;

use crate::error::LinError;
use crate::models::{
    Attachment, AttachmentWithIssue, Comment, Cycle, CycleWithIssues, Document,
    DocumentWithContent, FullIssueRelation, Issue, IssueWithComments, Label, NormalizedRelation,
    Project, Team, User, WorkflowState,
};

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

impl HumanDisplay for Label {
    fn human_fmt(&self) -> String {
        let group_indicator = if self.is_group {
            " [Group]".cyan().to_string()
        } else {
            String::new()
        };

        let mut parts = vec![format!("{}{}", self.name.bold(), group_indicator)];
        parts.push(format!("  {}: {}", "ID".dimmed(), self.id));
        parts.push(format!("  {}: {}", "Color".dimmed(), self.color));

        if let Some(desc) = &self.description {
            parts.push(format!("  {}: {}", "Description".dimmed(), desc));
        }

        parts.join("\n")
    }
}

impl HumanDisplay for Project {
    fn human_fmt(&self) -> String {
        // Color project state
        let state_colored = match self.state.as_str() {
            "completed" => self.state.green(),
            "canceled" => self.state.red().dimmed(),
            "started" => self.state.yellow(),
            "paused" => self.state.cyan(),
            "planned" | "backlog" => self.state.dimmed(),
            _ => self.state.normal(),
        };

        let mut parts = vec![format!("{}", self.name.bold())];
        parts.push(format!("  {}: {}", "State".dimmed(), state_colored));
        parts.push(format!("  {}: {:.0}%", "Progress".dimmed(), self.progress));

        if let Some(desc) = &self.description {
            parts.push(format!("  {}: {}", "Description".dimmed(), desc));
        }

        if let Some(start) = &self.start_date {
            parts.push(format!("  {}: {}", "Start".dimmed(), start));
        }

        if let Some(target) = &self.target_date {
            parts.push(format!("  {}: {}", "Target".dimmed(), target));
        }

        parts.push(format!("  {}: {}", "ID".dimmed(), self.id));

        parts.join("\n")
    }
}

impl HumanDisplay for Document {
    fn human_fmt(&self) -> String {
        let mut parts = vec![format!("{}", self.title.bold())];
        parts.push(format!("  {}: {}", "ID".dimmed(), self.id));

        if let Some(creator) = &self.creator {
            parts.push(format!("  {}: {}", "Creator".dimmed(), creator.name));
        }

        if let Some(project) = &self.project {
            parts.push(format!("  {}: {}", "Project".dimmed(), project.name));
        }

        // Format date (show date portion only)
        let date = if self.updated_at.len() >= 10 {
            &self.updated_at[..10]
        } else {
            &self.updated_at
        };
        parts.push(format!("  {}: {}", "Updated".dimmed(), date));

        parts.join("\n")
    }
}

impl HumanDisplay for DocumentWithContent {
    fn human_fmt(&self) -> String {
        let mut parts = vec![format!("{}", self.title.bold())];
        parts.push(format!("  {}: {}", "ID".dimmed(), self.id));

        if let Some(creator) = &self.creator {
            parts.push(format!("  {}: {}", "Creator".dimmed(), creator.name));
        }

        if let Some(project) = &self.project {
            parts.push(format!("  {}: {}", "Project".dimmed(), project.name));
        }

        // Format date (show date portion only)
        let date = if self.updated_at.len() >= 10 {
            &self.updated_at[..10]
        } else {
            &self.updated_at
        };
        parts.push(format!("  {}: {}", "Updated".dimmed(), date));

        // Add content section
        parts.push(format!("\n{}", "Content".bold()));
        if let Some(content) = &self.content {
            if content.is_empty() {
                parts.push("  (empty)".dimmed().to_string());
            } else {
                // Indent content for readability
                for line in content.lines() {
                    parts.push(format!("  {}", line));
                }
            }
        } else {
            parts.push("  (no content)".dimmed().to_string());
        }

        parts.join("\n")
    }
}

impl HumanDisplay for Attachment {
    fn human_fmt(&self) -> String {
        let mut parts = vec![format!("{}", self.title.bold())];
        parts.push(format!("  {}: {}", "ID".dimmed(), self.id));
        parts.push(format!("  {}: {}", "URL".dimmed(), self.url.cyan()));

        if let Some(subtitle) = &self.subtitle {
            parts.push(format!("  {}: {}", "Description".dimmed(), subtitle));
        }

        if let Some(creator) = &self.creator {
            parts.push(format!("  {}: {}", "Creator".dimmed(), creator.name));
        }

        // Format date (show date portion only)
        let date = if self.created_at.len() >= 10 {
            &self.created_at[..10]
        } else {
            &self.created_at
        };
        parts.push(format!("  {}: {}", "Created".dimmed(), date));

        parts.join("\n")
    }
}

impl HumanDisplay for AttachmentWithIssue {
    fn human_fmt(&self) -> String {
        let mut parts = vec![format!("{}", self.title.bold())];
        parts.push(format!("  {}: {}", "ID".dimmed(), self.id));
        parts.push(format!("  {}: {}", "URL".dimmed(), self.url.cyan()));

        if let Some(subtitle) = &self.subtitle {
            parts.push(format!("  {}: {}", "Description".dimmed(), subtitle));
        }

        if let Some(issue) = &self.issue {
            parts.push(format!(
                "  {}: {}",
                "Issue".dimmed(),
                issue.identifier.cyan()
            ));
        }

        if let Some(creator) = &self.creator {
            parts.push(format!("  {}: {}", "Creator".dimmed(), creator.name));
        }

        // Format date (show date portion only)
        let date = if self.created_at.len() >= 10 {
            &self.created_at[..10]
        } else {
            &self.created_at
        };
        parts.push(format!("  {}: {}", "Created".dimmed(), date));

        parts.join("\n")
    }
}

impl HumanDisplay for Cycle {
    fn human_fmt(&self) -> String {
        // Display cycle name or number
        let title = self
            .name
            .as_ref()
            .map(|n| format!("{}", n.bold()))
            .unwrap_or_else(|| format!("{}", format!("Cycle {}", self.number).bold()));

        let mut parts = vec![title];
        parts.push(format!("  {}: {:.0}%", "Progress".dimmed(), self.progress));

        if let Some(desc) = &self.description {
            parts.push(format!("  {}: {}", "Description".dimmed(), desc));
        }

        // Format dates (show date portion only)
        if let Some(starts) = &self.starts_at {
            let date = if starts.len() >= 10 {
                &starts[..10]
            } else {
                starts
            };
            parts.push(format!("  {}: {}", "Starts".dimmed(), date));
        }

        if let Some(ends) = &self.ends_at {
            let date = if ends.len() >= 10 { &ends[..10] } else { ends };
            parts.push(format!("  {}: {}", "Ends".dimmed(), date));
        }

        if let Some(completed) = &self.completed_at {
            let date = if completed.len() >= 10 {
                &completed[..10]
            } else {
                completed
            };
            parts.push(format!("  {}: {}", "Completed".dimmed(), date.green()));
        }

        parts.push(format!("  {}: {}", "ID".dimmed(), self.id));

        parts.join("\n")
    }
}

impl HumanDisplay for CycleWithIssues {
    fn human_fmt(&self) -> String {
        // Display cycle name or number
        let title = self
            .name
            .as_ref()
            .map(|n| format!("{}", n.bold()))
            .unwrap_or_else(|| format!("{}", format!("Cycle {}", self.number).bold()));

        let mut parts = vec![title];
        parts.push(format!("  {}: {:.0}%", "Progress".dimmed(), self.progress));

        if let Some(desc) = &self.description {
            parts.push(format!("  {}: {}", "Description".dimmed(), desc));
        }

        // Format dates (show date portion only)
        if let Some(starts) = &self.starts_at {
            let date = if starts.len() >= 10 {
                &starts[..10]
            } else {
                starts
            };
            parts.push(format!("  {}: {}", "Starts".dimmed(), date));
        }

        if let Some(ends) = &self.ends_at {
            let date = if ends.len() >= 10 { &ends[..10] } else { ends };
            parts.push(format!("  {}: {}", "Ends".dimmed(), date));
        }

        if let Some(completed) = &self.completed_at {
            let date = if completed.len() >= 10 {
                &completed[..10]
            } else {
                completed
            };
            parts.push(format!("  {}: {}", "Completed".dimmed(), date.green()));
        }

        parts.push(format!("  {}: {}", "ID".dimmed(), self.id));

        // Add issues section
        let issue_count = self.issues.nodes.len();
        parts.push(format!("\n  {} ({})", "Issues".bold(), issue_count));

        if self.issues.nodes.is_empty() {
            parts.push("  No issues in this cycle.".dimmed().to_string());
        } else {
            for issue in &self.issues.nodes {
                parts.push(String::new()); // blank line before each issue
                                           // Indent issue output
                for line in issue.human_fmt().lines() {
                    parts.push(format!("  {}", line));
                }
            }
        }

        parts.join("\n")
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

impl HumanDisplay for Comment {
    fn human_fmt(&self) -> String {
        let author = self
            .user
            .as_ref()
            .map(|u| u.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        // Format timestamp (show date portion only)
        let date = if self.created_at.len() >= 10 {
            &self.created_at[..10]
        } else {
            &self.created_at
        };

        let header = format!("{} {}", author.bold(), date.dimmed());

        // Indent the body for readability
        let body_lines: Vec<String> = self
            .body
            .lines()
            .map(|line| format!("  {}", line))
            .collect();
        let body = body_lines.join("\n");

        format!("{}\n{}", header, body)
    }
}

impl HumanDisplay for IssueWithComments {
    fn human_fmt(&self) -> String {
        let identifier = self.identifier.bold().cyan();
        let mut parts = vec![format!("{} {}", identifier, self.title)];

        if let Some(state) = &self.state {
            let status_colored = match state.type_.as_str() {
                "completed" => state.name.green(),
                "canceled" => state.name.red().dimmed(),
                "started" => state.name.yellow(),
                "backlog" | "unstarted" => state.name.dimmed(),
                _ => state.name.normal(),
            };
            parts.push(format!("  {}: {}", "Status".dimmed(), status_colored));
        }

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

        // Add comments section
        let comment_count = self.comments.nodes.len();
        parts.push(format!("\n  {} ({})", "Comments".bold(), comment_count));

        if self.comments.nodes.is_empty() {
            parts.push("  No comments yet.".dimmed().to_string());
        } else {
            for comment in &self.comments.nodes {
                parts.push(String::new()); // blank line before each comment
                                           // Indent comment output
                for line in comment.human_fmt().lines() {
                    parts.push(format!("  {}", line));
                }
            }
        }

        parts.join("\n")
    }
}

impl HumanDisplay for NormalizedRelation {
    fn human_fmt(&self) -> String {
        // Color the relation type based on its meaning
        let type_colored = match self.relation_type.as_str() {
            "parent" => "Parent".cyan().bold(),
            "child" => "Child".cyan(),
            "blocks" => "Blocks".red().bold(),
            "blocked_by" => "Blocked by".red(),
            "related" => "Related to".yellow(),
            "duplicate" => "Duplicate of".magenta(),
            other => other.normal(),
        };

        format!(
            "{} {} {}",
            type_colored,
            self.related_issue.identifier.bold().cyan(),
            self.related_issue.title
        )
    }
}

impl HumanDisplay for FullIssueRelation {
    fn human_fmt(&self) -> String {
        // Color the relation type
        let type_colored = match self.type_.as_str() {
            "blocks" => "blocks".red().bold(),
            "related" => "related to".yellow(),
            "duplicate" => "duplicate of".magenta(),
            other => other.normal(),
        };

        let source = self
            .issue
            .as_ref()
            .map(|i| format!("{} {}", i.identifier.bold().cyan(), i.title))
            .unwrap_or_else(|| "(unknown)".dimmed().to_string());

        let target = self
            .related_issue
            .as_ref()
            .map(|i| format!("{} {}", i.identifier.bold().cyan(), i.title))
            .unwrap_or_else(|| "(unknown)".dimmed().to_string());

        format!("{} {} {}", source, type_colored, target)
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

    #[test]
    fn test_label_human_display() {
        let label = Label {
            id: "label-123".to_string(),
            name: "Bug".to_string(),
            description: Some("Bug reports".to_string()),
            color: "#ff0000".to_string(),
            is_group: false,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-02".to_string(),
        };
        let output = label.human_fmt();
        assert!(output.contains("Bug"));
        assert!(output.contains("label-123"));
        assert!(output.contains("#ff0000"));
        assert!(output.contains("Bug reports"));
        assert!(!output.contains("[Group]"));
    }

    #[test]
    fn test_label_group_human_display() {
        let label = Label {
            id: "label-456".to_string(),
            name: "Feature".to_string(),
            description: None,
            color: "#00ff00".to_string(),
            is_group: true,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
        };
        let output = label.human_fmt();
        assert!(output.contains("Feature"));
        assert!(output.contains("[Group]"));
        assert!(output.contains("label-456"));
        assert!(output.contains("#00ff00"));
    }

    #[test]
    fn test_document_human_display() {
        use crate::models::DocumentProject;

        let document = Document {
            id: "doc-123".to_string(),
            title: "Project Overview".to_string(),
            icon: None,
            color: None,
            created_at: "2024-01-01T00:00:00.000Z".to_string(),
            updated_at: "2024-01-15T00:00:00.000Z".to_string(),
            creator: Some(User {
                id: "user-1".to_string(),
                name: "John Doe".to_string(),
                email: "john@example.com".to_string(),
                display_name: None,
                active: true,
            }),
            project: Some(DocumentProject {
                id: "project-1".to_string(),
                name: "Q1 Roadmap".to_string(),
            }),
        };
        let output = document.human_fmt();
        assert!(output.contains("Project Overview"));
        assert!(output.contains("doc-123"));
        assert!(output.contains("John Doe"));
        assert!(output.contains("Q1 Roadmap"));
        assert!(output.contains("2024-01-15"));
    }

    #[test]
    fn test_document_with_content_human_display() {
        let document = DocumentWithContent {
            id: "doc-789".to_string(),
            title: "Technical Spec".to_string(),
            content: Some("# Overview\n\nThis is the content.".to_string()),
            icon: None,
            color: None,
            created_at: "2024-01-01T00:00:00.000Z".to_string(),
            updated_at: "2024-01-20T00:00:00.000Z".to_string(),
            creator: None,
            project: None,
        };
        let output = document.human_fmt();
        assert!(output.contains("Technical Spec"));
        assert!(output.contains("doc-789"));
        assert!(output.contains("Content"));
        assert!(output.contains("# Overview"));
        assert!(output.contains("This is the content."));
    }

    #[test]
    fn test_document_with_empty_content_human_display() {
        let document = DocumentWithContent {
            id: "doc-empty".to_string(),
            title: "Empty Doc".to_string(),
            content: None,
            icon: None,
            color: None,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-01-01".to_string(),
            creator: None,
            project: None,
        };
        let output = document.human_fmt();
        assert!(output.contains("Empty Doc"));
        assert!(output.contains("(no content)"));
    }

    #[test]
    fn test_attachment_human_display() {
        let attachment = Attachment {
            id: "attach-123".to_string(),
            title: "Screenshot.png".to_string(),
            subtitle: Some("Bug screenshot".to_string()),
            url: "https://example.com/screenshot.png".to_string(),
            metadata: None,
            created_at: "2024-01-15T00:00:00.000Z".to_string(),
            updated_at: "2024-01-15T00:00:00.000Z".to_string(),
            creator: Some(User {
                id: "user-1".to_string(),
                name: "John Doe".to_string(),
                email: "john@example.com".to_string(),
                display_name: None,
                active: true,
            }),
        };
        let output = attachment.human_fmt();
        assert!(output.contains("Screenshot.png"));
        assert!(output.contains("attach-123"));
        assert!(output.contains("https://example.com/screenshot.png"));
        assert!(output.contains("Bug screenshot"));
        assert!(output.contains("John Doe"));
        assert!(output.contains("2024-01-15"));
    }

    #[test]
    fn test_attachment_with_issue_human_display() {
        use crate::models::AttachmentIssue;

        let attachment = AttachmentWithIssue {
            id: "attach-456".to_string(),
            title: "Log file".to_string(),
            subtitle: None,
            url: "https://example.com/log.txt".to_string(),
            metadata: None,
            created_at: "2024-01-20T00:00:00.000Z".to_string(),
            updated_at: "2024-01-20T00:00:00.000Z".to_string(),
            creator: None,
            issue: Some(AttachmentIssue {
                id: "issue-123".to_string(),
                identifier: "ENG-456".to_string(),
            }),
        };
        let output = attachment.human_fmt();
        assert!(output.contains("Log file"));
        assert!(output.contains("attach-456"));
        assert!(output.contains("https://example.com/log.txt"));
        assert!(output.contains("ENG-456"));
        assert!(output.contains("2024-01-20"));
    }

    #[test]
    fn test_normalized_relation_human_display() {
        use crate::models::RelatedIssue;

        let relation = NormalizedRelation {
            id: "rel-123".to_string(),
            relation_type: "blocks".to_string(),
            related_issue: RelatedIssue {
                id: "issue-456".to_string(),
                identifier: "ENG-456".to_string(),
                title: "Blocked issue".to_string(),
            },
        };
        let output = relation.human_fmt();
        assert!(output.contains("Blocks"));
        assert!(output.contains("ENG-456"));
        assert!(output.contains("Blocked issue"));
    }

    #[test]
    fn test_normalized_relation_parent_display() {
        use crate::models::RelatedIssue;

        let relation = NormalizedRelation {
            id: "parent:issue-100".to_string(),
            relation_type: "parent".to_string(),
            related_issue: RelatedIssue {
                id: "issue-100".to_string(),
                identifier: "ENG-100".to_string(),
                title: "Parent issue".to_string(),
            },
        };
        let output = relation.human_fmt();
        assert!(output.contains("Parent"));
        assert!(output.contains("ENG-100"));
        assert!(output.contains("Parent issue"));
    }

    #[test]
    fn test_full_issue_relation_human_display() {
        use crate::models::RelatedIssue;

        let relation = FullIssueRelation {
            id: "rel-new".to_string(),
            type_: "blocks".to_string(),
            issue: Some(RelatedIssue {
                id: "issue-1".to_string(),
                identifier: "ENG-1".to_string(),
                title: "Source issue".to_string(),
            }),
            related_issue: Some(RelatedIssue {
                id: "issue-2".to_string(),
                identifier: "ENG-2".to_string(),
                title: "Target issue".to_string(),
            }),
        };
        let output = relation.human_fmt();
        assert!(output.contains("ENG-1"));
        assert!(output.contains("Source issue"));
        assert!(output.contains("blocks"));
        assert!(output.contains("ENG-2"));
        assert!(output.contains("Target issue"));
    }
}
