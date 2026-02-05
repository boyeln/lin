//! Integration tests for issue relation functionality.
//!
//! These tests verify relation operations against the real Linear API.
//! All tests clean up after themselves by deleting created issues.

mod common;

use lin::api::queries::issue::{
    ISSUE_CREATE_MUTATION, ISSUE_RELATIONS_QUERY, ISSUE_RELATION_CREATE_MUTATION,
    ISSUE_RELATION_DELETE_MUTATION,
};
use lin::api::queries::team::TEAMS_QUERY;
use lin::models::{
    IssueCreateResponse, IssueRelationCreateResponse, IssueRelationDeleteResponse,
    IssueRelationsResponse, TeamsResponse,
};

/// Test the relation lifecycle: create issues -> add relation -> list relations -> remove relation -> cleanup.
///
/// This test verifies:
/// 1. Issue creation works (for two issues)
/// 2. Adding a "blocks" relation between issues works
/// 3. Listing relations returns the added relation
/// 4. Removing the relation works
/// 5. Issue deletion cleans up (cleanup)
#[test]
#[ignore]
fn test_relation_lifecycle() {
    let client = common::create_client();

    // First, get a team to create the issues in
    let teams_response: TeamsResponse = client
        .query(TEAMS_QUERY, serde_json::json!({"first": 1}))
        .expect("Should be able to list teams");

    assert!(
        !teams_response.teams.nodes.is_empty(),
        "Need at least one team for relation tests"
    );

    let team = &teams_response.teams.nodes[0];
    let team_id = &team.id;

    println!("Using team: {} ({})", team.name, team.key);

    // Generate unique issue titles
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let blocking_title = format!("{} Blocking Issue {}", common::TEST_ISSUE_PREFIX, timestamp);
    let blocked_title = format!(
        "{} Blocked Issue {}",
        common::TEST_ISSUE_PREFIX,
        timestamp + 1
    );

    // --- Step 1: Create Blocking Issue ---
    println!("Creating blocking issue: {}", blocking_title);

    let create_blocking = serde_json::json!({
        "input": {
            "title": blocking_title,
            "teamId": team_id,
            "description": "This issue blocks another issue.",
            "priority": 4
        }
    });

    let blocking_response: IssueCreateResponse = client
        .query(ISSUE_CREATE_MUTATION, create_blocking)
        .expect("Should be able to create blocking issue");

    assert!(
        blocking_response.issue_create.success,
        "Blocking issue creation should succeed"
    );

    let blocking_issue = blocking_response
        .issue_create
        .issue
        .expect("Blocking issue should be returned");

    let blocking_id = blocking_issue.id.clone();
    let blocking_identifier = blocking_issue.identifier.clone();

    println!(
        "Created blocking issue: {} ({})",
        blocking_identifier, blocking_id
    );

    // --- Step 2: Create Blocked Issue ---
    println!("Creating blocked issue: {}", blocked_title);

    let create_blocked = serde_json::json!({
        "input": {
            "title": blocked_title,
            "teamId": team_id,
            "description": "This issue is blocked by another issue.",
            "priority": 4
        }
    });

    let blocked_response: IssueCreateResponse = client
        .query(ISSUE_CREATE_MUTATION, create_blocked)
        .expect("Should be able to create blocked issue");

    assert!(
        blocked_response.issue_create.success,
        "Blocked issue creation should succeed"
    );

    let blocked_issue = blocked_response
        .issue_create
        .issue
        .expect("Blocked issue should be returned");

    let blocked_id = blocked_issue.id.clone();
    let blocked_identifier = blocked_issue.identifier.clone();

    println!(
        "Created blocked issue: {} ({})",
        blocked_identifier, blocked_id
    );

    // Use a closure to ensure cleanup runs even if assertions fail
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // --- Step 3: Create a "blocks" relation ---
        println!(
            "Creating 'blocks' relation: {} blocks {}",
            blocking_identifier, blocked_identifier
        );

        let relation_variables = serde_json::json!({
            "input": {
                "issueId": &blocking_id,
                "relatedIssueId": &blocked_id,
                "type": "blocks"
            }
        });

        let relation_response: IssueRelationCreateResponse = client
            .query(ISSUE_RELATION_CREATE_MUTATION, relation_variables)
            .expect("Should be able to create relation");

        assert!(
            relation_response.issue_relation_create.success,
            "Relation creation should succeed"
        );

        let created_relation = relation_response
            .issue_relation_create
            .issue_relation
            .expect("Created relation should be returned");

        println!("Created relation: {}", created_relation.id);
        assert_eq!(
            created_relation.type_, "blocks",
            "Relation type should be 'blocks'"
        );

        let relation_id = created_relation.id.clone();

        // --- Step 4: List relations on the blocking issue ---
        println!(
            "Listing relations for blocking issue: {}",
            blocking_identifier
        );

        let list_variables = serde_json::json!({
            "id": &blocking_id
        });

        let list_response: IssueRelationsResponse = client
            .query(ISSUE_RELATIONS_QUERY, list_variables)
            .expect("Should be able to list relations");

        assert!(
            !list_response.issue.relations.nodes.is_empty(),
            "Blocking issue should have outgoing relations"
        );

        let found_relation = list_response
            .issue
            .relations
            .nodes
            .iter()
            .find(|r| r.id == relation_id);

        assert!(found_relation.is_some(), "Should find the created relation");
        assert_eq!(
            found_relation.unwrap().type_,
            "blocks",
            "Found relation should be 'blocks' type"
        );

        println!(
            "Found {} relation(s)",
            list_response.issue.relations.nodes.len()
        );

        // --- Step 5: Remove the relation ---
        println!("Removing relation: {}", relation_id);

        let delete_variables = serde_json::json!({
            "id": &relation_id
        });

        let delete_response: IssueRelationDeleteResponse = client
            .query(ISSUE_RELATION_DELETE_MUTATION, delete_variables)
            .expect("Should be able to delete relation");

        assert!(
            delete_response.issue_relation_delete.success,
            "Relation deletion should succeed"
        );

        println!("Relation deleted successfully");

        // Verify relation is gone
        let verify_response: IssueRelationsResponse = client
            .query(
                ISSUE_RELATIONS_QUERY,
                serde_json::json!({"id": &blocking_id}),
            )
            .expect("Should be able to list relations after deletion");

        let still_exists = verify_response
            .issue
            .relations
            .nodes
            .iter()
            .any(|r| r.id == relation_id);

        assert!(
            !still_exists,
            "Relation should no longer exist after deletion"
        );

        println!("Verified relation was removed");
    }));

    // --- Cleanup: Delete both issues ---
    println!(
        "Deleting blocking issue: {} ({})",
        blocking_identifier, blocking_id
    );
    let delete_blocking = common::delete_issue(&client, &blocking_id);
    match delete_blocking {
        Ok(success) => {
            assert!(success, "Blocking issue deletion should succeed");
            println!("Blocking issue deleted successfully");
        }
        Err(e) => {
            eprintln!("Warning: Failed to delete blocking test issue: {}", e);
            eprintln!(
                "Manual cleanup may be required for issue: {}",
                blocking_identifier
            );
        }
    }

    println!(
        "Deleting blocked issue: {} ({})",
        blocked_identifier, blocked_id
    );
    let delete_blocked = common::delete_issue(&client, &blocked_id);
    match delete_blocked {
        Ok(success) => {
            assert!(success, "Blocked issue deletion should succeed");
            println!("Blocked issue deleted successfully");
        }
        Err(e) => {
            eprintln!("Warning: Failed to delete blocked test issue: {}", e);
            eprintln!(
                "Manual cleanup may be required for issue: {}",
                blocked_identifier
            );
        }
    }

    // Re-throw any panic from the test assertions
    if let Err(e) = result {
        std::panic::resume_unwind(e);
    }
}
