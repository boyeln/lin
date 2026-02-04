//! Issue management commands.
//!
//! Commands for listing and viewing issue information from Linear.

use crate::api::{queries, GraphQLClient};
use crate::error::LinError;
use crate::models::{
    IssueArchiveResponse, IssueCreateResponse, IssueDeleteResponse, IssueResponse,
    IssueUnarchiveResponse, IssueUpdateResponse, IssueWithCommentsResponse, IssuesResponse,
    IssuesWithCommentsResponse,
};
use crate::output::{output, HumanDisplay, OutputFormat};
use crate::Result;
use serde::Serialize;

/// Options for listing issues.
#[derive(Debug, Clone, Default)]
pub struct IssueListOptions {
    /// Filter by team key (e.g., "ENG").
    pub team: Option<String>,
    /// Filter by assignee ID or "me".
    pub assignee: Option<String>,
    /// Filter by state name.
    pub state: Option<String>,
    /// Filter by project ID.
    pub project: Option<String>,
    /// Maximum number of issues to return (default 50).
    pub limit: Option<i32>,
}

/// Options for creating a new issue.
#[derive(Debug, Clone)]
pub struct IssueCreateOptions {
    /// Issue title (required).
    pub title: String,
    /// Team ID (required).
    pub team_id: String,
    /// Issue description.
    pub description: Option<String>,
    /// Assignee user ID.
    pub assignee_id: Option<String>,
    /// Initial workflow state ID.
    pub state_id: Option<String>,
    /// Priority level (0=none, 1=urgent, 2=high, 3=normal, 4=low).
    pub priority: Option<i32>,
}

/// Options for updating an existing issue.
#[derive(Debug, Clone, Default)]
pub struct IssueUpdateOptions {
    /// New title.
    pub title: Option<String>,
    /// New description.
    pub description: Option<String>,
    /// New assignee user ID.
    pub assignee_id: Option<String>,
    /// New workflow state ID.
    pub state_id: Option<String>,
    /// New priority level (0=none, 1=urgent, 2=high, 3=normal, 4=low).
    pub priority: Option<i32>,
}

/// Check if a string looks like a UUID.
///
/// UUIDs are typically 36 characters with hyphens (8-4-4-4-12 format)
/// or 32 hex characters without hyphens.
///
/// # Arguments
///
/// * `s` - The string to check
///
/// # Returns
///
/// `true` if the string looks like a UUID, `false` otherwise.
///
/// # Example
///
/// ```
/// use lin::commands::issue::is_uuid;
///
/// assert!(is_uuid("550e8400-e29b-41d4-a716-446655440000"));
/// assert!(is_uuid("550e8400e29b41d4a716446655440000"));
/// assert!(!is_uuid("ENG-123"));
/// assert!(!is_uuid("ABC"));
/// ```
pub fn is_uuid(s: &str) -> bool {
    // Standard UUID format: 8-4-4-4-12 (36 chars with hyphens)
    if s.len() == 36 {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() == 5 {
            return parts[0].len() == 8
                && parts[1].len() == 4
                && parts[2].len() == 4
                && parts[3].len() == 4
                && parts[4].len() == 12
                && s.chars()
                    .filter(|c| *c != '-')
                    .all(|c| c.is_ascii_hexdigit());
        }
    }

    // UUID without hyphens: 32 hex characters
    if s.len() == 32 {
        return s.chars().all(|c| c.is_ascii_hexdigit());
    }

    false
}

/// Parse an issue identifier in the format "TEAM-NUMBER" (e.g., "ENG-123").
///
/// # Arguments
///
/// * `s` - The identifier string to parse
///
/// # Returns
///
/// A tuple of (team_key, issue_number) on success.
///
/// # Errors
///
/// Returns `LinError::Parse` if the string is not a valid identifier format.
///
/// # Example
///
/// ```
/// use lin::commands::issue::parse_identifier;
///
/// let (team, num) = parse_identifier("ENG-123").unwrap();
/// assert_eq!(team, "ENG");
/// assert_eq!(num, 123);
/// ```
pub fn parse_identifier(s: &str) -> Result<(String, i32)> {
    // Find the last hyphen (team keys might have hyphens in the future, though typically don't)
    let parts: Vec<&str> = s.split('-').collect();

    if parts.len() < 2 {
        return Err(LinError::parse(format!(
            "Invalid identifier format '{}': expected TEAM-NUMBER (e.g., ENG-123)",
            s
        )));
    }

    // Team key is everything before the last hyphen
    let team_parts = &parts[..parts.len() - 1];
    let team_key = team_parts.join("-");
    let number_str = parts[parts.len() - 1];

    // Validate team key: should be uppercase letters (and possibly hyphens between letters)
    // Don't allow empty team key, leading/trailing hyphens, or consecutive hyphens
    if team_key.is_empty()
        || team_key.starts_with('-')
        || team_key.ends_with('-')
        || team_key.contains("--")
        || !team_key.chars().all(|c| c.is_ascii_uppercase() || c == '-')
    {
        return Err(LinError::parse(format!(
            "Invalid team key '{}': expected uppercase letters (e.g., ENG, ABC-DEF)",
            team_key
        )));
    }

    // Parse the number
    let number: i32 = number_str.parse().map_err(|_| {
        LinError::parse(format!(
            "Invalid issue number '{}': expected an integer",
            number_str
        ))
    })?;

    if number <= 0 {
        return Err(LinError::parse(format!(
            "Invalid issue number '{}': must be positive",
            number
        )));
    }

    Ok((team_key, number))
}

