//! Integration tests for label-related functionality.
//!
//! These tests verify label operations against the real Linear API.

mod common;

use lin::api::queries;
use lin::models::LabelsResponse;

/// Test that we can list labels in the workspace.
///
/// This tests read operations and verifies:
/// 1. We can fetch labels from the workspace
/// 2. The response parsing works for collections
#[test]
#[ignore]
fn test_label_list() {
    let client = common::create_client();

    let variables = serde_json::json!({});

    let response: LabelsResponse = client
        .query(queries::LABELS_QUERY, variables)
        .expect("Should be able to list labels");

    // Workspaces might not have labels, so we just verify the query succeeded
    // and the response structure is valid
    println!("Found {} label(s)", response.issue_labels.nodes.len());

    // If there are labels, verify the data structure is valid
    if !response.issue_labels.nodes.is_empty() {
        let first_label = &response.issue_labels.nodes[0];
        assert!(!first_label.id.is_empty(), "Label ID should not be empty");
        assert!(
            !first_label.name.is_empty(),
            "Label name should not be empty"
        );
        assert!(
            !first_label.color.is_empty(),
            "Label color should not be empty"
        );

        println!(
            "First label: {} (color: {})",
            first_label.name, first_label.color
        );
    }
}
