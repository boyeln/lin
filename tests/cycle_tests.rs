//! Integration tests for cycle-related functionality.
//!
//! These tests verify cycle operations against the real Linear API.

mod common;

use lin::api::queries::cycle::{CYCLE_QUERY, CYCLES_QUERY};
use lin::api::queries::team::TEAMS_QUERY;
use lin::models::{CycleResponse, CyclesResponse, TeamsResponse};

/// Test that we can list cycles for a team.
///
/// This tests read operations and verifies:
/// 1. We can fetch cycles for a team
/// 2. The response parsing works correctly
/// 3. Cycles have the expected fields
#[test]
#[ignore]
fn test_cycles_list() {
    let client = common::create_client();

    // First, get a team to use for the cycles query
    let teams_response: TeamsResponse = client
        .query(TEAMS_QUERY, serde_json::json!({"first": 1}))
        .expect("Should be able to list teams");

    assert!(
        !teams_response.teams.nodes.is_empty(),
        "Should have at least one team to test cycles"
    );

    let team = &teams_response.teams.nodes[0];
    println!("Testing cycles for team: {} ({})", team.name, team.id);

    // Now fetch cycles for this team
    let variables = serde_json::json!({
        "teamId": team.id
    });

    let response: CyclesResponse = client
        .query(CYCLES_QUERY, variables)
        .expect("Should be able to list cycles");

    // Cycles might be empty for teams without sprints, which is valid
    println!(
        "Found {} cycle(s) for team {}",
        response.team.cycles.nodes.len(),
        team.name
    );

    // If there are cycles, verify the data is valid
    if !response.team.cycles.nodes.is_empty() {
        let first_cycle = &response.team.cycles.nodes[0];
        assert!(!first_cycle.id.is_empty(), "Cycle ID should not be empty");
        assert!(first_cycle.number > 0, "Cycle number should be positive");
        // Progress should be between 0 and 100
        assert!(
            first_cycle.progress >= 0.0 && first_cycle.progress <= 100.0,
            "Cycle progress should be between 0 and 100"
        );

        println!(
            "First cycle: {} (progress: {:.1}%)",
            first_cycle
                .name
                .as_ref()
                .map(|n| n.as_str())
                .unwrap_or(&format!("Cycle {}", first_cycle.number)),
            first_cycle.progress
        );
    }
}

/// Test that querying cycles for a non-existent team returns an error.
#[test]
#[ignore]
fn test_cycles_nonexistent_team() {
    let client = common::create_client();

    let variables = serde_json::json!({
        "teamId": "nonexistent-team-id-12345"
    });

    let result: Result<CyclesResponse, _> = client.query(CYCLES_QUERY, variables);

    assert!(
        result.is_err(),
        "Should return an error for non-existent team"
    );

    let err = result.unwrap_err();
    println!("Expected error for non-existent team: {}", err);
}

/// Test getting a specific cycle by ID.
///
/// This test verifies:
/// 1. Listing cycles to get a valid cycle ID
/// 2. Getting the cycle by its ID (with issues)
/// 3. Verifying the cycle data matches
#[test]
#[ignore]
fn test_cycle_get() {
    let client = common::create_client();

    // First, get a team
    let teams_response: TeamsResponse = client
        .query(TEAMS_QUERY, serde_json::json!({"first": 1}))
        .expect("Should be able to list teams");

    assert!(
        !teams_response.teams.nodes.is_empty(),
        "Should have at least one team"
    );

    let team = &teams_response.teams.nodes[0];

    // List cycles for the team
    let cycles_response: CyclesResponse = client
        .query(CYCLES_QUERY, serde_json::json!({"teamId": &team.id}))
        .expect("Should be able to list cycles");

    // Skip if no cycles exist
    if cycles_response.team.cycles.nodes.is_empty() {
        println!("No cycles found for team {}, skipping get test", team.name);
        return;
    }

    let first_cycle = &cycles_response.team.cycles.nodes[0];
    let cycle_id = &first_cycle.id;

    println!(
        "Testing get for cycle {} (ID: {})",
        first_cycle
            .name
            .as_ref()
            .map(|n| n.as_str())
            .unwrap_or(&format!("Cycle {}", first_cycle.number)),
        cycle_id
    );

    // Get the cycle by ID
    let variables = serde_json::json!({
        "id": cycle_id
    });

    let response: CycleResponse = client
        .query(CYCLE_QUERY, variables)
        .expect("Should be able to get cycle by ID");

    // Verify the cycle data matches
    assert_eq!(response.cycle.id, *cycle_id, "Cycle ID should match");
    assert_eq!(
        response.cycle.number, first_cycle.number,
        "Cycle number should match"
    );

    println!(
        "Successfully retrieved cycle {} with {} issue(s)",
        response.cycle.number,
        response.cycle.issues.nodes.len()
    );
}
