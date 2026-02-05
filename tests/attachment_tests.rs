//! Integration tests for attachment-related functionality.
//!
//! These tests verify attachment operations against the real Linear API.

mod common;

use lin::api::queries::attachment::ISSUE_ATTACHMENTS_QUERY;
use lin::api::queries::issue::ISSUES_QUERY;
use lin::models::IssueAttachmentsResponse;

/// Test that we can list attachments on an issue.
///
/// This tests read operations and verifies:
/// 1. We can fetch attachments for an issue
/// 2. The response parsing works for attachment collections
#[test]
#[ignore]
fn test_attachment_list() {
    use lin::models::IssuesResponse;

    let client = common::create_client();

    // First, get an issue to list attachments for
    let issues_variables = serde_json::json!({
        "first": 1
    });

    let issues_response: IssuesResponse = client
        .query(ISSUES_QUERY, issues_variables)
        .expect("Should be able to list issues");

    if issues_response.issues.nodes.is_empty() {
        println!("No issues found in workspace, skipping attachment list test");
        return;
    }

    let issue = &issues_response.issues.nodes[0];
    println!("Testing attachments for issue: {}", issue.identifier);

    // Now list attachments for that issue
    let variables = serde_json::json!({
        "id": issue.id
    });

    let response: IssueAttachmentsResponse = client
        .query(ISSUE_ATTACHMENTS_QUERY, variables)
        .expect("Should be able to list attachments");

    println!(
        "Found {} attachment(s) on issue {}",
        response.issue.attachments.nodes.len(),
        response.issue.identifier
    );

    // Attachments might be empty, which is valid
    // If there are attachments, verify the data is valid
    if !response.issue.attachments.nodes.is_empty() {
        let first_attachment = &response.issue.attachments.nodes[0];
        assert!(
            !first_attachment.id.is_empty(),
            "Attachment ID should not be empty"
        );
        assert!(
            !first_attachment.title.is_empty(),
            "Attachment title should not be empty"
        );
        assert!(
            !first_attachment.url.is_empty(),
            "Attachment URL should not be empty"
        );

        println!(
            "First attachment: {} ({})",
            first_attachment.title, first_attachment.url
        );
    }
}
