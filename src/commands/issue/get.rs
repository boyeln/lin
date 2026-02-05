//! Get operations for issues (get single issue, with or without comments).

use crate::api::queries::issue::{
    ISSUE_BY_IDENTIFIER_QUERY, ISSUE_BY_IDENTIFIER_WITH_COMMENTS_QUERY, ISSUE_QUERY,
    ISSUE_WITH_COMMENTS_QUERY,
};
use crate::api::GraphQLClient;
use crate::error::LinError;
use crate::models::{
    IssueResponse, IssueWithCommentsResponse, IssuesResponse, IssuesWithCommentsResponse,
};
use crate::output::{output, OutputFormat};
use crate::Result;

use super::{is_uuid, parse_identifier};

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
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use lin::api::GraphQLClient;
/// use lin::commands::issue::get::get_issue;
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
///
/// // By UUID
/// get_issue(&client, "550e8400-e29b-41d4-a716-446655440000", OutputFormat::Human)?;
///
/// // By identifier
/// get_issue(&client, "ENG-123", OutputFormat::Human)?;
/// # Ok(())
/// # }
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
        let response: IssueResponse = client.query(ISSUE_QUERY, variables)?;
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

        let response: IssuesResponse = client.query(ISSUE_BY_IDENTIFIER_QUERY, variables)?;

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
            client.query(ISSUE_WITH_COMMENTS_QUERY, variables)?;
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
            client.query(ISSUE_BY_IDENTIFIER_WITH_COMMENTS_QUERY, variables)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::GraphQLClient;
    use crate::output::OutputFormat;

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
}
