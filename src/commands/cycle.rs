//! Cycle (sprint) management commands.
//!
//! Commands for listing and viewing cycle information from Linear.

use crate::api::{queries, GraphQLClient};
use crate::models::{CycleResponse, CyclesResponse};
use crate::output::{output, OutputFormat};
use crate::Result;

/// List all cycles for a team.
///
/// Fetches cycles from the Linear API and outputs them.
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
/// use lin::commands::cycle::list_cycles;
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// list_cycles(&client, "team-123", OutputFormat::Human)?;
/// # Ok(())
/// # }
/// ```
pub fn list_cycles(client: &GraphQLClient, team_id: &str, format: OutputFormat) -> Result<()> {
    let variables = serde_json::json!({
        "teamId": team_id
    });
    let response: CyclesResponse = client.query(queries::CYCLES_QUERY, variables)?;
    output(&response.team.cycles.nodes, format);
    Ok(())
}

/// Get details of a specific cycle by ID, including its issues.
///
/// Fetches a single cycle from the Linear API and outputs it.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `id` - The cycle's unique identifier
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use lin::api::GraphQLClient;
/// use lin::commands::cycle::get_cycle;
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// get_cycle(&client, "cycle-123", OutputFormat::Human)?;
/// # Ok(())
/// # }
/// ```
pub fn get_cycle(client: &GraphQLClient, id: &str, format: OutputFormat) -> Result<()> {
    let variables = serde_json::json!({
        "id": id
    });
    let response: CycleResponse = client.query(queries::CYCLE_QUERY, variables)?;
    output(&response.cycle, format);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::GraphQLClient;

    #[test]
    fn test_list_cycles_success() {
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
                r#"{
                    "data": {
                        "team": {
                            "id": "team-123",
                            "cycles": {
                                "nodes": [
                                    {
                                        "id": "cycle-1",
                                        "number": 1,
                                        "name": "Sprint 1",
                                        "description": "First sprint",
                                        "startsAt": "2024-01-01T00:00:00.000Z",
                                        "endsAt": "2024-01-14T00:00:00.000Z",
                                        "completedAt": null,
                                        "progress": 50.0,
                                        "completedScopeHistory": [],
                                        "scopeHistory": []
                                    },
                                    {
                                        "id": "cycle-2",
                                        "number": 2,
                                        "name": "Sprint 2",
                                        "description": null,
                                        "startsAt": "2024-01-15T00:00:00.000Z",
                                        "endsAt": "2024-01-28T00:00:00.000Z",
                                        "completedAt": null,
                                        "progress": 0.0,
                                        "completedScopeHistory": [],
                                        "scopeHistory": []
                                    }
                                ]
                            }
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = list_cycles(&client, "team-123", OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_cycles_empty() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with empty cycles
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "team": {
                            "id": "team-123",
                            "cycles": {
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
        let result = list_cycles(&client, "team-123", OutputFormat::Human);

        // Verify success (empty list is still valid)
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_cycles_team_not_found() {
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
        let result = list_cycles(&client, "nonexistent-team", OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Entity not found"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_cycles_api_error() {
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
        let result = list_cycles(&client, "team-123", OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Not authenticated"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_cycles_http_error() {
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
        let result = list_cycles(&client, "team-123", OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("HTTP 401"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_get_cycle_success() {
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
                        "cycle": {
                            "id": "cycle-123",
                            "number": 5,
                            "name": "Sprint 5",
                            "description": "Q1 Final Sprint",
                            "startsAt": "2024-03-18T00:00:00.000Z",
                            "endsAt": "2024-03-31T00:00:00.000Z",
                            "completedAt": null,
                            "progress": 75.5,
                            "completedScopeHistory": [0.0, 25.0, 50.0],
                            "scopeHistory": [100.0, 100.0, 100.0],
                            "issues": {
                                "nodes": [
                                    {
                                        "id": "issue-1",
                                        "identifier": "ENG-123",
                                        "title": "Fix bug",
                                        "description": "A bug fix",
                                        "priority": 2,
                                        "createdAt": "2024-03-18T00:00:00.000Z",
                                        "updatedAt": "2024-03-19T00:00:00.000Z",
                                        "state": {
                                            "id": "state-1",
                                            "name": "In Progress",
                                            "color": "#f2c94c",
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
                                        }
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
        let result = get_cycle(&client, "cycle-123", OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_get_cycle_not_found() {
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
        let result = get_cycle(&client, "nonexistent-cycle", OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Entity not found"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_get_cycle_api_error() {
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
        let result = get_cycle(&client, "cycle-123", OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Not authenticated"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_get_cycle_http_error() {
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
        let result = get_cycle(&client, "cycle-123", OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("HTTP 500"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_cycles_json_output() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "team": {
                            "id": "team-123",
                            "cycles": {
                                "nodes": [
                                    {
                                        "id": "cycle-1",
                                        "number": 1,
                                        "name": "Sprint 1",
                                        "description": null,
                                        "startsAt": "2024-01-01T00:00:00.000Z",
                                        "endsAt": "2024-01-14T00:00:00.000Z",
                                        "completedAt": null,
                                        "progress": 50.0,
                                        "completedScopeHistory": [],
                                        "scopeHistory": []
                                    }
                                ]
                            }
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request with JSON format
        let result = list_cycles(&client, "team-123", OutputFormat::Json);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }
}
