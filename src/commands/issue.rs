//! Issue management commands.
//!
//! Commands for listing and viewing issue information from Linear.

use crate::api::{queries, GraphQLClient};
use crate::error::LinError;
use crate::models::{IssueResponse, IssuesResponse};
use crate::output::output_success;
use crate::Result;

/// Options for listing issues.
#[derive(Debug, Clone, Default)]
pub struct IssueListOptions {
    /// Filter by team key (e.g., "ENG").
    pub team: Option<String>,
    /// Filter by assignee ID or "me".
    pub assignee: Option<String>,
    /// Filter by state name.
    pub state: Option<String>,
    /// Maximum number of issues to return (default 50).
    pub limit: Option<i32>,
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
        || !team_key
            .chars()
            .all(|c| c.is_ascii_uppercase() || c == '-')
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
/// Fetches issues from the Linear API and outputs them as a JSON array.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `viewer_id` - The current user's ID (used if assignee is "me")
/// * `options` - Filter options for the query
///
/// # Example
///
/// ```ignore
/// use lin::api::GraphQLClient;
/// use lin::commands::issue::{list_issues, IssueListOptions};
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// let options = IssueListOptions {
///     team: Some("ENG".to_string()),
///     assignee: None,
///     state: None,
///     limit: Some(10),
/// };
/// list_issues(&client, None, options)?;
/// ```
pub fn list_issues(
    client: &GraphQLClient,
    viewer_id: Option<&str>,
    options: IssueListOptions,
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

    output_success(&response.issues.nodes);
    Ok(())
}

/// Get details of a specific issue by ID or identifier.
///
/// Fetches a single issue from the Linear API and outputs it as JSON.
/// Supports both UUID format (e.g., "550e8400-e29b-41d4-a716-446655440000")
/// and identifier format (e.g., "ENG-123").
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `id_or_identifier` - The issue's UUID or human-readable identifier
///
/// # Example
///
/// ```ignore
/// use lin::api::GraphQLClient;
/// use lin::commands::issue::get_issue;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
///
/// // By UUID
/// get_issue(&client, "550e8400-e29b-41d4-a716-446655440000")?;
///
/// // By identifier
/// get_issue(&client, "ENG-123")?;
/// ```
pub fn get_issue(client: &GraphQLClient, id_or_identifier: &str) -> Result<()> {
    if is_uuid(id_or_identifier) {
        // Query by UUID
        let variables = serde_json::json!({
            "id": id_or_identifier
        });
        let response: IssueResponse = client.query(queries::ISSUE_QUERY, variables)?;
        output_success(&response.issue);
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

        output_success(&response.issues.nodes[0]);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::GraphQLClient;

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

        let result = list_issues(&client, None, options);
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

        let result = list_issues(&client, None, options);
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

        let result = list_issues(&client, Some("user-123"), options);
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

        let result = list_issues(&client, None, options);
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

        let result = list_issues(&client, None, options);
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

        let result = list_issues(&client, None, options);
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
            limit: Some(25),
        };

        let result = list_issues(&client, None, options);
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

        let result = list_issues(&client, None, options);
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

        let result = list_issues(&client, None, options);
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
        let result = get_issue(&client, "550e8400-e29b-41d4-a716-446655440000");

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
        let result = get_issue(&client, "ENG-123");

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
        let result = get_issue(&client, "ENG-99999");

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
        let result = get_issue(&client, "550e8400-e29b-41d4-a716-446655440000");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Entity not found"));
        mock.assert();
    }

    #[test]
    fn test_get_issue_invalid_identifier() {
        let server = mockito::Server::new();
        let client = GraphQLClient::with_url("test-token", &server.url());

        let result = get_issue(&client, "invalid-identifier");
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
        let result = get_issue(&client, "550e8400-e29b-41d4-a716-446655440000");

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
        let result = get_issue(&client, "ENG-123");

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
        let result = get_issue(&client, "550e8400e29b41d4a716446655440000");

        assert!(result.is_ok());
        mock.assert();
    }
}
