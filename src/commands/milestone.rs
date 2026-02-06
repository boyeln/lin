//! Project milestone management commands.
//!
//! Commands for creating, updating, deleting, and viewing project milestones from Linear.

use crate::Result;
use crate::api::GraphQLClient;
use crate::api::queries::milestone::{
    PROJECT_MILESTONE_CREATE_MUTATION, PROJECT_MILESTONE_DELETE_MUTATION, PROJECT_MILESTONE_QUERY,
    PROJECT_MILESTONE_UPDATE_MUTATION, PROJECT_MILESTONES_QUERY,
};
use crate::config::Config;
use crate::error::LinError;
use crate::models::{
    ProjectMilestoneCreateResponse, ProjectMilestoneResponse, ProjectMilestoneUpdateResponse,
    ProjectMilestonesResponse,
};
use crate::output::{OutputFormat, output};

/// List all milestones for a specific project.
///
/// Fetches milestones from the Linear API and outputs them.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `project_slug_or_id` - The project's slug or unique identifier
/// * `format` - The output format (Human or Json)
pub fn list_milestones(
    client: &GraphQLClient,
    project_slug_or_id: &str,
    format: OutputFormat,
) -> Result<()> {
    // Resolve project slug to UUID if needed
    let project_id = Config::load()
        .ok()
        .and_then(|config| config.get_project_id(project_slug_or_id))
        .unwrap_or_else(|| project_slug_or_id.to_string());

    let variables = serde_json::json!({
        "projectId": project_id,
        "first": 50
    });

    let response: ProjectMilestonesResponse = client.query(PROJECT_MILESTONES_QUERY, variables)?;

    output(&response.project_milestones.nodes, format);
    Ok(())
}

/// Get details of a specific milestone by ID.
///
/// Fetches a single milestone from the Linear API and outputs it.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `id` - The milestone's unique identifier
/// * `format` - The output format (Human or Json)
pub fn get_milestone(client: &GraphQLClient, id: &str, format: OutputFormat) -> Result<()> {
    let variables = serde_json::json!({
        "id": id
    });

    let response: ProjectMilestoneResponse = client.query(PROJECT_MILESTONE_QUERY, variables)?;

    output(&response.project_milestone, format);
    Ok(())
}

/// Options for creating a milestone.
#[derive(Debug, Clone)]
pub struct MilestoneCreateOptions {
    /// The project's slug or unique identifier.
    pub project: String,
    /// The milestone's name.
    pub name: String,
    /// Optional description of the milestone.
    pub description: Option<String>,
    /// Target date for the milestone (YYYY-MM-DD format).
    pub target_date: Option<String>,
    /// Sort order of the milestone.
    pub sort_order: Option<f64>,
}

/// Create a new project milestone.
///
/// Creates a milestone in the Linear API and outputs the result.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `options` - The milestone creation options
/// * `format` - The output format (Human or Json)
pub fn create_milestone(
    client: &GraphQLClient,
    options: MilestoneCreateOptions,
    format: OutputFormat,
) -> Result<()> {
    // Resolve project slug to UUID if needed
    let project_id = Config::load()
        .ok()
        .and_then(|config| config.get_project_id(&options.project))
        .unwrap_or_else(|| options.project.clone());

    // Build input object with only non-None fields
    let mut input = serde_json::Map::new();
    input.insert("projectId".to_string(), serde_json::json!(project_id));
    input.insert("name".to_string(), serde_json::json!(options.name));

    if let Some(description) = options.description {
        input.insert("description".to_string(), serde_json::json!(description));
    }

    if let Some(target_date) = options.target_date {
        input.insert("targetDate".to_string(), serde_json::json!(target_date));
    }

    if let Some(sort_order) = options.sort_order {
        input.insert("sortOrder".to_string(), serde_json::json!(sort_order));
    }

    let variables = serde_json::json!({
        "input": input
    });

    let response: ProjectMilestoneCreateResponse =
        client.query(PROJECT_MILESTONE_CREATE_MUTATION, variables)?;

    // Validate success
    if !response.project_milestone_create.success {
        return Err(LinError::api("Failed to create milestone"));
    }

    let milestone = response
        .project_milestone_create
        .project_milestone
        .ok_or_else(|| LinError::api("No milestone returned in response"))?;

    output(&milestone, format);
    Ok(())
}

/// Options for updating a milestone.
#[derive(Debug, Clone)]
pub struct MilestoneUpdateOptions {
    /// The milestone's unique identifier.
    pub id: String,
    /// New name for the milestone (optional).
    pub name: Option<String>,
    /// New description for the milestone (optional).
    pub description: Option<String>,
    /// New target date for the milestone (optional).
    pub target_date: Option<String>,
    /// New sort order for the milestone (optional).
    pub sort_order: Option<f64>,
}

