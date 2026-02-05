//! Integration tests for git-related functionality.
//!
//! These tests verify git linking operations against the real Linear API.

mod common;

use lin::api::queries::attachment::{ATTACHMENT_CREATE_MUTATION, ISSUE_GIT_LINKS_QUERY};
use lin::api::queries::issue::{ISSUE_CREATE_MUTATION, ISSUES_QUERY};
use lin::api::queries::team::TEAMS_QUERY;
use lin::models::{
    AttachmentCreateResponse, IssueAttachmentsResponse, IssueCreateResponse, TeamsResponse,
};

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

/// Test linking a branch to an issue.
///
/// This test verifies:
/// 1. Creating an issue
/// 2. Linking a branch to the issue
/// 3. Listing git links to verify the branch appears
/// 4. Cleanup (delete issue)
#[test]
#[ignore]
fn test_link_branch() {
    let client = common::create_client();

    // First, get a team to create the issue in
    let teams_response: TeamsResponse = client
        .query(TEAMS_QUERY, serde_json::json!({"first": 1}))
        .expect("Should be able to list teams");

    assert!(
        !teams_response.teams.nodes.is_empty(),
        "Need at least one team for git link tests"
    );

    let team = &teams_response.teams.nodes[0];
    let team_id = &team.id;

    println!("Using team: {} ({})", team.name, team.key);

    // Generate a unique issue title
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let issue_title = format!("{} Git Branch Test {}", common::TEST_ISSUE_PREFIX, timestamp);

    // --- Step 1: Create Issue ---
    println!("Creating issue: {}", issue_title);

    let create_variables = serde_json::json!({
        "input": {
            "title": issue_title,
            "teamId": team_id,
            "description": "Test issue for git branch linking.",
            "priority": 4
        }
    });

    let create_response: IssueCreateResponse = client
        .query(ISSUE_CREATE_MUTATION, create_variables)
        .expect("Should be able to create issue");

    assert!(
        create_response.issue_create.success,
        "Issue creation should succeed"
    );

    let created_issue = create_response
        .issue_create
        .issue
        .expect("Created issue should be returned");

    let issue_id = created_issue.id.clone();
    let issue_identifier = created_issue.identifier.clone();

    println!("Created issue: {} ({})", issue_identifier, issue_id);

    // Use a closure to ensure cleanup runs even if assertions fail
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // --- Step 2: Link a branch ---
        let branch_name = format!("feature/test-{}", timestamp);
        let repo_url = "https://github.com/test-org/test-repo";
        let expected_url = format!("{}/tree/{}", repo_url, branch_name);

        println!("Linking branch: {}", branch_name);

        let link_variables = serde_json::json!({
            "input": {
                "issueId": &issue_id,
                "title": &branch_name,
                "url": &expected_url,
                "subtitle": "Git branch"
            }
        });

        let link_response: AttachmentCreateResponse = client
            .query(ATTACHMENT_CREATE_MUTATION, link_variables)
            .expect("Should be able to link branch");

        assert!(
            link_response.attachment_create.success,
            "Branch link creation should succeed"
        );

        let created_link = link_response
            .attachment_create
            .attachment
            .expect("Created link should be returned");

        println!("Created branch link: {}", created_link.id);
        assert_eq!(created_link.title, branch_name, "Branch name should match");
        assert_eq!(created_link.url, expected_url, "Branch URL should match");

        // --- Step 3: List git links to verify ---
        println!("Listing git links for issue: {}", issue_identifier);

        let list_variables = serde_json::json!({
            "id": &issue_id
        });

        let list_response: IssueAttachmentsResponse = client
            .query(ISSUE_GIT_LINKS_QUERY, list_variables)
            .expect("Should be able to list git links");

        assert!(
            !list_response.issue.attachments.nodes.is_empty(),
            "Issue should have at least one attachment"
        );

        let found = list_response
            .issue
            .attachments
            .nodes
            .iter()
            .any(|a| a.id == created_link.id && a.url.contains("/tree/"));

        assert!(found, "Branch link should appear in list");

        println!(
            "Found {} attachment(s) on issue",
            list_response.issue.attachments.nodes.len()
        );
    }));

    // Cleanup: Delete issue
    println!("Deleting issue: {} ({})", issue_identifier, issue_id);

    let delete_result = common::delete_issue(&client, &issue_id);

    match delete_result {
        Ok(success) => {
            assert!(success, "Issue deletion should succeed");
            println!("Issue deleted successfully");
        }
        Err(e) => {
            eprintln!("Warning: Failed to delete test issue: {}", e);
            eprintln!(
                "Manual cleanup may be required for issue: {}",
                issue_identifier
            );
        }
    }

    // Re-throw any panic from the test assertions
    if let Err(e) = result {
        std::panic::resume_unwind(e);
    }
}

