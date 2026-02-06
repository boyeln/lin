//! Team management commands.
//!
//! Commands for listing and viewing team information from Linear.

use crate::Result;
use crate::api::GraphQLClient;
use crate::api::queries::team::{TEAM_QUERY, TEAMS_QUERY};
use crate::config::Config;
use crate::models::{TeamResponse, TeamsResponse};
use crate::output::{OutputFormat, output};
use serde::Serialize;

/// List all teams in the organization.
///
/// Fetches teams from the Linear API and outputs them.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use lin::api::GraphQLClient;
/// use lin::commands::team::list_teams;
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// list_teams(&client, OutputFormat::Human)?;
/// # Ok(())
/// # }
/// ```
pub fn list_teams(client: &GraphQLClient, format: OutputFormat) -> Result<()> {
    let response: TeamsResponse = client.query(TEAMS_QUERY, serde_json::json!({}))?;
    output(&response.teams.nodes, format);
    Ok(())
}

/// Get details of a specific team by ID.
///
/// Fetches a single team from the Linear API and outputs it.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `id` - The team's unique identifier
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use lin::api::GraphQLClient;
/// use lin::commands::team::get_team;
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// get_team(&client, "team-123", OutputFormat::Human)?;
/// # Ok(())
/// # }
/// ```
pub fn get_team(client: &GraphQLClient, id: &str, format: OutputFormat) -> Result<()> {
    let variables = serde_json::json!({
        "id": id
    });
    let response: TeamResponse = client.query(TEAM_QUERY, variables)?;
    output(&response.team, format);
    Ok(())
}

/// Switch the current team or show the current team.
///
/// If a team key is provided, switches to that team.
/// If no team key is provided, shows the current team.
///
/// # Arguments
///
/// * `team_key` - Optional team key to switch to
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use lin::commands::team::switch_team;
/// use lin::output::OutputFormat;
///
/// // Switch to ENG team
/// switch_team(Some("ENG".to_string()), OutputFormat::Human)?;
///
/// // Show current team
/// switch_team(None, OutputFormat::Human)?;
/// # Ok(())
/// # }
/// ```
pub fn switch_team(team_key: Option<String>, format: OutputFormat) -> Result<()> {
    let mut config = Config::load()?;

    if let Some(team) = team_key {
        // Switch to the specified team
        config.set_current_team(&team)?;
        config.save()?;

        let response = TeamSwitchResponse { team: team.clone() };
        output(&response, format);
    } else {
        // Show current team
        match config.get_current_team() {
            Some(team) => {
                let response = CurrentTeamResponse {
                    current_team: Some(team),
                };
                output(&response, format);
            }
            None => {
                let response = CurrentTeamResponse { current_team: None };
                output(&response, format);
            }
        }
    }

    Ok(())
}

// Response types for JSON output

#[derive(Debug, Serialize)]
struct TeamSwitchResponse {
    team: String,
}

impl crate::output::HumanDisplay for TeamSwitchResponse {
    fn human_fmt(&self) -> String {
        format!("✓ Switched to team '{}'", self.team)
    }
}

#[derive(Debug, Serialize)]
struct CurrentTeamResponse {
    current_team: Option<String>,
}

impl crate::output::HumanDisplay for CurrentTeamResponse {
    fn human_fmt(&self) -> String {
        match &self.current_team {
            Some(team) => format!("Current team: {}", team),
            None => {
                "No team selected. Use 'lin team switch <key>' to set a default team.".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::GraphQLClient;

    #[test]
    fn test_list_teams_success() {
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
                        "teams": {
                            "nodes": [
                                {
                                    "id": "team-1",
                                    "key": "ENG",
                                    "name": "Engineering",
                                    "description": "The engineering team",
                                    "issueEstimationType": {
                                        "id": "tshirt",
                                        "name": "T-Shirt Sizes",
                                        "values": [1, 2, 3, 5, 8]
                                    }
                                },
                                {
                                    "id": "team-2",
                                    "key": "DES",
                                    "name": "Design",
                                    "description": null,
                                    "issueEstimationType": null
                                }
                            ]
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = list_teams(&client, OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_teams_empty() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with empty teams
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "teams": {
                            "nodes": []
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = list_teams(&client, OutputFormat::Human);

        // Verify success (empty list is still valid)
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_teams_api_error() {
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

        // Make request
        let result = list_teams(&client, OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Not authenticated"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_teams_http_error() {
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
        let result = list_teams(&client, OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("HTTP 401"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_get_team_success() {
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
                            "key": "ENG",
                            "name": "Engineering",
                            "description": "The engineering team",
                            "issueEstimationType": {
                                "id": "fibonacci",
                                "name": "Fibonacci",
                                "values": [1, 2, 3, 5, 8, 13]
                            }
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = get_team(&client, "team-123", OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_get_team_with_null_description() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with null description
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "team": {
                            "id": "team-456",
                            "key": "DES",
                            "name": "Design",
                            "description": null,
                            "issueEstimationType": null
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = get_team(&client, "team-456", OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_get_team_not_found() {
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
        let result = get_team(&client, "nonexistent-team", OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Entity not found"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_get_team_api_error() {
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
        let result = get_team(&client, "team-123", OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Not authenticated"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_get_team_http_error() {
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
        let result = get_team(&client, "team-123", OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("HTTP 500"));

        // Verify mock was called
        mock.assert();
    }
}
