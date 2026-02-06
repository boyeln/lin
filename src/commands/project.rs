//! Project management commands.
//!
//! Commands for listing and viewing project information from Linear.

use crate::Result;
use crate::api::GraphQLClient;
use crate::api::queries::project::{PROJECT_QUERY, PROJECTS_QUERY};
use crate::config::Config;
use crate::models::{ProjectResponse, ProjectsResponse};
use crate::output::{OutputFormat, output};

/// Options for listing projects.
#[derive(Debug, Clone, Default)]
pub struct ProjectListOptions {
    // No filters - projects are organization-wide
}

/// List all projects in the organization.
///
/// Fetches projects from the Linear API, caches their slugs, and outputs them.
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
/// use lin::commands::project::{list_projects, ProjectListOptions};
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// let options = ProjectListOptions::default();
/// list_projects(&client, options, OutputFormat::Human)?;
/// # Ok(())
/// # }
/// ```
pub fn list_projects(
    client: &GraphQLClient,
    _options: ProjectListOptions,
    format: OutputFormat,
) -> Result<()> {
    let response: ProjectsResponse = client.query(PROJECTS_QUERY, serde_json::json!({}))?;

    // Cache project slugs (ignore errors if config not available)
    let projects_for_cache: Vec<(String, String)> = response
        .projects
        .nodes
        .iter()
        .map(|p| (p.id.clone(), p.name.clone()))
        .collect();

    if let Ok(mut config) = Config::load() {
        let _ = config.cache_projects(projects_for_cache);
        let _ = config.save();
    }

    output(&response.projects.nodes, format);
    Ok(())
}

/// Get details of a specific project by slug or ID.
///
/// Fetches a single project from the Linear API and outputs it.
/// Accepts either a project slug (e.g., "q1-backend") or UUID.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `slug_or_id` - The project's slug or unique identifier
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use lin::api::GraphQLClient;
/// use lin::commands::project::get_project;
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// get_project(&client, "q1-backend", OutputFormat::Human)?;
/// # Ok(())
/// # }
/// ```
pub fn get_project(client: &GraphQLClient, slug_or_id: &str, format: OutputFormat) -> Result<()> {
    // Resolve slug to UUID if needed (ignore errors if config not available)
    let project_id = Config::load()
        .ok()
        .and_then(|config| config.get_project_id(slug_or_id))
        .unwrap_or_else(|| slug_or_id.to_string());

    let variables = serde_json::json!({
        "id": project_id
    });
    let response: ProjectResponse = client.query(PROJECT_QUERY, variables)?;

    // Cache this project for future slug lookups (ignore errors if config not available)
    if let Ok(mut config) = Config::load() {
        let _ = config.cache_projects(vec![(
            response.project.id.clone(),
            response.project.name.clone(),
        )]);
        let _ = config.save();
    }

    output(&response.project, format);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::GraphQLClient;

    #[test]
    fn test_list_projects_success() {
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
                        "projects": {
                            "nodes": [
                                {
                                    "id": "project-1",
                                    "name": "Q1 Roadmap",
                                    "description": "First quarter roadmap",
                                    "state": "started",
                                    "createdAt": "2024-01-01T00:00:00.000Z",
                                    "updatedAt": "2024-01-15T00:00:00.000Z",
                                    "targetDate": "2024-03-31",
                                    "startDate": "2024-01-01",
                                    "progress": 25.0
                                },
                                {
                                    "id": "project-2",
                                    "name": "Feature X",
                                    "description": null,
                                    "state": "planned",
                                    "createdAt": "2024-02-01T00:00:00.000Z",
                                    "updatedAt": "2024-02-01T00:00:00.000Z",
                                    "targetDate": null,
                                    "startDate": null,
                                    "progress": 0.0
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
        let options = ProjectListOptions::default();
        let result = list_projects(&client, options, OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_projects_empty() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with empty projects
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "projects": {
                            "nodes": []
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let options = ProjectListOptions::default();
        let result = list_projects(&client, options, OutputFormat::Human);

        // Verify success (empty list is still valid)
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_projects_api_error() {
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
        let options = ProjectListOptions::default();
        let result = list_projects(&client, options, OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Not authenticated"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_projects_http_error() {
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
        let options = ProjectListOptions::default();
        let result = list_projects(&client, options, OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("HTTP 401"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_get_project_success() {
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
                        "project": {
                            "id": "project-123",
                            "name": "Q1 Roadmap",
                            "description": "First quarter roadmap",
                            "state": "started",
                            "createdAt": "2024-01-01T00:00:00.000Z",
                            "updatedAt": "2024-01-15T00:00:00.000Z",
                            "targetDate": "2024-03-31",
                            "startDate": "2024-01-01",
                            "progress": 75.5
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = get_project(&client, "project-123", OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_get_project_with_null_fields() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with null optional fields
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "project": {
                            "id": "project-456",
                            "name": "Simple Project",
                            "description": null,
                            "state": "planned",
                            "createdAt": "2024-01-01T00:00:00.000Z",
                            "updatedAt": "2024-01-01T00:00:00.000Z",
                            "targetDate": null,
                            "startDate": null,
                            "progress": 0.0
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = get_project(&client, "project-456", OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_get_project_not_found() {
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
        let result = get_project(&client, "nonexistent-project", OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Entity not found"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_get_project_api_error() {
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
        let result = get_project(&client, "project-123", OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Not authenticated"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_get_project_http_error() {
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
        let result = get_project(&client, "project-123", OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("HTTP 500"));

        // Verify mock was called
        mock.assert();
    }
}