/// Test linking a pull request to an issue.
///
/// This test verifies:
/// 1. Creating an issue
/// 2. Linking a PR to the issue
/// 3. Listing git links to verify the PR appears
/// 4. Cleanup (delete issue)
#[test]
#[ignore]
fn test_link_pr() {
    let client = common::create_client();

    // First, get a team to create the issue in
    let teams_response: TeamsResponse = client
        .query(TEAMS_QUERY, serde_json::json!({"first": 1}))
        .expect("Should be able to list teams");

    assert!(
        !teams_response.teams.nodes.is_empty(),
        "Need at least one team for git link tests"
    );

    let team = &teams_response.teams.nodes[0];
    let team_id = &team.id;

    println!("Using team: {} ({})", team.name, team.key);

    // Generate a unique issue title
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let issue_title = format!("{} Git PR Test {}", common::TEST_ISSUE_PREFIX, timestamp);

    // --- Step 1: Create Issue ---
    println!("Creating issue: {}", issue_title);

    let create_variables = serde_json::json!({
        "input": {
            "title": issue_title,
            "teamId": team_id,
            "description": "Test issue for PR linking.",
            "priority": 4
        }
    });

    let create_response: IssueCreateResponse = client
        .query(ISSUE_CREATE_MUTATION, create_variables)
        .expect("Should be able to create issue");

    assert!(
        create_response.issue_create.success,
        "Issue creation should succeed"
    );

    let created_issue = create_response
        .issue_create
        .issue
        .expect("Created issue should be returned");

    let issue_id = created_issue.id.clone();
    let issue_identifier = created_issue.identifier.clone();

    println!("Created issue: {} ({})", issue_identifier, issue_id);

    // Use a closure to ensure cleanup runs even if assertions fail
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // --- Step 2: Link a PR ---
        let pr_url = format!("https://github.com/test-org/test-repo/pull/{}", timestamp % 1000);
        let pr_title = format!("PR #{}", timestamp % 1000);

        println!("Linking PR: {}", pr_url);

        let link_variables = serde_json::json!({
            "input": {
                "issueId": &issue_id,
                "title": &pr_title,
                "url": &pr_url,
                "subtitle": "Pull Request"
            }
        });

        let link_response: AttachmentCreateResponse = client
            .query(ATTACHMENT_CREATE_MUTATION, link_variables)
            .expect("Should be able to link PR");

        assert!(
            link_response.attachment_create.success,
            "PR link creation should succeed"
        );

        let created_link = link_response
            .attachment_create
            .attachment
            .expect("Created link should be returned");

        println!("Created PR link: {}", created_link.id);
        assert_eq!(created_link.title, pr_title, "PR title should match");
        assert_eq!(created_link.url, pr_url, "PR URL should match");

        // --- Step 3: List git links to verify ---
        println!("Listing git links for issue: {}", issue_identifier);

        let list_variables = serde_json::json!({
            "id": &issue_id
        });

        let list_response: IssueAttachmentsResponse = client
            .query(ISSUE_GIT_LINKS_QUERY, list_variables)
            .expect("Should be able to list git links");

        assert!(
            !list_response.issue.attachments.nodes.is_empty(),
            "Issue should have at least one attachment"
        );

        let found = list_response
            .issue
            .attachments
            .nodes
            .iter()
            .any(|a| a.id == created_link.id && a.url.contains("/pull/"));

        assert!(found, "PR link should appear in list");

        println!(
            "Found {} attachment(s) on issue",
            list_response.issue.attachments.nodes.len()
        );
    }));

    // Cleanup: Delete issue
    println!("Deleting issue: {} ({})", issue_identifier, issue_id);

    let delete_result = common::delete_issue(&client, &issue_id);

    match delete_result {
        Ok(success) => {
            assert!(success, "Issue deletion should succeed");
            println!("Issue deleted successfully");
        }
        Err(e) => {
            eprintln!("Warning: Failed to delete test issue: {}", e);
            eprintln!(
                "Manual cleanup may be required for issue: {}",
                issue_identifier
            );
        }
    }

    // Re-throw any panic from the test assertions
    if let Err(e) = result {
        std::panic::resume_unwind(e);
    }
}
