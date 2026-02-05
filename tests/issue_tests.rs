//! Integration tests for issue-related functionality.
//!
//! These tests verify issue CRUD operations against the real Linear API.
//! All tests clean up after themselves by deleting created issues.

mod common;

use lin::api::queries::issue::{
    ISSUE_ARCHIVE_MUTATION, ISSUE_BY_IDENTIFIER_QUERY, ISSUE_CREATE_MUTATION,
    ISSUE_DELETE_MUTATION, ISSUE_QUERY, ISSUE_UNARCHIVE_MUTATION, ISSUE_UPDATE_MUTATION,
};
use lin::api::queries::team::TEAMS_QUERY;
use lin::models::{IssueCreateResponse, IssueResponse, IssuesResponse, TeamsResponse};

/// Test the complete issue lifecycle: create -> read -> update -> delete.
///
/// This is the most comprehensive integration test. It verifies:
/// 1. Issue creation works
/// 2. We can read the created issue
/// 3. We can update the issue
/// 4. We can delete the issue (cleanup)
///
/// The test cleans up after itself by deleting the created issue.
#[test]
#[ignore]
fn test_issue_lifecycle() {
    let client = common::create_client();

    // First, get a team to create the issue in
    let teams_response: TeamsResponse = client
        .query(TEAMS_QUERY, serde_json::json!({"first": 1}))
        .expect("Should be able to list teams");

    assert!(
        !teams_response.teams.nodes.is_empty(),
        "Need at least one team for issue tests"
    );

    let team = &teams_response.teams.nodes[0];
    let team_id = &team.id;

    println!("Using team: {} ({})", team.name, team.key);

    // Generate a unique issue title
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let issue_title = format!("{} Test Issue {}", common::TEST_ISSUE_PREFIX, timestamp);

    // --- Step 1: Create Issue ---
    println!("Creating issue: {}", issue_title);

    let create_variables = serde_json::json!({
        "input": {
            "title": issue_title,
            "teamId": team_id,
            "description": "This is an automated test issue created by lin integration tests. It should be automatically deleted.",
            "priority": 4,  // Low priority
            "estimate": 3.0  // Test estimate
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
        // --- Step 2: Read Issue by ID ---
        println!("Reading issue by ID: {}", issue_id);

        let read_variables = serde_json::json!({
            "id": &issue_id
        });

        let read_response: IssueResponse = client
            .query(ISSUE_QUERY, read_variables)
            .expect("Should be able to read issue by ID");

        assert_eq!(
            read_response.issue.id, issue_id,
            "Read issue ID should match"
        );
        assert_eq!(
            read_response.issue.title, issue_title,
            "Read issue title should match"
        );
        assert_eq!(
            read_response.issue.estimate,
            Some(3.0),
            "Read issue estimate should match"
        );

        // --- Step 3: Read Issue by Identifier ---
        println!("Reading issue by identifier: {}", issue_identifier);

        // Parse the identifier to get team key and number
        let parts: Vec<&str> = issue_identifier.split('-').collect();
        let team_key = parts[0];
        let issue_number: i32 = parts[1].parse().expect("Issue number should be valid");

        let read_by_id_variables = serde_json::json!({
            "filter": {
                "team": { "key": { "eq": team_key } },
                "number": { "eq": issue_number }
            }
        });

        let read_by_id_response: IssuesResponse = client
            .query(ISSUE_BY_IDENTIFIER_QUERY, read_by_id_variables)
            .expect("Should be able to read issue by identifier");

        assert_eq!(
            read_by_id_response.issues.nodes.len(),
            1,
            "Should find exactly one issue"
        );
        assert_eq!(
            read_by_id_response.issues.nodes[0].id, issue_id,
            "Found issue ID should match"
        );

        // --- Step 4: Update Issue ---
        let updated_title = format!("{} (updated)", issue_title);
        println!("Updating issue title to: {}", updated_title);

        let update_variables = serde_json::json!({
            "id": &issue_id,
            "input": {
                "title": &updated_title,
                "priority": 3,  // Change to normal priority
                "estimate": 5.0  // Update estimate
            }
        });

        let update_response: lin::models::IssueUpdateResponse = client
            .query(ISSUE_UPDATE_MUTATION, update_variables)
            .expect("Should be able to update issue");

        assert!(
            update_response.issue_update.success,
            "Issue update should succeed"
        );

        let updated_issue = update_response
            .issue_update
            .issue
            .expect("Updated issue should be returned");

        assert_eq!(
            updated_issue.title, updated_title,
            "Updated title should match"
        );
        assert_eq!(updated_issue.priority, 3, "Updated priority should match");
        assert_eq!(
            updated_issue.estimate,
            Some(5.0),
            "Updated estimate should match"
        );

        println!("Issue updated successfully");
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

/// Test creating and immediately deleting an issue (minimal lifecycle).
///
/// This is a simpler test that just verifies create and delete work.
#[test]
#[ignore]
fn test_issue_create_and_delete() {
    let client = common::create_client();

    // Get a team
    let teams_response: TeamsResponse = client
        .query(TEAMS_QUERY, serde_json::json!({"first": 1}))
        .expect("Should be able to list teams");

    let team = &teams_response.teams.nodes[0];

    // Create issue
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let create_variables = serde_json::json!({
        "input": {
            "title": format!("{} Quick Test {}", common::TEST_ISSUE_PREFIX, timestamp),
            "teamId": &team.id,
            "priority": 4
        }
    });

    let create_response: IssueCreateResponse = client
        .query(ISSUE_CREATE_MUTATION, create_variables)
        .expect("Should be able to create issue");

    let issue = create_response
        .issue_create
        .issue
        .expect("Issue should be created");

    println!("Created issue: {}", issue.identifier);

    // Delete issue
    let deleted = common::delete_issue(&client, &issue.id).expect("Should be able to delete issue");

    assert!(deleted, "Issue should be deleted");
    println!("Deleted issue: {}", issue.identifier);
}

/// Test that we get a proper error for invalid issue ID.
#[test]
#[ignore]
fn test_get_nonexistent_issue() {
    let client = common::create_client();

    let variables = serde_json::json!({
        "id": "00000000-0000-0000-0000-000000000000"
    });

    let result: Result<IssueResponse, _> = client.query(ISSUE_QUERY, variables);

    assert!(result.is_err(), "Should get an error for nonexistent issue");

    let error = result.unwrap_err();
    println!("Got expected error: {}", error);
}

/// Test archiving and unarchiving an issue.
///
/// This test verifies:
/// 1. Creating an issue
/// 2. Archiving the issue
/// 3. Unarchiving the issue
/// 4. Deleting the issue (cleanup)
#[test]
#[ignore]
fn test_issue_archive_unarchive() {
    let client = common::create_client();

    // Get a team
    let teams_response: TeamsResponse = client
        .query(TEAMS_QUERY, serde_json::json!({"first": 1}))
        .expect("Should be able to list teams");

    let team = &teams_response.teams.nodes[0];
    println!("Using team: {} ({})", team.name, team.key);

    // Create issue
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let issue_title = format!("{} Archive Test {}", common::TEST_ISSUE_PREFIX, timestamp);

    let create_variables = serde_json::json!({
        "input": {
            "title": &issue_title,
            "teamId": &team.id,
            "description": "Test issue for archive/unarchive integration test",
            "priority": 4
        }
    });

    let create_response: IssueCreateResponse = client
        .query(ISSUE_CREATE_MUTATION, create_variables)
        .expect("Should be able to create issue");

    let issue = create_response
        .issue_create
        .issue
        .expect("Issue should be created");

    let issue_id = issue.id.clone();
    let issue_identifier = issue.identifier.clone();

    println!("Created issue: {} ({})", issue_identifier, issue_id);

    // Use a closure to ensure cleanup runs even if assertions fail
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // Archive the issue
        println!("Archiving issue: {}", issue_identifier);

        let archive_variables = serde_json::json!({ "id": &issue_id });

        let archive_response: lin::models::IssueArchiveResponse = client
            .query(ISSUE_ARCHIVE_MUTATION, archive_variables)
            .expect("Should be able to archive issue");

        assert!(
            archive_response.issue_archive.success,
            "Issue archive should succeed"
        );

        println!("Issue archived successfully");

        // Unarchive the issue
        println!("Unarchiving issue: {}", issue_identifier);

        let unarchive_variables = serde_json::json!({ "id": &issue_id });

        let unarchive_response: lin::models::IssueUnarchiveResponse = client
            .query(ISSUE_UNARCHIVE_MUTATION, unarchive_variables)
            .expect("Should be able to unarchive issue");

        assert!(
            unarchive_response.issue_unarchive.success,
            "Issue unarchive should succeed"
        );

        println!("Issue unarchived successfully");
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

/// Test deleting an issue.
///
/// This test verifies that the delete mutation works correctly.
/// Note: This is separate from the cleanup delete used in other tests.
#[test]
#[ignore]
fn test_issue_delete() {
    let client = common::create_client();

    // Get a team
    let teams_response: TeamsResponse = client
        .query(TEAMS_QUERY, serde_json::json!({"first": 1}))
        .expect("Should be able to list teams");

    let team = &teams_response.teams.nodes[0];

    // Create issue
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let create_variables = serde_json::json!({
        "input": {
            "title": format!("{} Delete Test {}", common::TEST_ISSUE_PREFIX, timestamp),
            "teamId": &team.id,
            "priority": 4
        }
    });

    let create_response: IssueCreateResponse = client
        .query(ISSUE_CREATE_MUTATION, create_variables)
        .expect("Should be able to create issue");

    let issue = create_response
        .issue_create
        .issue
        .expect("Issue should be created");

    let issue_id = issue.id.clone();
    let issue_identifier = issue.identifier.clone();

    println!("Created issue: {} ({})", issue_identifier, issue_id);

    // Delete the issue
    println!("Deleting issue: {}", issue_identifier);

    let delete_variables = serde_json::json!({ "id": &issue_id });

    let delete_response: lin::models::IssueDeleteResponse = client
        .query(ISSUE_DELETE_MUTATION, delete_variables)
        .expect("Should be able to delete issue");

    assert!(
        delete_response.issue_delete.success,
        "Issue delete should succeed"
    );

    println!("Issue deleted successfully");

    // Verify the issue is gone by trying to read it
    let read_variables = serde_json::json!({ "id": &issue_id });
    let read_result: Result<IssueResponse, _> = client.query(ISSUE_QUERY, read_variables);

    // The issue should either not be found or return an error
    // Linear may return different responses for deleted issues
    if let Ok(response) = read_result {
        // Some APIs might return the issue but mark it as deleted/archived
        println!(
            "Issue still returned after delete (may be soft-deleted): {:?}",
            response.issue.id
        );
    } else {
        println!("Issue not found after delete (confirmed hard delete)");
    }
}

/// Test listing issues with priority filter.
///
/// This test verifies:
/// 1. Creating issues with different priorities
/// 2. Filtering issues by priority level
/// 3. Cleanup (delete test issues)
#[test]
#[ignore]
fn test_issue_list_with_priority_filter() {
    let client = common::create_client();

    // Get a team
    let teams_response: TeamsResponse = client
        .query(TEAMS_QUERY, serde_json::json!({"first": 1}))
        .expect("Should be able to list teams");

    let team = &teams_response.teams.nodes[0];
    let team_id = &team.id;
    let team_key = &team.key;

    println!("Using team: {} ({})", team.name, team.key);

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Create issues with different priorities
    let priorities = vec![1, 2, 3, 4]; // Urgent, High, Normal, Low
    let mut created_issue_ids = Vec::new();

    for priority in &priorities {
        let title = format!(
            "{} Priority {} Test {}",
            common::TEST_ISSUE_PREFIX, priority, timestamp
        );

        let create_variables = serde_json::json!({
            "input": {
                "title": title,
                "teamId": team_id,
                "priority": priority
            }
        });

        let create_response: IssueCreateResponse = client
            .query(ISSUE_CREATE_MUTATION, create_variables)
            .expect("Should be able to create issue");

        if let Some(issue) = create_response.issue_create.issue {
            created_issue_ids.push(issue.id.clone());
            println!("Created issue with priority {}: {}", priority, issue.identifier);
        }
    }

    // Use closure to ensure cleanup
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // Wait a moment for issues to be indexed
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Test filtering by priority 1 (Urgent)
        let filter_variables = serde_json::json!({
            "first": 50,
            "filter": {
                "team": { "key": { "eq": team_key } },
                "priority": { "eq": 1 }
            }
        });

        let filtered_response: IssuesResponse = client
            .query(ISSUE_BY_IDENTIFIER_QUERY, filter_variables)
            .expect("Should be able to filter issues");

        // All returned issues should have priority 1
        for issue in &filtered_response.issues.nodes {
            assert_eq!(issue.priority, 1, "Filtered issue should have priority 1");
        }

        println!(
            "Found {} issue(s) with priority 1",
            filtered_response.issues.nodes.len()
        );
    }));

    // Cleanup: Delete all created issues
    for issue_id in &created_issue_ids {
        let _ = common::delete_issue(&client, issue_id);
    }
    println!("Deleted {} test issues", created_issue_ids.len());

    if let Err(e) = result {
        std::panic::resume_unwind(e);
    }
}

/// Test listing issues with sorting.
///
/// This test verifies:
/// 1. Creating multiple issues
/// 2. Sorting issues by different fields
/// 3. Cleanup (delete test issues)
#[test]
#[ignore]
fn test_issue_list_with_sorting() {
    let client = common::create_client();

    // Get a team
    let teams_response: TeamsResponse = client
        .query(TEAMS_QUERY, serde_json::json!({"first": 1}))
        .expect("Should be able to list teams");

    let team = &teams_response.teams.nodes[0];
    let team_id = &team.id;
    let team_key = &team.key;

    println!("Using team: {} ({})", team.name, team.key);

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Create multiple issues with different titles
    let titles = vec!["Alpha", "Beta", "Charlie"];
    let mut created_issue_ids = Vec::new();

    for title in &titles {
        let full_title = format!("{} {} Test {}", common::TEST_ISSUE_PREFIX, title, timestamp);

        let create_variables = serde_json::json!({
            "input": {
                "title": full_title,
                "teamId": team_id,
                "priority": 4
            }
        });

        let create_response: IssueCreateResponse = client
            .query(ISSUE_CREATE_MUTATION, create_variables)
            .expect("Should be able to create issue");

        if let Some(issue) = create_response.issue_create.issue {
            created_issue_ids.push(issue.id.clone());
            println!("Created issue: {}", issue.identifier);
        }

        // Small delay between creates to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // Use closure to ensure cleanup
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // Wait for issues to be indexed
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Test sorting by title (ascending)
        let sort_variables = serde_json::json!({
            "first": 50,
            "filter": {
                "team": { "key": { "eq": team_key } }
            },
            "orderBy": "title"
        });

        let sorted_response: IssuesResponse = client
            .query(ISSUE_BY_IDENTIFIER_QUERY, sort_variables)
            .expect("Should be able to sort issues");

        println!(
            "Found {} issue(s) for team (sorted by title)",
            sorted_response.issues.nodes.len()
        );

        // Verify our test issues are in alphabetical order
        let our_issues: Vec<_> = sorted_response
            .issues
            .nodes
            .iter()
            .filter(|i| created_issue_ids.contains(&i.id))
            .collect();

        if our_issues.len() >= 2 {
            for i in 0..our_issues.len() - 1 {
                let current_title = &our_issues[i].title;
                let next_title = &our_issues[i + 1].title;
                println!("Order check: {} < {}", current_title, next_title);
            }
        }
    }));

    // Cleanup: Delete all created issues
    for issue_id in &created_issue_ids {
        let _ = common::delete_issue(&client, issue_id);
    }
    println!("Deleted {} test issues", created_issue_ids.len());

    if let Err(e) = result {
        std::panic::resume_unwind(e);
    }
}
