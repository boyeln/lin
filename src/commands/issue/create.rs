//! Create operations for issues.

use crate::api::{queries, GraphQLClient};
use crate::error::LinError;
use crate::models::IssueCreateResponse;
use crate::output::{output, OutputFormat};
use crate::Result;

use super::IssueCreateOptions;

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
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
///     label_ids: None,
/// };
/// create_issue(&client, options, OutputFormat::Human)?;
/// # Ok(())
/// # }
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

    if let Some(label_ids) = options.label_ids {
        input.insert("labelIds".to_string(), serde_json::json!(label_ids));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::GraphQLClient;
    use crate::output::OutputFormat;

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
            label_ids: None,
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
            label_ids: None,
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
            label_ids: None,
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
            label_ids: None,
        };

        let result = create_issue(&client, options, OutputFormat::Human);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Not authenticated"));
        mock.assert();
    }
}
