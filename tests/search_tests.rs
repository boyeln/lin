//! Integration tests for search functionality.
//!
//! These tests verify the issue search feature against the real Linear API.

mod common;

use lin::api::queries::issue::ISSUE_CREATE_MUTATION;
use lin::api::queries::search::ISSUE_SEARCH_QUERY;
use lin::api::queries::team::TEAMS_QUERY;
use lin::models::{IssueCreateResponse, IssueSearchResponse, TeamsResponse};

/// Test searching for issues.
///
/// This test creates an issue with a unique title, searches for it,
/// and then deletes the issue (cleanup).
#[test]
#[ignore]
fn test_search_issues() {
    let client = common::create_client();

    // First, get a team to create the issue in
    let teams_response: TeamsResponse = client
        .query(TEAMS_QUERY, serde_json::json!({"first": 1}))
        .expect("Should be able to list teams");

    assert!(
        !teams_response.teams.nodes.is_empty(),
        "Need at least one team for search tests"
    );

    let team = &teams_response.teams.nodes[0];
    let team_id = &team.id;

    println!("Using team: {} ({})", team.name, team.key);

    // Generate a unique issue title with a searchable keyword
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let unique_keyword = format!("searchtest{}", timestamp);
    let issue_title = format!(
        "{} Search Test {}",
        common::TEST_ISSUE_PREFIX,
        unique_keyword
    );

    // Create the issue
    println!("Creating issue: {}", issue_title);

    let create_variables = serde_json::json!({
        "input": {
            "title": issue_title,
            "teamId": team_id,
            "description": format!("Test issue for search integration test. Unique keyword: {}", unique_keyword),
            "priority": 4  // Low priority
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
        // Wait a moment for the issue to be indexed
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Search for the issue using the unique keyword
        println!("Searching for: {}", unique_keyword);

        let search_variables = serde_json::json!({
            "first": 10,
            "filter": {
                "searchableContent": { "contains": &unique_keyword }
            }
        });

        let search_response: IssueSearchResponse = client
            .query(ISSUE_SEARCH_QUERY, search_variables)
            .expect("Should be able to search issues");

        println!(
            "Search returned {} result(s)",
            search_response.issues.nodes.len()
        );

        // The search should find our issue
        let found = search_response
            .issues
            .nodes
            .iter()
            .any(|issue| issue.id == issue_id);

        assert!(found, "Should find the created issue in search results");

        println!("Successfully found issue in search results");
    }));

    // Cleanup: Delete the issue
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
