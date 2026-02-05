//! Workflow state management commands.
//!
//! Commands for listing workflow states from Linear teams.

use crate::Result;
use crate::api::GraphQLClient;
use crate::api::queries::workflow::WORKFLOW_STATES_QUERY;
use crate::models::WorkflowStatesResponse;
use crate::output::{OutputFormat, output};

/// List all workflow states for a team.
///
/// Fetches workflow states from the Linear API and outputs them.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `team_id` - The team's unique identifier (UUID or key)
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use lin::api::GraphQLClient;
/// use lin::commands::workflow::list_workflow_states;
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// list_workflow_states(&client, "team-123", OutputFormat::Human)?;
/// # Ok(())
/// # }
/// ```
pub fn list_workflow_states(
    client: &GraphQLClient,
    team_id: &str,
    format: OutputFormat,
) -> Result<()> {
    let variables = serde_json::json!({
        "id": team_id
    });
    let response: WorkflowStatesResponse = client.query(WORKFLOW_STATES_QUERY, variables)?;
    output(&response.team.states.nodes, format);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::GraphQLClient;

    #[test]
    fn test_list_workflow_states_success() {
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
                        "team": {
                            "id": "team-123",
                            "states": {
                                "nodes": [
                                    {
                                        "id": "state-1",
                                        "name": "Backlog",
                                        "color": "#bec2c8",
                                        "type": "backlog"
                                    },
                                    {
                                        "id": "state-2",
                                        "name": "Todo",
                                        "color": "#e2e2e2",
                                        "type": "unstarted"
                                    },
                                    {
                                        "id": "state-3",
                                        "name": "In Progress",
                                        "color": "#f2c94c",
                                        "type": "started"
                                    },
                                    {
                                        "id": "state-4",
                                        "name": "Done",
                                        "color": "#5e6ad2",
                                        "type": "completed"
                                    },
                                    {
                                        "id": "state-5",
                                        "name": "Canceled",
                                        "color": "#95a2b3",
                                        "type": "canceled"
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

        // Make request
        let result = list_workflow_states(&client, "team-123", OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_workflow_states_empty() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with empty states
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "team": {
                            "id": "team-123",
                            "states": {
                                "nodes": []
                            }
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = list_workflow_states(&client, "team-123", OutputFormat::Human);

        // Verify success (empty list is still valid)
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_workflow_states_team_not_found() {
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
        let result = list_workflow_states(&client, "nonexistent-team", OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Entity not found"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_workflow_states_api_error() {
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
        let result = list_workflow_states(&client, "team-123", OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Not authenticated"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_workflow_states_http_error() {
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

        // Make request
        let result = list_workflow_states(&client, "team-123", OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("HTTP 401"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_workflow_states_json_output() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "team": {
                            "id": "team-123",
                            "states": {
                                "nodes": [
                                    {
                                        "id": "state-1",
                                        "name": "Todo",
                                        "color": "#e2e2e2",
                                        "type": "unstarted"
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

        // Make request with JSON format
        let result = list_workflow_states(&client, "team-123", OutputFormat::Json);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }
}
