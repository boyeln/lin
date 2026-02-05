//! Integration tests for label-related functionality.
//!
//! These tests verify label operations against the real Linear API.

mod common;

use lin::api::queries::label::{LABEL_QUERY, LABELS_QUERY};
use lin::models::{LabelResponse, LabelsResponse};

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
        .query(LABELS_QUERY, variables)
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

/// Test getting a specific label by ID.
///
/// This test verifies:
/// 1. Listing labels to get a valid label ID
/// 2. Getting the label by its ID
/// 3. Verifying the label data matches
#[test]
#[ignore]
fn test_label_get() {
    let client = common::create_client();

    // First, list labels to get a valid label ID
    let labels_response: LabelsResponse = client
        .query(LABELS_QUERY, serde_json::json!({}))
        .expect("Should be able to list labels");

    // Skip if no labels exist
    if labels_response.issue_labels.nodes.is_empty() {
        println!("No labels found in workspace, skipping get test");
        return;
    }

    let first_label = &labels_response.issue_labels.nodes[0];
    let label_id = &first_label.id;

    println!("Testing get for label: {} ({})", first_label.name, label_id);

    // Get the label by ID
    let variables = serde_json::json!({
        "id": label_id
    });

    let response: LabelResponse = client
        .query(LABEL_QUERY, variables)
        .expect("Should be able to get label by ID");

    // Verify the label data matches
    assert_eq!(response.issue_label.id, *label_id, "Label ID should match");
    assert_eq!(
        response.issue_label.name, first_label.name,
        "Label name should match"
    );
    assert_eq!(
        response.issue_label.color, first_label.color,
        "Label color should match"
    );

    println!("Successfully retrieved label: {}", response.issue_label.name);
}
