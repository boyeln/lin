//! Integration tests for milestone-related functionality.
//!
//! These tests verify milestone operations against the real Linear API.

mod common;

use lin::api::queries::milestone::{
    PROJECT_MILESTONE_CREATE_MUTATION, PROJECT_MILESTONE_DELETE_MUTATION, PROJECT_MILESTONE_QUERY,
    PROJECT_MILESTONE_UPDATE_MUTATION, PROJECT_MILESTONES_QUERY,
};
use lin::api::queries::project::PROJECTS_QUERY;
use lin::models::{
    ProjectMilestoneCreateResponse, ProjectMilestoneResponse, ProjectMilestoneUpdateResponse,
    ProjectMilestonesResponse, ProjectsResponse,
};

const TEST_MILESTONE_PREFIX: &str = "[lin-test]";

/// Test creating, listing, getting, updating, and deleting a milestone.
///
/// This comprehensive test verifies:
/// 1. Creating a milestone with [lin-test] prefix
/// 2. Listing milestones for the project
/// 3. Getting the specific milestone by ID
/// 4. Updating the milestone (name, description, target date)
/// 5. Deleting the milestone
#[test]
#[ignore]
fn test_milestone_lifecycle() {
    let client = common::create_client();

    // First, get a project to test with
    let projects_response: ProjectsResponse = client
        .query(PROJECTS_QUERY, serde_json::json!({}))
        .expect("Should be able to list projects");

    if projects_response.projects.nodes.is_empty() {
        println!("No projects found in workspace, skipping milestone test");
        return;
    }

    let project = &projects_response.projects.nodes[0];
    let project_id = &project.id;
    println!(
        "Testing milestones for project: {} ({})",
        project.name, project_id
    );

    // 1. Create a test milestone
    let milestone_name = format!("{} Test Milestone", TEST_MILESTONE_PREFIX);
    let create_variables = serde_json::json!({
        "input": {
            "projectId": project_id,
            "name": milestone_name,
            "description": "Integration test milestone",
            "targetDate": "2026-12-31",
            "sortOrder": 999.0
        }
    });

    let create_response: ProjectMilestoneCreateResponse = client
        .query(PROJECT_MILESTONE_CREATE_MUTATION, create_variables)
        .expect("Should be able to create milestone");

    assert!(
        create_response.project_milestone_create.success,
        "Milestone creation should succeed"
    );

    let created_milestone = create_response
        .project_milestone_create
        .project_milestone
        .expect("Created milestone should be returned");

    let milestone_id = created_milestone.id.clone();
    println!(
        "Created milestone: {} ({})",
        created_milestone.name, milestone_id
    );

    // 2. List milestones for the project
    let list_variables = serde_json::json!({
        "projectId": project_id,
        "first": 50
    });

    let list_response: ProjectMilestonesResponse = client
        .query(PROJECT_MILESTONES_QUERY, list_variables)
        .expect("Should be able to list milestones");

    let found_milestone = list_response
        .project_milestones
        .nodes
        .iter()
        .find(|m| m.id == milestone_id);

    assert!(
        found_milestone.is_some(),
        "Created milestone should be in list"
    );
    println!(
        "Found {} milestone(s) for project",
        list_response.project_milestones.nodes.len()
    );

    // 3. Get the specific milestone by ID
    let get_variables = serde_json::json!({
        "id": milestone_id
    });

    let get_response: ProjectMilestoneResponse = client
        .query(PROJECT_MILESTONE_QUERY, get_variables)
        .expect("Should be able to get milestone by ID");

    assert_eq!(
        get_response.project_milestone.id, milestone_id,
        "Milestone ID should match"
    );
    assert_eq!(
        get_response.project_milestone.name, milestone_name,
        "Milestone name should match"
    );
    println!(
        "Retrieved milestone: {}",
        get_response.project_milestone.name
    );

    // 4. Update the milestone
    let updated_name = format!("{} Updated Test Milestone", TEST_MILESTONE_PREFIX);
    let update_variables = serde_json::json!({
        "id": milestone_id,
        "input": {
            "name": updated_name,
            "description": "Updated integration test milestone",
            "targetDate": "2027-06-30"
        }
    });

    let update_response: ProjectMilestoneUpdateResponse = client
        .query(PROJECT_MILESTONE_UPDATE_MUTATION, update_variables)
        .expect("Should be able to update milestone");

    assert!(
        update_response.project_milestone_update.success,
        "Milestone update should succeed"
    );

    let updated_milestone = update_response
        .project_milestone_update
        .project_milestone
        .expect("Updated milestone should be returned");

    assert_eq!(
        updated_milestone.name, updated_name,
        "Milestone name should be updated"
    );
    assert_eq!(
        updated_milestone.description,
        Some("Updated integration test milestone".to_string()),
        "Milestone description should be updated"
    );
    assert_eq!(
        updated_milestone.target_date,
        Some("2027-06-30".to_string()),
        "Milestone target date should be updated"
    );
    println!("Updated milestone: {}", updated_milestone.name);

    // 5. Delete the milestone (cleanup)
    let delete_variables = serde_json::json!({
        "id": milestone_id
    });

    let delete_response: serde_json::Value = client
        .query(PROJECT_MILESTONE_DELETE_MUTATION, delete_variables)
        .expect("Should be able to delete milestone");

    let success = delete_response
        .get("projectMilestoneDelete")
        .and_then(|v| v.get("success"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    assert!(success, "Milestone deletion should succeed");
    println!("Deleted milestone: {}", milestone_id);

    // Verify deletion by trying to list again
    let verify_list_variables = serde_json::json!({
        "projectId": project_id,
        "first": 50
    });

    let verify_list_response: ProjectMilestonesResponse = client
        .query(PROJECT_MILESTONES_QUERY, verify_list_variables)
        .expect("Should be able to list milestones after deletion");

    let still_exists = verify_list_response
        .project_milestones
        .nodes
        .iter()
        .any(|m| m.id == milestone_id);

    assert!(!still_exists, "Deleted milestone should not be in list");
    println!("Verified milestone was deleted");
}

/// Test listing milestones for a project.
///
/// This test verifies:
/// 1. We can fetch milestones for a specific project
/// 2. The response parsing works for collections
#[test]
#[ignore]
fn test_milestone_list() {
    let client = common::create_client();

    // First, get a project to test with
    let projects_response: ProjectsResponse = client
        .query(PROJECTS_QUERY, serde_json::json!({}))
        .expect("Should be able to list projects");

    if projects_response.projects.nodes.is_empty() {
        println!("No projects found in workspace, skipping milestone list test");
        return;
    }

    let project = &projects_response.projects.nodes[0];
    let project_id = &project.id;

    let variables = serde_json::json!({
        "projectId": project_id,
        "first": 50
    });

    let response: ProjectMilestonesResponse = client
        .query(PROJECT_MILESTONES_QUERY, variables)
        .expect("Should be able to list milestones");

    // Milestones might be empty for projects without milestones, which is valid
    println!(
        "Found {} milestone(s) for project {}",
        response.project_milestones.nodes.len(),
        project.name
    );

    // If there are milestones, verify the data is valid
    if !response.project_milestones.nodes.is_empty() {
        let first_milestone = &response.project_milestones.nodes[0];
        assert!(
            !first_milestone.id.is_empty(),
            "Milestone ID should not be empty"
        );
        assert!(
            !first_milestone.name.is_empty(),
            "Milestone name should not be empty"
        );
        assert!(
            !first_milestone.status.is_empty(),
            "Milestone status should not be empty"
        );

        println!(
            "First milestone: {} (status: {}, progress: {:.1}%)",
            first_milestone.name, first_milestone.status, first_milestone.progress
        );
    }
}