/// List issues with optional filters.
///
/// Fetches issues from the Linear API and outputs them.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `viewer_id` - The current user's ID (used if assignee is "me")
/// * `options` - Filter options for the query
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```ignore
/// use lin::api::GraphQLClient;
/// use lin::commands::issue::{list_issues, IssueListOptions};
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// let options = IssueListOptions {
///     team: Some("ENG".to_string()),
///     assignee: None,
///     state: None,
///     limit: Some(10),
/// };
/// list_issues(&client, None, options, OutputFormat::Human)?;
/// ```
pub fn list_issues(
    client: &GraphQLClient,
    viewer_id: Option<&str>,
    options: IssueListOptions,
    format: OutputFormat,
) -> Result<()> {
    // Build the filter object
    let mut filter = serde_json::Map::new();

    // Add team filter if specified
    if let Some(team_key) = &options.team {
        filter.insert(
            "team".to_string(),
            serde_json::json!({ "key": { "eq": team_key } }),
        );
    }

    // Add assignee filter if specified
    if let Some(assignee) = &options.assignee {
        let assignee_id = if assignee.to_lowercase() == "me" {
            viewer_id
                .ok_or_else(|| {
                    LinError::config(
                        "Cannot use 'me' as assignee without viewer ID. Please authenticate first.",
                    )
                })?
                .to_string()
        } else {
            assignee.clone()
        };
        filter.insert(
            "assignee".to_string(),
            serde_json::json!({ "id": { "eq": assignee_id } }),
        );
    }

    // Add state filter if specified
    if let Some(state_name) = &options.state {
        filter.insert(
            "state".to_string(),
            serde_json::json!({ "name": { "eq": state_name } }),
        );
    }

    // Add project filter if specified
    if let Some(project_id) = &options.project {
        filter.insert(
            "project".to_string(),
            serde_json::json!({ "id": { "eq": project_id } }),
        );
    }

    // Build variables
    let mut variables = serde_json::Map::new();
    variables.insert(
        "first".to_string(),
        serde_json::json!(options.limit.unwrap_or(50)),
    );

    if !filter.is_empty() {
        variables.insert("filter".to_string(), serde_json::Value::Object(filter));
    }

    let response: IssuesResponse =
        client.query(queries::ISSUES_QUERY, serde_json::Value::Object(variables))?;

    output(&response.issues.nodes, format);
    Ok(())
}

/// Get details of a specific issue by ID or identifier.
///
/// Fetches a single issue from the Linear API and outputs it.
/// Supports both UUID format (e.g., "550e8400-e29b-41d4-a716-446655440000")
/// and identifier format (e.g., "ENG-123").
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `id_or_identifier` - The issue's UUID or human-readable identifier
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```ignore
/// use lin::api::GraphQLClient;
/// use lin::commands::issue::get_issue;
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
///
/// // By UUID
/// get_issue(&client, "550e8400-e29b-41d4-a716-446655440000", OutputFormat::Human)?;
///
/// // By identifier
/// get_issue(&client, "ENG-123", OutputFormat::Human)?;
/// ```
pub fn get_issue(
    client: &GraphQLClient,
    id_or_identifier: &str,
    format: OutputFormat,
) -> Result<()> {
    get_issue_impl(client, id_or_identifier, false, format)
}

/// Get details of a specific issue by ID or identifier, optionally with comments.
///
/// Fetches a single issue from the Linear API and outputs it.
/// When `with_comments` is true, also fetches and displays comments on the issue.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `id_or_identifier` - The issue's UUID or human-readable identifier
/// * `with_comments` - Whether to include comments in the output
/// * `format` - The output format (Human or Json)
pub fn get_issue_with_comments(
    client: &GraphQLClient,
    id_or_identifier: &str,
    with_comments: bool,
    format: OutputFormat,
) -> Result<()> {
    get_issue_impl(client, id_or_identifier, with_comments, format)
}

fn get_issue_impl(
    client: &GraphQLClient,
    id_or_identifier: &str,
    with_comments: bool,
    format: OutputFormat,
) -> Result<()> {
    if with_comments {
        get_issue_with_comments_impl(client, id_or_identifier, format)
    } else {
        get_issue_without_comments(client, id_or_identifier, format)
    }
}

fn get_issue_without_comments(
    client: &GraphQLClient,
    id_or_identifier: &str,
    format: OutputFormat,
) -> Result<()> {
    if is_uuid(id_or_identifier) {
        // Query by UUID
        let variables = serde_json::json!({
            "id": id_or_identifier
        });
        let response: IssueResponse = client.query(queries::ISSUE_QUERY, variables)?;
        output(&response.issue, format);
    } else {
        // Parse as identifier and query
        let (team_key, number) = parse_identifier(id_or_identifier)?;

        // Build filter to find issue by team key and number
        let variables = serde_json::json!({
            "filter": {
                "team": { "key": { "eq": team_key } },
                "number": { "eq": number }
            }
        });

        let response: IssuesResponse =
            client.query(queries::ISSUE_BY_IDENTIFIER_QUERY, variables)?;

        if response.issues.nodes.is_empty() {
            return Err(LinError::api(format!(
                "Issue '{}' not found",
                id_or_identifier
            )));
        }

        output(&response.issues.nodes[0], format);
    }

    Ok(())
}

