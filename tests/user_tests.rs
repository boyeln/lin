//! Integration tests for user-related functionality.
//!
//! These tests verify authentication and user operations against the real Linear API.

mod common;

use lin::api::queries;
use lin::models::ViewerResponse;

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
        .query(queries::VIEWER_QUERY, serde_json::json!({}))
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
