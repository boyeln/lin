//! Integration tests for attachment-related functionality.
//!
//! These tests verify attachment operations against the real Linear API.

mod common;

use lin::api::queries::attachment::{
    ATTACHMENT_CREATE_MUTATION, ATTACHMENT_QUERY, FILE_UPLOAD_CREATE_MUTATION,
    ISSUE_ATTACHMENTS_QUERY,
};
use lin::api::queries::issue::{ISSUE_CREATE_MUTATION, ISSUES_QUERY};
use lin::api::queries::team::TEAMS_QUERY;
use lin::models::{
    AttachmentCreateResponse, AttachmentResponse, FileUploadResponse, IssueAttachmentsResponse,
    IssueCreateResponse, TeamsResponse,
};

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

/// Test uploading and getting an attachment.
///
/// This test verifies:
/// 1. Creating an issue
/// 2. Creating a test file
/// 3. Uploading the file as an attachment
/// 4. Getting the attachment by ID
/// 5. Listing attachments to verify it appears
/// 6. Cleanup (delete issue)
#[test]
#[ignore]
fn test_attachment_upload_and_get() {
    use std::fs;
    use std::io::Write;

    let client = common::create_client();

    // First, get a team to create the issue in
    let teams_response: TeamsResponse = client
        .query(TEAMS_QUERY, serde_json::json!({"first": 1}))
        .expect("Should be able to list teams");

    assert!(
        !teams_response.teams.nodes.is_empty(),
        "Need at least one team for attachment tests"
    );

    let team = &teams_response.teams.nodes[0];
    let team_id = &team.id;

    println!("Using team: {} ({})", team.name, team.key);

    // Generate a unique issue title
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let issue_title = format!(
        "{} Attachment Test {}",
        common::TEST_ISSUE_PREFIX,
        timestamp
    );

    // --- Step 1: Create Issue ---
    println!("Creating issue: {}", issue_title);

    let create_variables = serde_json::json!({
        "input": {
            "title": issue_title,
            "teamId": team_id,
            "description": "Test issue for attachment integration tests.",
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

    // Create a temporary test file
    let test_file_path = format!("/tmp/lin_test_{}.txt", timestamp);
    let test_content = format!("Test attachment content created at {}", timestamp);

    // Use a closure to ensure cleanup runs even if assertions fail
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // Write test file
        let mut file = fs::File::create(&test_file_path).expect("Should create test file");
        file.write_all(test_content.as_bytes())
            .expect("Should write test content");
        drop(file); // Explicitly close file

        println!("Created test file: {}", test_file_path);

        // --- Step 2: Request upload URL ---
        println!("Requesting upload URL");

        let content_type = "text/plain";
        let upload_variables = serde_json::json!({
            "contentType": content_type,
            "filename": format!("test_{}.txt", timestamp),
            "size": test_content.len() as i32
        });

        let upload_response: FileUploadResponse = client
            .query(FILE_UPLOAD_CREATE_MUTATION, upload_variables)
            .expect("Should be able to request upload URL");

        let upload_file = &upload_response.file_upload.upload_file;
        assert!(
            !upload_file.upload_url.is_empty(),
            "Upload URL should not be empty"
        );
        assert!(
            !upload_file.asset_url.is_empty(),
            "Asset URL should not be empty"
        );

        println!("Got upload URL: {}", upload_file.upload_url);

        // --- Step 3: Upload file to presigned URL ---
        println!("Uploading file to presigned URL");

        let http_client = reqwest::blocking::Client::new();
        let file_content = fs::read(&test_file_path).expect("Should read test file");

        let mut headers = reqwest::header::HeaderMap::new();

        // The content-type must match what was specified in the upload request
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static(content_type),
        );

        // Add all other headers from the upload response
        for header in &upload_file.headers {
            let key_lower = header.key.to_lowercase();
            if let Ok(value) = reqwest::header::HeaderValue::from_str(&header.value) {
                // Skip content-type since we already set it
                if key_lower != "content-type" {
                    if let Ok(name) = reqwest::header::HeaderName::try_from(key_lower.as_str()) {
                        headers.insert(name, value);
                    }
                }
            }
        }

        println!("Upload headers: {:?}", headers);

        let upload_result = http_client
            .put(&upload_file.upload_url)
            .headers(headers)
            .body(file_content)
            .send()
            .expect("Should upload file");

        if !upload_result.status().is_success() {
            let status = upload_result.status();
            let body = upload_result
                .text()
                .unwrap_or_else(|_| "Unable to read body".to_string());
            panic!("File upload failed with status {}: {}", status, body);
        }

        println!("File uploaded successfully");

        // --- Step 4: Create attachment record ---
        println!("Creating attachment record");

        let attachment_variables = serde_json::json!({
            "input": {
                "issueId": &issue_id,
                "title": format!("test_{}.txt", timestamp),
                "url": &upload_file.asset_url
            }
        });

        let attachment_response: AttachmentCreateResponse = client
            .query(ATTACHMENT_CREATE_MUTATION, attachment_variables)
            .expect("Should be able to create attachment");

        assert!(
            attachment_response.attachment_create.success,
            "Attachment creation should succeed"
        );

        let created_attachment = attachment_response
            .attachment_create
            .attachment
            .expect("Created attachment should be returned");

        let attachment_id = created_attachment.id.clone();
        println!("Created attachment: {}", attachment_id);

        assert_eq!(
            created_attachment.title,
            format!("test_{}.txt", timestamp),
            "Attachment title should match"
        );

        // --- Step 5: Get attachment by ID ---
        println!("Getting attachment by ID: {}", attachment_id);

        let get_variables = serde_json::json!({
            "id": &attachment_id
        });

        let get_response: AttachmentResponse = client
            .query(ATTACHMENT_QUERY, get_variables)
            .expect("Should be able to get attachment");

        assert_eq!(
            get_response.attachment.id, attachment_id,
            "Retrieved attachment ID should match"
        );
        assert_eq!(
            get_response.attachment.title,
            format!("test_{}.txt", timestamp),
            "Retrieved attachment title should match"
        );
        assert_eq!(
            get_response.attachment.url, upload_file.asset_url,
            "Retrieved attachment URL should match"
        );

        println!("Retrieved attachment successfully");

        // --- Step 6: List attachments to verify ---
        println!("Listing attachments for issue: {}", issue_identifier);

        let list_variables = serde_json::json!({
            "id": &issue_id
        });

        let list_response: IssueAttachmentsResponse = client
            .query(ISSUE_ATTACHMENTS_QUERY, list_variables)
            .expect("Should be able to list attachments");

        assert!(
            !list_response.issue.attachments.nodes.is_empty(),
            "Issue should have at least one attachment"
        );

        let found = list_response
            .issue
            .attachments
            .nodes
            .iter()
            .any(|a| a.id == attachment_id);

        assert!(found, "Attachment should appear in list");

        println!(
            "Found {} attachment(s) on issue",
            list_response.issue.attachments.nodes.len()
        );
    }));

    // Cleanup: Delete test file
    if std::path::Path::new(&test_file_path).exists() {
        fs::remove_file(&test_file_path).ok();
        println!("Deleted test file");
    }

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

/// Test getting an attachment that doesn't exist.
#[test]
#[ignore]
fn test_get_nonexistent_attachment() {
    let client = common::create_client();

    let variables = serde_json::json!({
        "id": "00000000-0000-0000-0000-000000000000"
    });

    let result: Result<AttachmentResponse, _> = client.query(ATTACHMENT_QUERY, variables);

    assert!(
        result.is_err(),
        "Should get an error for nonexistent attachment"
    );

    let error = result.unwrap_err();
    println!("Got expected error: {}", error);
}
