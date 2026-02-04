//! Integration tests for cycle-related functionality.
//!
//! These tests verify cycle operations against the real Linear API.

mod common;

use lin::api::queries;
use lin::models::{CyclesResponse, TeamsResponse};

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
        .query(queries::TEAMS_QUERY, serde_json::json!({"first": 1}))
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
        .query(queries::CYCLES_QUERY, variables)
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

    let result: Result<CyclesResponse, _> = client.query(queries::CYCLES_QUERY, variables);

    assert!(
        result.is_err(),
        "Should return an error for non-existent team"
    );

    let err = result.unwrap_err();
    println!("Expected error for non-existent team: {}", err);
}
