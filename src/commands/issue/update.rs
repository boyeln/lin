//! Update operations for issues.

use crate::Result;
use crate::api::GraphQLClient;
use crate::api::queries::issue::{ISSUE_BY_IDENTIFIER_QUERY, ISSUE_UPDATE_MUTATION};
use crate::error::LinError;
use crate::models::{IssueUpdateResponse, IssuesResponse};
use crate::output::{OutputFormat, output};

use super::{IssueUpdateOptions, is_uuid, parse_identifier};

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
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use lin::api::GraphQLClient;
/// use lin::commands::issue::update::update_issue;
/// use lin::commands::issue::IssueUpdateOptions;
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// let options = IssueUpdateOptions {
///     title: Some("New title".to_string()),
///     priority: Some(1), // Urgent
///     project_id: None,
///     ..Default::default()
/// };
/// update_issue(&client, "ENG-123", options, OutputFormat::Human)?;
/// # Ok(())
/// # }
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
            client.query(ISSUE_BY_IDENTIFIER_QUERY, lookup_variables)?;

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

    if let Some(estimate) = options.estimate {
        input.insert("estimate".to_string(), serde_json::json!(estimate));
    }

    if let Some(label_ids) = options.label_ids {
        input.insert("labelIds".to_string(), serde_json::json!(label_ids));
    }

    if let Some(project_id) = options.project_id {
        input.insert("projectId".to_string(), serde_json::json!(project_id));
    }

    let variables = serde_json::json!({
        "id": issue_id,
        "input": input
    });

    let response: IssueUpdateResponse = client.query(ISSUE_UPDATE_MUTATION, variables)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::GraphQLClient;
    use crate::output::OutputFormat;

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
                                "estimate": null,
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
            estimate: None,
            label_ids: None,
            project_id: None,
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
                                    "estimate": null,
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
                                "estimate": null,
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
                                "estimate": null,
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
            estimate: None,
            label_ids: None,
            project_id: None,
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
}
