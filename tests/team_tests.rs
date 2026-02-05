//! Integration tests for team-related functionality.
//!
//! These tests verify team operations against the real Linear API.

mod common;

use lin::api::queries::team::{TEAM_QUERY, TEAMS_QUERY};
use lin::models::{TeamResponse, TeamsResponse};

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
        .query(TEAMS_QUERY, variables)
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

/// Test getting a specific team by ID.
///
/// This test verifies:
/// 1. Listing teams to get a valid team ID
/// 2. Getting the team by its ID
/// 3. Verifying the team data matches
#[test]
#[ignore]
fn test_team_get() {
    let client = common::create_client();

    // First, list teams to get a valid team ID
    let teams_response: TeamsResponse = client
        .query(TEAMS_QUERY, serde_json::json!({"first": 1}))
        .expect("Should be able to list teams");

    assert!(
        !teams_response.teams.nodes.is_empty(),
        "Should have at least one team"
    );

    let first_team = &teams_response.teams.nodes[0];
    let team_id = &first_team.id;

    println!("Testing get for team: {} ({})", first_team.name, team_id);

    // Get the team by ID
    let variables = serde_json::json!({
        "id": team_id
    });

    let response: TeamResponse = client
        .query(TEAM_QUERY, variables)
        .expect("Should be able to get team by ID");

    // Verify the team data matches
    assert_eq!(response.team.id, *team_id, "Team ID should match");
    assert_eq!(response.team.name, first_team.name, "Team name should match");
    assert_eq!(response.team.key, first_team.key, "Team key should match");

    println!("Successfully retrieved team: {}", response.team.name);
}