fn get_issue_with_comments_impl(
    client: &GraphQLClient,
    id_or_identifier: &str,
    format: OutputFormat,
) -> Result<()> {
    if is_uuid(id_or_identifier) {
        // Query by UUID with comments
        let variables = serde_json::json!({
            "id": id_or_identifier
        });
        let response: IssueWithCommentsResponse =
            client.query(queries::ISSUE_WITH_COMMENTS_QUERY, variables)?;
        output(&response.issue, format);
    } else {
        // Parse as identifier and query with comments
        let (team_key, number) = parse_identifier(id_or_identifier)?;

        // Build filter to find issue by team key and number
        let variables = serde_json::json!({
            "filter": {
                "team": { "key": { "eq": team_key } },
                "number": { "eq": number }
            }
        });

        let response: IssuesWithCommentsResponse =
            client.query(queries::ISSUE_BY_IDENTIFIER_WITH_COMMENTS_QUERY, variables)?;

        if response.issues.nodes.is_empty() {
            return Err(LinError::api(format!(
                "Issue '{}' not found",
                id_or_identifier
            )));
        }

        output(&response.issues.nodes[0], format);
    }

    Ok(())
}

/// Create a new issue in Linear.
///
/// Creates an issue with the specified options and outputs the created issue.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `options` - Options for the new issue (title and team_id are required)
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```ignore
/// use lin::api::GraphQLClient;
/// use lin::commands::issue::{create_issue, IssueCreateOptions};
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// let options = IssueCreateOptions {
///     title: "Fix the bug".to_string(),
///     team_id: "team-123".to_string(),
///     description: Some("Detailed description".to_string()),
///     assignee_id: None,
///     state_id: None,
///     priority: Some(2), // High priority
/// };
/// create_issue(&client, options, OutputFormat::Human)?;
/// ```
pub fn create_issue(
    client: &GraphQLClient,
    options: IssueCreateOptions,
    format: OutputFormat,
) -> Result<()> {
    // Build the input object for the mutation
    let mut input = serde_json::Map::new();
    input.insert("title".to_string(), serde_json::json!(options.title));
    input.insert("teamId".to_string(), serde_json::json!(options.team_id));

    if let Some(description) = options.description {
        input.insert("description".to_string(), serde_json::json!(description));
    }

    if let Some(assignee_id) = options.assignee_id {
        input.insert("assigneeId".to_string(), serde_json::json!(assignee_id));
    }

    if let Some(state_id) = options.state_id {
        input.insert("stateId".to_string(), serde_json::json!(state_id));
    }

    if let Some(priority) = options.priority {
        input.insert("priority".to_string(), serde_json::json!(priority));
    }

    let variables = serde_json::json!({
        "input": input
    });

    let response: IssueCreateResponse = client.query(queries::ISSUE_CREATE_MUTATION, variables)?;

    if !response.issue_create.success {
        return Err(LinError::api("Failed to create issue"));
    }

    match response.issue_create.issue {
        Some(issue) => {
            output(&issue, format);
            Ok(())
        }
        None => Err(LinError::api(
            "Issue creation succeeded but no issue returned",
        )),
    }
}

/// Update an existing issue in Linear.
///
/// Updates an issue identified by ID or identifier (e.g., "ENG-123") and outputs
/// the updated issue.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `id_or_identifier` - The issue's UUID or human-readable identifier
/// * `options` - Fields to update (all optional)
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```ignore
/// use lin::api::GraphQLClient;
/// use lin::commands::issue::{update_issue, IssueUpdateOptions};
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// let options = IssueUpdateOptions {
///     title: Some("New title".to_string()),
///     priority: Some(1), // Urgent
///     ..Default::default()
/// };
/// update_issue(&client, "ENG-123", options, OutputFormat::Human)?;
/// ```
pub fn update_issue(
    client: &GraphQLClient,
    id_or_identifier: &str,
    options: IssueUpdateOptions,
    format: OutputFormat,
) -> Result<()> {
    // First, resolve the issue ID if given an identifier
    let issue_id = if is_uuid(id_or_identifier) {
        id_or_identifier.to_string()
    } else {
        // Parse the identifier and look up the issue to get its UUID
        let (team_key, number) = parse_identifier(id_or_identifier)?;

        let lookup_variables = serde_json::json!({
            "filter": {
                "team": { "key": { "eq": team_key } },
                "number": { "eq": number }
            }
        });

        let lookup_response: IssuesResponse =
            client.query(queries::ISSUE_BY_IDENTIFIER_QUERY, lookup_variables)?;

        if lookup_response.issues.nodes.is_empty() {
            return Err(LinError::api(format!(
                "Issue '{}' not found",
                id_or_identifier
            )));
        }

        lookup_response.issues.nodes[0].id.clone()
    };

    // Build the input object for the mutation
    let mut input = serde_json::Map::new();

    if let Some(title) = options.title {
        input.insert("title".to_string(), serde_json::json!(title));
    }

    if let Some(description) = options.description {
        input.insert("description".to_string(), serde_json::json!(description));
    }

    if let Some(assignee_id) = options.assignee_id {
        input.insert("assigneeId".to_string(), serde_json::json!(assignee_id));
    }

    if let Some(state_id) = options.state_id {
        input.insert("stateId".to_string(), serde_json::json!(state_id));
    }

    if let Some(priority) = options.priority {
        input.insert("priority".to_string(), serde_json::json!(priority));
    }

    let variables = serde_json::json!({
        "id": issue_id,
        "input": input
    });

    let response: IssueUpdateResponse = client.query(queries::ISSUE_UPDATE_MUTATION, variables)?;

    if !response.issue_update.success {
        return Err(LinError::api("Failed to update issue"));
    }

    match response.issue_update.issue {
        Some(issue) => {
            output(&issue, format);
            Ok(())
        }
        None => Err(LinError::api(
            "Issue update succeeded but no issue returned",
        )),
    }
}

/// Simple message response for delete/archive operations.
#[derive(Debug, Serialize)]
struct MessageResponse {
    message: String,
}

impl HumanDisplay for MessageResponse {
    fn human_fmt(&self) -> String {
        self.message.clone()
    }
}

