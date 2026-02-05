//! Integration tests for project-related functionality.
//!
//! These tests verify project operations against the real Linear API.

mod common;

use lin::api::queries::project::PROJECTS_QUERY;
use lin::models::ProjectsResponse;

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
