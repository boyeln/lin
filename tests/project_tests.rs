//! Integration tests for project-related functionality.
//!
//! These tests verify project operations against the real Linear API.

mod common;

use lin::api::queries::project::{PROJECT_QUERY, PROJECTS_QUERY};
use lin::models::{ProjectResponse, ProjectsResponse};

/// Test that we can list projects in the organization.
///
/// This tests read operations and verifies:
/// 1. We can fetch projects
/// 2. The response parsing works for collections
#[test]
#[ignore]
fn test_project_list() {
    let client = common::create_client();

    let variables = serde_json::json!({});

    let response: ProjectsResponse = client
        .query(PROJECTS_QUERY, variables)
        .expect("Should be able to list projects");

    // Projects might be empty for new workspaces, which is valid
    println!("Found {} project(s)", response.projects.nodes.len());

    // If there are projects, verify the data is valid
    if !response.projects.nodes.is_empty() {
        let first_project = &response.projects.nodes[0];
        assert!(
            !first_project.id.is_empty(),
            "Project ID should not be empty"
        );
        assert!(
            !first_project.name.is_empty(),
            "Project name should not be empty"
        );
        assert!(
            !first_project.state.is_empty(),
            "Project state should not be empty"
        );

        println!(
            "First project: {} (state: {}, progress: {:.1}%)",
            first_project.name, first_project.state, first_project.progress
        );
    }
}

/// Test getting a specific project by ID.
///
/// This test verifies:
/// 1. Listing projects to get a valid project ID
/// 2. Getting the project by its ID
/// 3. Verifying the project data matches
#[test]
#[ignore]
fn test_project_get() {
    let client = common::create_client();

    // First, list projects to get a valid project ID
    let projects_response: ProjectsResponse = client
        .query(PROJECTS_QUERY, serde_json::json!({}))
        .expect("Should be able to list projects");

    // Skip if no projects exist
    if projects_response.projects.nodes.is_empty() {
        println!("No projects found in workspace, skipping get test");
        return;
    }

    let first_project = &projects_response.projects.nodes[0];
    let project_id = &first_project.id;

    println!(
        "Testing get for project: {} ({})",
        first_project.name, project_id
    );

    // Get the project by ID
    let variables = serde_json::json!({
        "id": project_id
    });

    let response: ProjectResponse = client
        .query(PROJECT_QUERY, variables)
        .expect("Should be able to get project by ID");

    // Verify the project data matches
    assert_eq!(response.project.id, *project_id, "Project ID should match");
    assert_eq!(
        response.project.name, first_project.name,
        "Project name should match"
    );
    assert_eq!(
        response.project.state, first_project.state,
        "Project state should match"
    );

    println!("Successfully retrieved project: {}", response.project.name);
}