/// Resolve an issue identifier to its UUID.
///
/// If the input is already a UUID, returns it as-is.
/// Otherwise, parses the identifier and looks up the issue.
fn resolve_issue_id(client: &GraphQLClient, id_or_identifier: &str) -> Result<String> {
    if is_uuid(id_or_identifier) {
        Ok(id_or_identifier.to_string())
    } else {
        let (team_key, number) = parse_identifier(id_or_identifier)?;

        let lookup_variables = serde_json::json!({
            "filter": {
                "team": { "key": { "eq": team_key } },
                "number": { "eq": number }
            }
        });

        let lookup_response: IssuesResponse =
            client.query(queries::ISSUE_BY_IDENTIFIER_QUERY, lookup_variables)?;

        if lookup_response.issues.nodes.is_empty() {
            return Err(LinError::api(format!(
                "Issue '{}' not found",
                id_or_identifier
            )));
        }

        Ok(lookup_response.issues.nodes[0].id.clone())
    }
}

/// Delete an issue in Linear.
///
/// Deletes an issue identified by ID or identifier (e.g., "ENG-123").
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `id_or_identifier` - The issue's UUID or human-readable identifier
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```ignore
/// use lin::api::GraphQLClient;
/// use lin::commands::issue::delete_issue;
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// delete_issue(&client, "ENG-123", OutputFormat::Human)?;
/// ```
pub fn delete_issue(
    client: &GraphQLClient,
    id_or_identifier: &str,
    format: OutputFormat,
) -> Result<()> {
    let issue_id = resolve_issue_id(client, id_or_identifier)?;

    let variables = serde_json::json!({ "id": issue_id });
    let response: IssueDeleteResponse = client.query(queries::ISSUE_DELETE_MUTATION, variables)?;

    if !response.issue_delete.success {
        return Err(LinError::api("Failed to delete issue"));
    }

    let message = MessageResponse {
        message: format!("Issue '{}' deleted successfully", id_or_identifier),
    };
    output(&message, format);
    Ok(())
}

/// Archive an issue in Linear.
///
/// Archives an issue identified by ID or identifier (e.g., "ENG-123").
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `id_or_identifier` - The issue's UUID or human-readable identifier
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```ignore
/// use lin::api::GraphQLClient;
/// use lin::commands::issue::archive_issue;
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// archive_issue(&client, "ENG-123", OutputFormat::Human)?;
/// ```
pub fn archive_issue(
    client: &GraphQLClient,
    id_or_identifier: &str,
    format: OutputFormat,
) -> Result<()> {
    let issue_id = resolve_issue_id(client, id_or_identifier)?;

    let variables = serde_json::json!({ "id": issue_id });
    let response: IssueArchiveResponse =
        client.query(queries::ISSUE_ARCHIVE_MUTATION, variables)?;

    if !response.issue_archive.success {
        return Err(LinError::api("Failed to archive issue"));
    }

    let message = MessageResponse {
        message: format!("Issue '{}' archived successfully", id_or_identifier),
    };
    output(&message, format);
    Ok(())
}

