//! Common utilities for integration tests.
//!
//! This module provides shared helpers for integration tests that run
//! against the real Linear API.

use lin::api::queries::issue::ISSUE_DELETE_MUTATION;
use lin::api::GraphQLClient;
use lin::models::IssueDeleteResponse;

/// Test prefix for issues created during tests.
/// This helps identify and clean up test issues if needed.
pub const TEST_ISSUE_PREFIX: &str = "[lin-test]";

/// Get the API token from environment variable.
pub fn get_api_token() -> String {
    std::env::var("LINEAR_API_TOKEN")
        .expect("LINEAR_API_TOKEN environment variable must be set to run integration tests")
}

/// Create a GraphQL client with the API token from environment.
pub fn create_client() -> GraphQLClient {
    GraphQLClient::new(&get_api_token())
}

/// Helper to delete an issue by ID (for cleanup).
pub fn delete_issue(client: &GraphQLClient, issue_id: &str) -> Result<bool, String> {
    let variables = serde_json::json!({
        "id": issue_id
    });

    let response: IssueDeleteResponse = client
        .query(ISSUE_DELETE_MUTATION, variables)
        .map_err(|e| format!("Failed to delete issue: {}", e))?;

    Ok(response.issue_delete.success)
}
