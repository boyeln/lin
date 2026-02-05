//! Integration tests for user-related functionality.
//!
//! These tests verify authentication and user operations against the real Linear API.

mod common;

use lin::api::queries::organization::VIEWER_QUERY;
use lin::api::queries::user::USERS_QUERY;
use lin::models::{UsersResponse, ViewerResponse};

/// Test that we can authenticate and get the current user's information.
///
/// This is the most basic integration test - it verifies that:
/// 1. The API token is valid
/// 2. We can make authenticated requests
/// 3. The response parsing works correctly
#[test]
#[ignore]
fn test_user_me() {
    let client = common::create_client();

    let response: ViewerResponse = client
        .query(VIEWER_QUERY, serde_json::json!({}))
        .expect("Should be able to fetch current user");

    // Verify we got valid user data
    assert!(
        !response.viewer.id.is_empty(),
        "User ID should not be empty"
    );
    assert!(
        !response.viewer.name.is_empty(),
        "User name should not be empty"
    );
    assert!(
        !response.viewer.email.is_empty(),
        "User email should not be empty"
    );
    assert!(response.viewer.active, "User should be active");

    println!("Successfully authenticated as: {}", response.viewer.name);
}

/// Test listing users in the organization.
///
/// This test verifies:
/// 1. We can fetch a list of users
/// 2. The response parsing works for user collections
/// 3. User data is valid
#[test]
#[ignore]
fn test_user_list() {
    let client = common::create_client();

    let variables = serde_json::json!({
        "first": 10
    });

    let response: UsersResponse = client
        .query(USERS_QUERY, variables)
        .expect("Should be able to list users");

    // Verify we got at least one user (test account should have at least the authenticated user)
    assert!(
        !response.users.nodes.is_empty(),
        "Should have at least one user"
    );

    // Verify user data is valid
    let first_user = &response.users.nodes[0];
    assert!(!first_user.id.is_empty(), "User ID should not be empty");
    assert!(!first_user.name.is_empty(), "User name should not be empty");
    assert!(
        !first_user.email.is_empty(),
        "User email should not be empty"
    );

    println!(
        "Found {} user(s), first user: {} ({})",
        response.users.nodes.len(),
        first_user.name,
        first_user.email
    );
}