/// Unarchive an issue in Linear.
///
/// Unarchives an issue identified by ID or identifier (e.g., "ENG-123").
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `id_or_identifier` - The issue's UUID or human-readable identifier
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```ignore
/// use lin::api::GraphQLClient;
/// use lin::commands::issue::unarchive_issue;
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// unarchive_issue(&client, "ENG-123", OutputFormat::Human)?;
/// ```
pub fn unarchive_issue(
    client: &GraphQLClient,
    id_or_identifier: &str,
    format: OutputFormat,
) -> Result<()> {
    let issue_id = resolve_issue_id(client, id_or_identifier)?;

    let variables = serde_json::json!({ "id": issue_id });
    let response: IssueUnarchiveResponse =
        client.query(queries::ISSUE_UNARCHIVE_MUTATION, variables)?;

    if !response.issue_unarchive.success {
        return Err(LinError::api("Failed to unarchive issue"));
    }

    let message = MessageResponse {
        message: format!("Issue '{}' unarchived successfully", id_or_identifier),
    };
    output(&message, format);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::GraphQLClient;
    use crate::output::OutputFormat;

    // =============================================================================
    // is_uuid tests
    // =============================================================================

    #[test]
    fn test_is_uuid_standard_format() {
        assert!(is_uuid("550e8400-e29b-41d4-a716-446655440000"));
        assert!(is_uuid("123e4567-e89b-12d3-a456-426614174000"));
        assert!(is_uuid("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"));
    }

    #[test]
    fn test_is_uuid_without_hyphens() {
        assert!(is_uuid("550e8400e29b41d4a716446655440000"));
        assert!(is_uuid("123e4567e89b12d3a456426614174000"));
        assert!(is_uuid("aaaaaaaabbbbccccddddeeeeeeeeeeee"));
    }

    #[test]
    fn test_is_uuid_lowercase_and_uppercase() {
        assert!(is_uuid("550E8400-E29B-41D4-A716-446655440000"));
        assert!(is_uuid("550e8400-E29B-41D4-a716-446655440000"));
    }

    #[test]
    fn test_is_uuid_not_uuid_identifier() {
        assert!(!is_uuid("ENG-123"));
        assert!(!is_uuid("ABC-1"));
        assert!(!is_uuid("TEAM-999"));
    }

    #[test]
    fn test_is_uuid_not_uuid_other() {
        assert!(!is_uuid("ABC"));
        assert!(!is_uuid("123"));
        assert!(!is_uuid(""));
        assert!(!is_uuid("not-a-uuid-string-at-all"));
        assert!(!is_uuid("550e8400-e29b-41d4-a716")); // Too short
        assert!(!is_uuid("550e8400-e29b-41d4-a716-446655440000-extra")); // Too long
        assert!(!is_uuid("550e8400-e29b-41d4-a716-44665544000g")); // Invalid char 'g'
    }

    #[test]
    fn test_is_uuid_wrong_hyphen_positions() {
        assert!(!is_uuid("5-50e8400-e29b-41d4-a716-46655440000"));
        assert!(!is_uuid("550e8400e29b-41d4-a716-446655440000"));
    }

    // =============================================================================
    // parse_identifier tests
    // =============================================================================

    #[test]
    fn test_parse_identifier_simple() {
        let (team, num) = parse_identifier("ENG-123").unwrap();
        assert_eq!(team, "ENG");
        assert_eq!(num, 123);
    }

    #[test]
    fn test_parse_identifier_single_letter_team() {
        let (team, num) = parse_identifier("A-1").unwrap();
        assert_eq!(team, "A");
        assert_eq!(num, 1);
    }

    #[test]
    fn test_parse_identifier_long_team() {
        let (team, num) = parse_identifier("ENGINEERING-999").unwrap();
        assert_eq!(team, "ENGINEERING");
        assert_eq!(num, 999);
    }

    #[test]
    fn test_parse_identifier_large_number() {
        let (team, num) = parse_identifier("ENG-123456").unwrap();
        assert_eq!(team, "ENG");
        assert_eq!(num, 123456);
    }

    #[test]
    fn test_parse_identifier_no_hyphen() {
        let result = parse_identifier("ENG123");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid identifier format"));
    }

    #[test]
    fn test_parse_identifier_lowercase_team() {
        let result = parse_identifier("eng-123");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid team key"));
    }

    #[test]
    fn test_parse_identifier_mixed_case_team() {
        let result = parse_identifier("Eng-123");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid team key"));
    }

    #[test]
    fn test_parse_identifier_invalid_number() {
        let result = parse_identifier("ENG-abc");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid issue number"));
    }

    #[test]
    fn test_parse_identifier_negative_number() {
        let result = parse_identifier("ENG--5");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_identifier_zero() {
        let result = parse_identifier("ENG-0");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("must be positive"));
    }

    #[test]
    fn test_parse_identifier_empty_string() {
        let result = parse_identifier("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_identifier_just_hyphen() {
        let result = parse_identifier("-");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_identifier_only_number() {
        let result = parse_identifier("-123");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid team key"));
    }

    // =============================================================================
    // list_issues tests
    // =============================================================================

    #[test]
    fn test_list_issues_success() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "issues": {
                            "nodes": [
                                {
                                    "id": "issue-1",
                                    "identifier": "ENG-1",
                                    "title": "First issue",
                                    "description": "Description 1",
                                    "priority": 1,
                                    "state": {
                                        "id": "state-1",
                                        "name": "In Progress",
                                        "color": "#0066ff",
                                        "type": "started"
                                    },
                                    "team": {
                                        "id": "team-1",
                                        "key": "ENG",
                                        "name": "Engineering",
                                        "description": null
                                    },
                                    "assignee": null,
                                    "createdAt": "2024-01-01T00:00:00.000Z",
                                    "updatedAt": "2024-01-01T00:00:00.000Z"
                                },
                                {
                                    "id": "issue-2",
                                    "identifier": "ENG-2",
                                    "title": "Second issue",
                                    "description": null,
                                    "priority": 2,
                                    "state": null,
                                    "team": null,
                                    "assignee": null,
                                    "createdAt": "2024-01-02T00:00:00.000Z",
                                    "updatedAt": "2024-01-02T00:00:00.000Z"
                                }
                            ]
                        }
                    }
                }"##,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions::default();

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_with_team_filter() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issues": {
                            "nodes": []
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            team: Some("ENG".to_string()),
            ..Default::default()
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_with_assignee_me() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issues": {
                            "nodes": []
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            assignee: Some("me".to_string()),
            ..Default::default()
        };

        let result = list_issues(&client, Some("user-123"), options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_with_assignee_me_no_viewer() {
        let server = mockito::Server::new();
        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            assignee: Some("me".to_string()),
            ..Default::default()
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Cannot use 'me'"));
    }

    #[test]
    fn test_list_issues_with_state_filter() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issues": {
                            "nodes": []
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            state: Some("In Progress".to_string()),
            ..Default::default()
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_with_project_filter() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issues": {
                            "nodes": []
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            project: Some("project-123".to_string()),
            ..Default::default()
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_with_limit() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issues": {
                            "nodes": []
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            limit: Some(10),
            ..Default::default()
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_with_all_filters() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issues": {
                            "nodes": []
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions {
            team: Some("ENG".to_string()),
            assignee: Some("user-456".to_string()),
            state: Some("Done".to_string()),
            project: Some("project-789".to_string()),
            limit: Some(25),
        };

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_issues_api_error() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": null,
                    "errors": [
                        {
                            "message": "Not authenticated"
                        }
                    ]
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("invalid-token", &server.url());
        let options = IssueListOptions::default();

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Not authenticated"));
        mock.assert();
    }

    #[test]
    fn test_list_issues_empty() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issues": {
                            "nodes": []
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueListOptions::default();

        let result = list_issues(&client, None, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    // =============================================================================
    // get_issue tests
    // =============================================================================

    #[test]
    fn test_get_issue_by_uuid() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "issue": {
                            "id": "550e8400-e29b-41d4-a716-446655440000",
                            "identifier": "ENG-123",
                            "title": "Test issue",
                            "description": "A test issue",
                            "priority": 2,
                            "state": {
                                "id": "state-1",
                                "name": "In Progress",
                                "color": "#0066ff",
                                "type": "started"
                            },
                            "team": {
                                "id": "team-1",
                                "key": "ENG",
                                "name": "Engineering",
                                "description": null
                            },
                            "assignee": {
                                "id": "user-1",
                                "name": "John Doe",
                                "email": "john@example.com",
                                "displayName": "JD",
                                "active": true
                            },
                            "createdAt": "2024-01-01T00:00:00.000Z",
                            "updatedAt": "2024-01-02T00:00:00.000Z"
                        }
                    }
                }"##,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = get_issue(
            &client,
            "550e8400-e29b-41d4-a716-446655440000",
            OutputFormat::Human,
        );

        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_get_issue_by_identifier() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "issues": {
                            "nodes": [
                                {
                                    "id": "issue-abc",
                                    "identifier": "ENG-123",
                                    "title": "Test issue",
                                    "description": "A test issue",
                                    "priority": 2,
                                    "state": {
                                        "id": "state-1",
                                        "name": "In Progress",
                                        "color": "#0066ff",
                                        "type": "started"
                                    },
                                    "team": {
                                        "id": "team-1",
                                        "key": "ENG",
                                        "name": "Engineering",
                                        "description": null
                                    },
                                    "assignee": null,
                                    "createdAt": "2024-01-01T00:00:00.000Z",
                                    "updatedAt": "2024-01-02T00:00:00.000Z"
                                }
                            ]
                        }
                    }
                }"##,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = get_issue(&client, "ENG-123", OutputFormat::Human);

        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_get_issue_by_identifier_not_found() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issues": {
                            "nodes": []
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = get_issue(&client, "ENG-99999", OutputFormat::Human);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not found"));
        mock.assert();
    }

    #[test]
    fn test_get_issue_uuid_not_found() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": null,
                    "errors": [
                        {
                            "message": "Entity not found"
                        }
                    ]
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = get_issue(
            &client,
            "550e8400-e29b-41d4-a716-446655440000",
            OutputFormat::Human,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Entity not found"));
        mock.assert();
    }

    #[test]
    fn test_get_issue_invalid_identifier() {
        let server = mockito::Server::new();
        let client = GraphQLClient::with_url("test-token", &server.url());

        let result = get_issue(&client, "invalid-identifier", OutputFormat::Human);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid team key"));
    }

    #[test]
    fn test_get_issue_api_error() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": null,
                    "errors": [
                        {
                            "message": "Not authenticated"
                        }
                    ]
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("invalid-token", &server.url());
        let result = get_issue(
            &client,
            "550e8400-e29b-41d4-a716-446655440000",
            OutputFormat::Human,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Not authenticated"));
        mock.assert();
    }

    #[test]
    fn test_get_issue_http_error() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(500)
            .with_body("Internal Server Error")
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = get_issue(&client, "ENG-123", OutputFormat::Human);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("HTTP 500"));
        mock.assert();
    }

    #[test]
    fn test_get_issue_with_uuid_without_hyphens() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "issue": {
                            "id": "550e8400e29b41d4a716446655440000",
                            "identifier": "ENG-456",
                            "title": "Another issue",
                            "description": null,
                            "priority": 0,
                            "state": null,
                            "team": null,
                            "assignee": null,
                            "createdAt": "2024-01-01T00:00:00.000Z",
                            "updatedAt": "2024-01-01T00:00:00.000Z"
                        }
                    }
                }"##,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = get_issue(
            &client,
            "550e8400e29b41d4a716446655440000",
            OutputFormat::Human,
        );

        assert!(result.is_ok());
        mock.assert();
    }

    // =============================================================================
    // create_issue tests
    // =============================================================================

    #[test]
    fn test_create_issue_minimal_options() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "issueCreate": {
                            "success": true,
                            "issue": {
                                "id": "issue-new",
                                "identifier": "ENG-999",
                                "title": "New Issue",
                                "description": null,
                                "priority": 0,
                                "state": null,
                                "team": {
                                    "id": "team-1",
                                    "key": "ENG",
                                    "name": "Engineering",
                                    "description": null
                                },
                                "assignee": null,
                                "createdAt": "2024-01-01T00:00:00.000Z",
                                "updatedAt": "2024-01-01T00:00:00.000Z"
                            }
                        }
                    }
                }"##,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueCreateOptions {
            title: "New Issue".to_string(),
            team_id: "team-1".to_string(),
            description: None,
            assignee_id: None,
            state_id: None,
            priority: None,
        };

        let result = create_issue(&client, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_create_issue_all_options() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "issueCreate": {
                            "success": true,
                            "issue": {
                                "id": "issue-full",
                                "identifier": "ENG-1000",
                                "title": "Full Issue",
                                "description": "Detailed description",
                                "priority": 2,
                                "state": {
                                    "id": "state-1",
                                    "name": "In Progress",
                                    "color": "#0066ff",
                                    "type": "started"
                                },
                                "team": {
                                    "id": "team-1",
                                    "key": "ENG",
                                    "name": "Engineering",
                                    "description": null
                                },
                                "assignee": {
                                    "id": "user-1",
                                    "name": "John Doe",
                                    "email": "john@example.com",
                                    "displayName": "JD",
                                    "active": true
                                },
                                "createdAt": "2024-01-01T00:00:00.000Z",
                                "updatedAt": "2024-01-01T00:00:00.000Z"
                            }
                        }
                    }
                }"##,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueCreateOptions {
            title: "Full Issue".to_string(),
            team_id: "team-1".to_string(),
            description: Some("Detailed description".to_string()),
            assignee_id: Some("user-1".to_string()),
            state_id: Some("state-1".to_string()),
            priority: Some(2),
        };

        let result = create_issue(&client, options, OutputFormat::Human);
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_create_issue_failure() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issueCreate": {
                            "success": false,
                            "issue": null
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueCreateOptions {
            title: "Bad Issue".to_string(),
            team_id: "invalid-team".to_string(),
            description: None,
            assignee_id: None,
            state_id: None,
            priority: None,
        };

        let result = create_issue(&client, options, OutputFormat::Human);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Failed to create issue"));
        mock.assert();
    }

    #[test]
    fn test_create_issue_api_error() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": null,
                    "errors": [
                        {
                            "message": "Not authenticated"
                        }
                    ]
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("invalid-token", &server.url());
        let options = IssueCreateOptions {
            title: "Test".to_string(),
            team_id: "team-1".to_string(),
            description: None,
            assignee_id: None,
            state_id: None,
            priority: None,
        };

        let result = create_issue(&client, options, OutputFormat::Human);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Not authenticated"));
        mock.assert();
    }

    // =============================================================================
    // update_issue tests
    // =============================================================================

    #[test]
    fn test_update_issue_by_uuid() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "issueUpdate": {
                            "success": true,
                            "issue": {
                                "id": "550e8400-e29b-41d4-a716-446655440000",
                                "identifier": "ENG-123",
                                "title": "Updated Title",
                                "description": "Updated description",
                                "priority": 1,
                                "state": {
                                    "id": "state-2",
                                    "name": "Done",
                                    "color": "#00ff00",
                                    "type": "completed"
                                },
                                "team": {
                                    "id": "team-1",
                                    "key": "ENG",
                                    "name": "Engineering",
                                    "description": null
                                },
                                "assignee": null,
                                "createdAt": "2024-01-01T00:00:00.000Z",
                                "updatedAt": "2024-01-02T00:00:00.000Z"
                            }
                        }
                    }
                }"##,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueUpdateOptions {
            title: Some("Updated Title".to_string()),
            description: Some("Updated description".to_string()),
            assignee_id: None,
            state_id: Some("state-2".to_string()),
            priority: Some(1),
        };

        let result = update_issue(
            &client,
            "550e8400-e29b-41d4-a716-446655440000",
            options,
            OutputFormat::Human,
        );
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_update_issue_by_identifier() {
        let mut server = mockito::Server::new();

        // First mock: lookup by identifier
        let lookup_mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "issues": {
                            "nodes": [
                                {
                                    "id": "issue-uuid-123",
                                    "identifier": "ENG-123",
                                    "title": "Original Title",
                                    "description": null,
                                    "priority": 0,
                                    "state": null,
                                    "team": null,
                                    "assignee": null,
                                    "createdAt": "2024-01-01T00:00:00.000Z",
                                    "updatedAt": "2024-01-01T00:00:00.000Z"
                                }
                            ]
                        }
                    }
                }"##,
            )
            .create();

        // Second mock: update mutation
        let update_mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "issueUpdate": {
                            "success": true,
                            "issue": {
                                "id": "issue-uuid-123",
                                "identifier": "ENG-123",
                                "title": "New Title",
                                "description": null,
                                "priority": 0,
                                "state": null,
                                "team": null,
                                "assignee": null,
                                "createdAt": "2024-01-01T00:00:00.000Z",
                                "updatedAt": "2024-01-02T00:00:00.000Z"
                            }
                        }
                    }
                }"##,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueUpdateOptions {
            title: Some("New Title".to_string()),
            ..Default::default()
        };

        let result = update_issue(&client, "ENG-123", options, OutputFormat::Human);
        assert!(result.is_ok());
        lookup_mock.assert();
        update_mock.assert();
    }

    #[test]
    fn test_update_issue_not_found() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issues": {
                            "nodes": []
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueUpdateOptions {
            title: Some("New Title".to_string()),
            ..Default::default()
        };

        let result = update_issue(&client, "ENG-99999", options, OutputFormat::Human);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not found"));
        mock.assert();
    }

    #[test]
    fn test_update_issue_invalid_identifier() {
        let server = mockito::Server::new();
        let client = GraphQLClient::with_url("test-token", &server.url());

        let options = IssueUpdateOptions {
            title: Some("New Title".to_string()),
            ..Default::default()
        };

        let result = update_issue(&client, "invalid-identifier", options, OutputFormat::Human);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid team key"));
    }

    #[test]
    fn test_update_issue_failure() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issueUpdate": {
                            "success": false,
                            "issue": null
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = IssueUpdateOptions {
            title: Some("New Title".to_string()),
            ..Default::default()
        };

        let result = update_issue(
            &client,
            "550e8400-e29b-41d4-a716-446655440000",
            options,
            OutputFormat::Human,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Failed to update issue"));
        mock.assert();
    }

    #[test]
    fn test_update_issue_partial_update() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "issueUpdate": {
                            "success": true,
                            "issue": {
                                "id": "550e8400-e29b-41d4-a716-446655440000",
                                "identifier": "ENG-123",
                                "title": "Original Title",
                                "description": null,
                                "priority": 3,
                                "state": null,
                                "team": null,
                                "assignee": null,
                                "createdAt": "2024-01-01T00:00:00.000Z",
                                "updatedAt": "2024-01-02T00:00:00.000Z"
                            }
                        }
                    }
                }"##,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        // Only update priority
        let options = IssueUpdateOptions {
            title: None,
            description: None,
            assignee_id: None,
            state_id: None,
            priority: Some(3),
        };

        let result = update_issue(
            &client,
            "550e8400-e29b-41d4-a716-446655440000",
            options,
            OutputFormat::Human,
        );
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_update_issue_api_error() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": null,
                    "errors": [
                        {
                            "message": "Not authenticated"
                        }
                    ]
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("invalid-token", &server.url());
        let options = IssueUpdateOptions {
            title: Some("New Title".to_string()),
            ..Default::default()
        };

        let result = update_issue(
            &client,
            "550e8400-e29b-41d4-a716-446655440000",
            options,
            OutputFormat::Human,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Not authenticated"));
        mock.assert();
    }

    // =============================================================================
    // delete_issue tests
    // =============================================================================

    #[test]
    fn test_delete_issue_by_uuid() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issueDelete": {
                            "success": true
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = delete_issue(
            &client,
            "550e8400-e29b-41d4-a716-446655440000",
            OutputFormat::Human,
        );

        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_delete_issue_by_identifier() {
        let mut server = mockito::Server::new();

        // First mock: lookup by identifier
        let lookup_mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "issues": {
                            "nodes": [
                                {
                                    "id": "issue-uuid-123",
                                    "identifier": "ENG-123",
                                    "title": "Issue to delete",
                                    "description": null,
                                    "priority": 0,
                                    "state": null,
                                    "team": null,
                                    "assignee": null,
                                    "createdAt": "2024-01-01T00:00:00.000Z",
                                    "updatedAt": "2024-01-01T00:00:00.000Z"
                                }
                            ]
                        }
                    }
                }"##,
            )
            .create();

        // Second mock: delete mutation
        let delete_mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issueDelete": {
                            "success": true
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = delete_issue(&client, "ENG-123", OutputFormat::Human);

        assert!(result.is_ok());
        lookup_mock.assert();
        delete_mock.assert();
    }

    #[test]
    fn test_delete_issue_not_found() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issues": {
                            "nodes": []
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = delete_issue(&client, "ENG-99999", OutputFormat::Human);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not found"));
        mock.assert();
    }

    #[test]
    fn test_delete_issue_failure() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issueDelete": {
                            "success": false
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = delete_issue(
            &client,
            "550e8400-e29b-41d4-a716-446655440000",
            OutputFormat::Human,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Failed to delete issue"));
        mock.assert();
    }

    #[test]
    fn test_delete_issue_invalid_identifier() {
        let server = mockito::Server::new();
        let client = GraphQLClient::with_url("test-token", &server.url());

        let result = delete_issue(&client, "invalid-identifier", OutputFormat::Human);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid team key"));
    }

    // =============================================================================
    // archive_issue tests
    // =============================================================================

    #[test]
    fn test_archive_issue_by_uuid() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issueArchive": {
                            "success": true
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = archive_issue(
            &client,
            "550e8400-e29b-41d4-a716-446655440000",
            OutputFormat::Human,
        );

        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_archive_issue_by_identifier() {
        let mut server = mockito::Server::new();

        // First mock: lookup by identifier
        let lookup_mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "issues": {
                            "nodes": [
                                {
                                    "id": "issue-uuid-456",
                                    "identifier": "ENG-456",
                                    "title": "Issue to archive",
                                    "description": null,
                                    "priority": 0,
                                    "state": null,
                                    "team": null,
                                    "assignee": null,
                                    "createdAt": "2024-01-01T00:00:00.000Z",
                                    "updatedAt": "2024-01-01T00:00:00.000Z"
                                }
                            ]
                        }
                    }
                }"##,
            )
            .create();

        // Second mock: archive mutation
        let archive_mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issueArchive": {
                            "success": true
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = archive_issue(&client, "ENG-456", OutputFormat::Human);

        assert!(result.is_ok());
        lookup_mock.assert();
        archive_mock.assert();
    }

    #[test]
    fn test_archive_issue_failure() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issueArchive": {
                            "success": false
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = archive_issue(
            &client,
            "550e8400-e29b-41d4-a716-446655440000",
            OutputFormat::Human,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Failed to archive issue"));
        mock.assert();
    }

    #[test]
    fn test_archive_issue_not_found() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issues": {
                            "nodes": []
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = archive_issue(&client, "ENG-99999", OutputFormat::Human);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not found"));
        mock.assert();
    }

    // =============================================================================
    // unarchive_issue tests
    // =============================================================================

    #[test]
    fn test_unarchive_issue_by_uuid() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issueUnarchive": {
                            "success": true
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = unarchive_issue(
            &client,
            "550e8400-e29b-41d4-a716-446655440000",
            OutputFormat::Human,
        );

        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_unarchive_issue_by_identifier() {
        let mut server = mockito::Server::new();

        // First mock: lookup by identifier
        let lookup_mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "issues": {
                            "nodes": [
                                {
                                    "id": "issue-uuid-789",
                                    "identifier": "ENG-789",
                                    "title": "Issue to unarchive",
                                    "description": null,
                                    "priority": 0,
                                    "state": null,
                                    "team": null,
                                    "assignee": null,
                                    "createdAt": "2024-01-01T00:00:00.000Z",
                                    "updatedAt": "2024-01-01T00:00:00.000Z"
                                }
                            ]
                        }
                    }
                }"##,
            )
            .create();

        // Second mock: unarchive mutation
        let unarchive_mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issueUnarchive": {
                            "success": true
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = unarchive_issue(&client, "ENG-789", OutputFormat::Human);

        assert!(result.is_ok());
        lookup_mock.assert();
        unarchive_mock.assert();
    }

    #[test]
    fn test_unarchive_issue_failure() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issueUnarchive": {
                            "success": false
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = unarchive_issue(
            &client,
            "550e8400-e29b-41d4-a716-446655440000",
            OutputFormat::Human,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Failed to unarchive issue"));
        mock.assert();
    }

    #[test]
    fn test_unarchive_issue_not_found() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issues": {
                            "nodes": []
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = unarchive_issue(&client, "ENG-99999", OutputFormat::Human);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not found"));
        mock.assert();
    }
}
