//! Integration tests for team-related functionality.
//!
//! These tests verify team operations against the real Linear API.

mod common;

use lin::api::queries;
use lin::models::TeamsResponse;

/// Test that we can list teams in the organization.
///
/// This tests read operations and verifies:
/// 1. We can fetch teams
/// 2. The response parsing works for collections
#[test]
#[ignore]
fn test_team_list() {
    let client = common::create_client();

    let variables = serde_json::json!({
        "first": 10
    });

    let response: TeamsResponse = client
        .query(queries::TEAMS_QUERY, variables)
        .expect("Should be able to list teams");

    // Verify we got at least one team (test account should have at least one)
    assert!(
        !response.teams.nodes.is_empty(),
        "Should have at least one team"
    );

    // Verify team data is valid
    let first_team = &response.teams.nodes[0];
    assert!(!first_team.id.is_empty(), "Team ID should not be empty");
    assert!(!first_team.key.is_empty(), "Team key should not be empty");
    assert!(!first_team.name.is_empty(), "Team name should not be empty");

    println!(
        "Found {} team(s), first team: {} ({})",
        response.teams.nodes.len(),
        first_team.name,
        first_team.key
    );
}
