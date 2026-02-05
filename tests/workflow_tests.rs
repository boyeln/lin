//! Integration tests for workflow state-related functionality.
//!
//! These tests verify workflow state operations against the real Linear API.

mod common;

use lin::api::queries::team::TEAMS_QUERY;
use lin::api::queries::workflow::WORKFLOW_STATES_QUERY;
use lin::models::{TeamsResponse, WorkflowStatesResponse};

/// Test that we can list workflow states for a team.
///
/// This tests read operations and verifies:
/// 1. We can fetch workflow states for a team
/// 2. The response parsing works correctly
/// 3. Workflow states have the expected fields
#[test]
#[ignore]
fn test_workflow_states_list() {
    let client = common::create_client();

    // First, get a team to use for the workflow states query
    let teams_response: TeamsResponse = client
        .query(TEAMS_QUERY, serde_json::json!({"first": 1}))
        .expect("Should be able to list teams");

    assert!(
        !teams_response.teams.nodes.is_empty(),
        "Should have at least one team to test workflow states"
    );

    let team = &teams_response.teams.nodes[0];
    println!(
        "Testing workflow states for team: {} ({})",
        team.name, team.id
    );

    // Now fetch workflow states for this team
    let variables = serde_json::json!({
        "id": team.id
    });

    let response: WorkflowStatesResponse = client
        .query(WORKFLOW_STATES_QUERY, variables)
        .expect("Should be able to list workflow states");

    // Verify we got workflow states (every team should have at least the default states)
    assert!(
        !response.team.states.nodes.is_empty(),
        "Should have at least one workflow state"
    );

    // Verify workflow state data is valid
    let first_state = &response.team.states.nodes[0];
    assert!(
        !first_state.id.is_empty(),
        "Workflow state ID should not be empty"
    );
    assert!(
        !first_state.name.is_empty(),
        "Workflow state name should not be empty"
    );
    assert!(
        !first_state.color.is_empty(),
        "Workflow state color should not be empty"
    );
    assert!(
        !first_state.type_.is_empty(),
        "Workflow state type should not be empty"
    );

    // Verify the type is one of the expected values
    let valid_types = ["backlog", "unstarted", "started", "completed", "canceled"];
    assert!(
        valid_types.contains(&first_state.type_.as_str()),
        "Workflow state type '{}' should be one of: {:?}",
        first_state.type_,
        valid_types
    );

    println!(
        "Found {} workflow state(s) for team {}:",
        response.team.states.nodes.len(),
        team.name
    );
    for state in &response.team.states.nodes {
        println!("  - {} ({}) [{}]", state.name, state.type_, state.color);
    }
}

/// Test that querying workflow states for a non-existent team returns an error.
#[test]
#[ignore]
fn test_workflow_states_nonexistent_team() {
    let client = common::create_client();

    let variables = serde_json::json!({
        "id": "nonexistent-team-id-12345"
    });

    let result: Result<WorkflowStatesResponse, _> = client.query(WORKFLOW_STATES_QUERY, variables);

    assert!(
        result.is_err(),
        "Should return an error for non-existent team"
    );

    let err = result.unwrap_err();
    println!("Expected error for non-existent team: {}", err);
}
