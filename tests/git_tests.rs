//! Integration tests for git-related functionality.
//!
//! These tests verify git linking operations against the real Linear API.

mod common;

use lin::api::queries::attachment::ISSUE_GIT_LINKS_QUERY;
use lin::api::queries::issue::ISSUES_QUERY;
use lin::models::IssueAttachmentsResponse;

/// Test that we can list git links (attachments) on an issue.
///
/// This tests read operations and verifies:
/// 1. We can fetch attachments for an issue
/// 2. The response parsing works for attachment collections
/// 3. Git links are correctly identified and filtered
#[test]
#[ignore]
fn test_git_links_list() {
    use lin::models::IssuesResponse;

    let client = common::create_client();

    // First, get an issue to list git links for
    let issues_variables = serde_json::json!({
        "first": 1
    });

    let issues_response: IssuesResponse = client
        .query(ISSUES_QUERY, issues_variables)
        .expect("Should be able to list issues");

    if issues_response.issues.nodes.is_empty() {
        println!("No issues found in workspace, skipping git links list test");
        return;
    }

    let issue = &issues_response.issues.nodes[0];
    println!("Testing git links for issue: {}", issue.identifier);

    // Now list attachments for that issue (git links are stored as attachments)
    let variables = serde_json::json!({
        "id": issue.id
    });

    let response: IssueAttachmentsResponse = client
        .query(ISSUE_GIT_LINKS_QUERY, variables)
        .expect("Should be able to list attachments/git links");

    println!(
        "Found {} attachment(s) on issue {}",
        response.issue.attachments.nodes.len(),
        response.issue.identifier
    );

    // Filter to show only git-related links
    let git_links: Vec<_> = response
        .issue
        .attachments
        .nodes
        .iter()
        .filter(|a| {
            let url = a.url.to_lowercase();
            url.contains("github.com")
                || url.contains("gitlab.com")
                || url.contains("bitbucket.org")
                || url.contains("/tree/")
                || url.contains("/pull/")
                || url.contains("/merge_requests/")
                || a.subtitle.as_deref() == Some("Git branch")
                || a.subtitle.as_deref() == Some("Pull Request")
        })
        .collect();

    println!("Found {} git link(s)", git_links.len());

    // Git links might be empty, which is valid
    // If there are git links, verify the data is valid
    for link in &git_links {
        assert!(!link.id.is_empty(), "Git link ID should not be empty");
        assert!(!link.title.is_empty(), "Git link title should not be empty");
        assert!(!link.url.is_empty(), "Git link URL should not be empty");

        println!("Git link: {} -> {}", link.title, link.url);
    }
}
