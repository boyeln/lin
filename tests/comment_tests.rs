//! Integration tests for comment-related functionality.
//!
//! These tests verify comment operations against the real Linear API.
//! All tests clean up after themselves by deleting created issues.

mod common;

use lin::api::queries;
use lin::models::{
    CommentCreateResponse, IssueCommentsResponse, IssueCreateResponse, IssueWithCommentsResponse,
    TeamsResponse,
};

/// Test the comment lifecycle: create issue -> add comment -> list comments -> delete issue.
///
/// This test verifies:
/// 1. Issue creation works
/// 2. Adding a comment to the issue works
/// 3. Listing comments returns the added comment
/// 4. Fetching issue with comments includes the comment
/// 5. Issue deletion cleans up (cleanup)
#[test]
#[ignore]
fn test_comment_lifecycle() {
    let client = common::create_client();

    // First, get a team to create the issue in
    let teams_response: TeamsResponse = client
        .query(queries::TEAMS_QUERY, serde_json::json!({"first": 1}))
        .expect("Should be able to list teams");

    assert!(
        !teams_response.teams.nodes.is_empty(),
        "Need at least one team for comment tests"
    );

    let team = &teams_response.teams.nodes[0];
    let team_id = &team.id;

    println!("Using team: {} ({})", team.name, team.key);

    // Generate a unique issue title
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let issue_title = format!("{} Comment Test {}", common::TEST_ISSUE_PREFIX, timestamp);

    // --- Step 1: Create Issue ---
    println!("Creating issue: {}", issue_title);

    let create_variables = serde_json::json!({
        "input": {
            "title": issue_title,
            "teamId": team_id,
            "description": "Test issue for comment integration tests.",
            "priority": 4
        }
    });

    let create_response: IssueCreateResponse = client
        .query(queries::ISSUE_CREATE_MUTATION, create_variables)
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
        // --- Step 2: Add a Comment ---
        let comment_body = format!(
            "This is an automated test comment created at timestamp {}",
            timestamp
        );
        println!("Adding comment to issue: {}", issue_identifier);

        let comment_variables = serde_json::json!({
            "input": {
                "issueId": &issue_id,
                "body": &comment_body
            }
        });

        let comment_response: CommentCreateResponse = client
            .query(queries::COMMENT_CREATE_MUTATION, comment_variables)
            .expect("Should be able to create comment");

        assert!(
            comment_response.comment_create.success,
            "Comment creation should succeed"
        );

        let created_comment = comment_response
            .comment_create
            .comment
            .expect("Created comment should be returned");

        println!("Created comment: {}", created_comment.id);
        assert_eq!(
            created_comment.body, comment_body,
            "Comment body should match"
        );

        // --- Step 3: List Comments ---
        println!("Listing comments for issue: {}", issue_identifier);

        let list_variables = serde_json::json!({
            "id": &issue_id
        });

        let list_response: IssueCommentsResponse = client
            .query(queries::ISSUE_COMMENTS_QUERY, list_variables)
            .expect("Should be able to list comments");

        assert!(
            !list_response.issue.comments.nodes.is_empty(),
            "Should have at least one comment"
        );

        let found_comment = list_response
            .issue
            .comments
            .nodes
            .iter()
            .find(|c| c.id == created_comment.id);

        assert!(found_comment.is_some(), "Should find the created comment");
        assert_eq!(
            found_comment.unwrap().body,
            comment_body,
            "Found comment body should match"
        );

        println!(
            "Found {} comment(s)",
            list_response.issue.comments.nodes.len()
        );

        // --- Step 4: Fetch Issue with Comments ---
        println!(
            "Fetching issue with comments: {} ({})",
            issue_identifier, issue_id
        );

        let issue_with_comments_variables = serde_json::json!({
            "id": &issue_id
        });

        let issue_with_comments_response: IssueWithCommentsResponse = client
            .query(
                queries::ISSUE_WITH_COMMENTS_QUERY,
                issue_with_comments_variables,
            )
            .expect("Should be able to fetch issue with comments");

        assert_eq!(
            issue_with_comments_response.issue.id, issue_id,
            "Issue ID should match"
        );
        assert!(
            !issue_with_comments_response.issue.comments.nodes.is_empty(),
            "Issue should have comments"
        );

        println!(
            "Issue has {} comment(s)",
            issue_with_comments_response.issue.comments.nodes.len()
        );
    }));

    // --- Step 5: Delete Issue (Cleanup) ---
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

/// Test adding multiple comments to an issue.
#[test]
#[ignore]
fn test_multiple_comments() {
    let client = common::create_client();

    // Get a team
    let teams_response: TeamsResponse = client
        .query(queries::TEAMS_QUERY, serde_json::json!({"first": 1}))
        .expect("Should be able to list teams");

    let team = &teams_response.teams.nodes[0];

    // Create issue
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let create_variables = serde_json::json!({
        "input": {
            "title": format!("{} Multi-Comment Test {}", common::TEST_ISSUE_PREFIX, timestamp),
            "teamId": &team.id,
            "priority": 4
        }
    });

    let create_response: IssueCreateResponse = client
        .query(queries::ISSUE_CREATE_MUTATION, create_variables)
        .expect("Should be able to create issue");

    let issue = create_response
        .issue_create
        .issue
        .expect("Issue should be created");

    let issue_id = issue.id.clone();
    println!("Created issue: {}", issue.identifier);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // Add multiple comments
        for i in 1..=3 {
            let comment_variables = serde_json::json!({
                "input": {
                    "issueId": &issue_id,
                    "body": format!("Test comment #{}", i)
                }
            });

            let response: CommentCreateResponse = client
                .query(queries::COMMENT_CREATE_MUTATION, comment_variables)
                .expect("Should be able to create comment");

            assert!(response.comment_create.success);
            println!("Added comment #{}", i);
        }

        // Verify all comments are present
        let list_variables = serde_json::json!({
            "id": &issue_id
        });

        let list_response: IssueCommentsResponse = client
            .query(queries::ISSUE_COMMENTS_QUERY, list_variables)
            .expect("Should be able to list comments");

        assert!(
            list_response.issue.comments.nodes.len() >= 3,
            "Should have at least 3 comments"
        );

        println!(
            "Issue has {} comments",
            list_response.issue.comments.nodes.len()
        );
    }));

    // Cleanup
    let _ = common::delete_issue(&client, &issue_id);
    println!("Deleted issue: {}", issue.identifier);

    if let Err(e) = result {
        std::panic::resume_unwind(e);
    }
}