/// Update an existing project milestone.
///
/// Updates a milestone in the Linear API and outputs the result.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `options` - The milestone update options
/// * `format` - The output format (Human or Json)
pub fn update_milestone(
    client: &GraphQLClient,
    options: MilestoneUpdateOptions,
    format: OutputFormat,
) -> Result<()> {
    // Build input object with only non-None fields
    let mut input = serde_json::Map::new();

    if let Some(name) = options.name {
        input.insert("name".to_string(), serde_json::json!(name));
    }

    if let Some(description) = options.description {
        input.insert("description".to_string(), serde_json::json!(description));
    }

    if let Some(target_date) = options.target_date {
        input.insert("targetDate".to_string(), serde_json::json!(target_date));
    }

    if let Some(sort_order) = options.sort_order {
        input.insert("sortOrder".to_string(), serde_json::json!(sort_order));
    }

    let variables = serde_json::json!({
        "id": options.id,
        "input": input
    });

    let response: ProjectMilestoneUpdateResponse =
        client.query(PROJECT_MILESTONE_UPDATE_MUTATION, variables)?;

    // Validate success
    if !response.project_milestone_update.success {
        return Err(LinError::api("Failed to update milestone"));
    }

    let milestone = response
        .project_milestone_update
        .project_milestone
        .ok_or_else(|| LinError::api("No milestone returned in response"))?;

    output(&milestone, format);
    Ok(())
}

/// Delete a project milestone.
///
/// Deletes a milestone from the Linear API.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `id` - The milestone's unique identifier
pub fn delete_milestone(client: &GraphQLClient, id: &str) -> Result<()> {
    let variables = serde_json::json!({
        "id": id
    });

    let response: serde_json::Value = client.query(PROJECT_MILESTONE_DELETE_MUTATION, variables)?;

    // Validate success
    let success = response
        .get("projectMilestoneDelete")
        .and_then(|v| v.get("success"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !success {
        return Err(LinError::api("Failed to delete milestone"));
    }

    println!("Milestone deleted successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_milestones_success() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .match_header("content-type", "application/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "projectMilestones": {
                            "nodes": [
                                {
                                    "id": "milestone-1",
                                    "name": "Sprint 1",
                                    "description": "First sprint",
                                    "targetDate": "2024-03-31",
                                    "sortOrder": 100.0,
                                    "status": "next",
                                    "progress": 45.5,
                                    "createdAt": "2024-01-01T00:00:00.000Z",
                                    "updatedAt": "2024-01-15T00:00:00.000Z"
                                }
                            ]
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = list_milestones(&client, "project-123", OutputFormat::Human);

        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_get_milestone_success() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "projectMilestone": {
                            "id": "milestone-123",
                            "name": "Sprint 1",
                            "description": "First sprint",
                            "targetDate": "2024-03-31",
                            "sortOrder": 100.0,
                            "status": "next",
                            "progress": 45.5,
                            "createdAt": "2024-01-01T00:00:00.000Z",
                            "updatedAt": "2024-01-15T00:00:00.000Z"
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = get_milestone(&client, "milestone-123", OutputFormat::Human);

        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_create_milestone_success() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "projectMilestoneCreate": {
                            "success": true,
                            "projectMilestone": {
                                "id": "milestone-new",
                                "name": "New Milestone",
                                "description": "Test milestone",
                                "targetDate": "2024-06-30",
                                "sortOrder": 50.0,
                                "status": "unstarted",
                                "progress": 0.0,
                                "createdAt": "2024-02-01T00:00:00.000Z",
                                "updatedAt": "2024-02-01T00:00:00.000Z"
                            }
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = MilestoneCreateOptions {
            project: "project-123".to_string(),
            name: "New Milestone".to_string(),
            description: Some("Test milestone".to_string()),
            target_date: Some("2024-06-30".to_string()),
            sort_order: Some(50.0),
        };
        let result = create_milestone(&client, options, OutputFormat::Human);

        if let Err(e) = &result {
            eprintln!("Error: {}", e);
        }
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_update_milestone_success() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "projectMilestoneUpdate": {
                            "success": true,
                            "projectMilestone": {
                                "id": "milestone-123",
                                "name": "Updated Milestone",
                                "description": "Updated description",
                                "targetDate": "2024-07-31",
                                "sortOrder": 75.0,
                                "status": "next",
                                "progress": 30.0,
                                "createdAt": "2024-01-01T00:00:00.000Z",
                                "updatedAt": "2024-02-15T00:00:00.000Z"
                            }
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let options = MilestoneUpdateOptions {
            id: "milestone-123".to_string(),
            name: Some("Updated Milestone".to_string()),
            description: Some("Updated description".to_string()),
            target_date: Some("2024-07-31".to_string()),
            sort_order: Some(75.0),
        };
        let result = update_milestone(&client, options, OutputFormat::Human);

        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_delete_milestone_success() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "projectMilestoneDelete": {
                            "success": true
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = delete_milestone(&client, "milestone-123");

        assert!(result.is_ok());
        mock.assert();
    }
}
