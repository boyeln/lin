//! Label management commands.
//!
//! Commands for listing and viewing label information from Linear.

use crate::api::queries::label::{LABELS_QUERY, LABEL_QUERY, TEAM_LABELS_QUERY};
use crate::api::GraphQLClient;
use crate::models::{LabelResponse, LabelsResponse, TeamLabelsResponse};
use crate::output::{output, OutputFormat};
use crate::Result;

/// Options for listing labels.
#[derive(Debug, Clone, Default)]
pub struct LabelListOptions {
    /// Filter by team ID (optional - if provided, only show team-specific labels).
    pub team_id: Option<String>,
}

/// List labels in the workspace or for a specific team.
///
/// Fetches labels from the Linear API and outputs them.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `options` - Filter options for the query
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use lin::api::GraphQLClient;
/// use lin::commands::label::{list_labels, LabelListOptions};
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// let options = LabelListOptions { team_id: None };
/// list_labels(&client, options, OutputFormat::Human)?;
/// # Ok(())
/// # }
/// ```
pub fn list_labels(
    client: &GraphQLClient,
    options: LabelListOptions,
    format: OutputFormat,
) -> Result<()> {
    if let Some(team_id) = options.team_id {
        // Query team-specific labels
        let variables = serde_json::json!({
            "teamId": team_id
        });
        let response: TeamLabelsResponse = client.query(TEAM_LABELS_QUERY, variables)?;
        output(&response.team.labels.nodes, format);
    } else {
        // Query workspace labels
        let response: LabelsResponse = client.query(LABELS_QUERY, serde_json::json!({}))?;
        output(&response.issue_labels.nodes, format);
    }
    Ok(())
}

/// Get details of a specific label by ID.
///
/// Fetches a single label from the Linear API and outputs it.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `id` - The label's unique identifier
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use lin::api::GraphQLClient;
/// use lin::commands::label::get_label;
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// get_label(&client, "label-123", OutputFormat::Human)?;
/// # Ok(())
/// # }
/// ```
pub fn get_label(client: &GraphQLClient, id: &str, format: OutputFormat) -> Result<()> {
    let variables = serde_json::json!({
        "id": id
    });
    let response: LabelResponse = client.query(LABEL_QUERY, variables)?;
    output(&response.issue_label, format);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::GraphQLClient;

    #[test]
    fn test_list_labels_success() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response
        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .match_header("content-type", "application/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "issueLabels": {
                            "nodes": [
                                {
                                    "id": "label-1",
                                    "name": "Bug",
                                    "description": "Bug reports",
                                    "color": "#ff0000",
                                    "isGroup": false,
                                    "createdAt": "2024-01-01T00:00:00.000Z",
                                    "updatedAt": "2024-01-01T00:00:00.000Z"
                                },
                                {
                                    "id": "label-2",
                                    "name": "Feature",
                                    "description": null,
                                    "color": "#00ff00",
                                    "isGroup": false,
                                    "createdAt": "2024-01-02T00:00:00.000Z",
                                    "updatedAt": "2024-01-02T00:00:00.000Z"
                                }
                            ]
                        }
                    }
                }"##,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = LabelListOptions::default();

        // Make request
        let result = list_labels(&client, options, OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_labels_empty() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with empty labels
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issueLabels": {
                            "nodes": []
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = LabelListOptions::default();

        // Make request
        let result = list_labels(&client, options, OutputFormat::Human);

        // Verify success (empty list is still valid)
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_labels_with_team_filter() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response
        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "team": {
                            "id": "team-123",
                            "labels": {
                                "nodes": [
                                    {
                                        "id": "label-1",
                                        "name": "Team Bug",
                                        "description": "Team-specific bug label",
                                        "color": "#ff0000",
                                        "isGroup": false,
                                        "createdAt": "2024-01-01T00:00:00.000Z",
                                        "updatedAt": "2024-01-01T00:00:00.000Z"
                                    }
                                ]
                            }
                        }
                    }
                }"##,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = LabelListOptions {
            team_id: Some("team-123".to_string()),
        };

        // Make request
        let result = list_labels(&client, options, OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_labels_api_error() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with GraphQL error
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

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("invalid-token", &server.url());
        let options = LabelListOptions::default();

        // Make request
        let result = list_labels(&client, options, OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Not authenticated"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_labels_http_error() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with HTTP error
        let mock = server
            .mock("POST", "/")
            .with_status(401)
            .with_body(r#"{"error": "Unauthorized"}"#)
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("invalid-token", &server.url());
        let options = LabelListOptions::default();

        // Make request
        let result = list_labels(&client, options, OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("HTTP 401"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_get_label_success() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response
        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .match_header("content-type", "application/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "issueLabel": {
                            "id": "label-123",
                            "name": "Bug",
                            "description": "Bug reports",
                            "color": "#ff0000",
                            "isGroup": false,
                            "createdAt": "2024-01-01T00:00:00.000Z",
                            "updatedAt": "2024-01-15T00:00:00.000Z"
                        }
                    }
                }"##,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = get_label(&client, "label-123", OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_get_label_with_null_description() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with null description
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "issueLabel": {
                            "id": "label-456",
                            "name": "Feature",
                            "description": null,
                            "color": "#00ff00",
                            "isGroup": true,
                            "createdAt": "2024-01-01T00:00:00.000Z",
                            "updatedAt": "2024-01-01T00:00:00.000Z"
                        }
                    }
                }"##,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = get_label(&client, "label-456", OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_get_label_not_found() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with GraphQL error for not found
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

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = get_label(&client, "nonexistent-label", OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Entity not found"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_get_label_api_error() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with authentication error
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

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("invalid-token", &server.url());

        // Make request
        let result = get_label(&client, "label-123", OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Not authenticated"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_get_label_http_error() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with HTTP 500
        let mock = server
            .mock("POST", "/")
            .with_status(500)
            .with_body("Internal Server Error")
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = get_label(&client, "label-123", OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("HTTP 500"));

        // Verify mock was called
        mock.assert();
    }
}
