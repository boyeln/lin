//! Integration tests for document-related functionality.
//!
//! These tests verify document operations against the real Linear API.

mod common;

use lin::api::queries;
use lin::models::DocumentsResponse;

/// Test that we can list documents in the organization.
///
/// This tests read operations and verifies:
/// 1. We can fetch documents
/// 2. The response parsing works for collections
#[test]
#[ignore]
fn test_document_list() {
    let client = common::create_client();

    let variables = serde_json::json!({});

    let response: DocumentsResponse = client
        .query(queries::DOCUMENTS_QUERY, variables)
        .expect("Should be able to list documents");

    // Documents might be empty for new workspaces, which is valid
    println!("Found {} document(s)", response.documents.nodes.len());

    // If there are documents, verify the data is valid
    if !response.documents.nodes.is_empty() {
        let first_document = &response.documents.nodes[0];
        assert!(
            !first_document.id.is_empty(),
            "Document ID should not be empty"
        );
        assert!(
            !first_document.title.is_empty(),
            "Document title should not be empty"
        );

        println!("First document: {}", first_document.title);
    }
}

/// Test that we can create and retrieve a document.
///
/// This tests write operations and verifies:
/// 1. We can create a document
/// 2. We can retrieve the created document with content
#[test]
#[ignore]
fn test_document_create_and_get() {
    use lin::models::{DocumentCreateResponse, DocumentResponse};
    use std::time::{SystemTime, UNIX_EPOCH};

    let client = common::create_client();

    // Create a test document with a unique title
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let test_title = format!("[lin-test] Test Document {}", timestamp);
    let test_content = "Test document created by lin integration tests.";

    let create_variables = serde_json::json!({
        "input": {
            "title": test_title,
            "content": test_content
        }
    });

    let create_response: DocumentCreateResponse = client
        .query(queries::DOCUMENT_CREATE_MUTATION, create_variables)
        .expect("Should be able to create a document");

    assert!(
        create_response.document_create.success,
        "Document creation should succeed"
    );
    let created_doc = create_response
        .document_create
        .document
        .expect("Should return created document");
    assert_eq!(created_doc.title, test_title);
    assert_eq!(created_doc.content, Some(test_content.to_string()));

    println!(
        "Created document: {} (ID: {})",
        created_doc.title, created_doc.id
    );

    // Retrieve the document to verify it was created
    let get_variables = serde_json::json!({
        "id": created_doc.id
    });

    let get_response: DocumentResponse = client
        .query(queries::DOCUMENT_QUERY, get_variables)
        .expect("Should be able to get the document");

    assert_eq!(get_response.document.id, created_doc.id);
    assert_eq!(get_response.document.title, test_title);
    assert_eq!(
        get_response.document.content,
        Some(test_content.to_string())
    );

    println!("Retrieved document: {}", get_response.document.title);

    // Note: Linear's API doesn't have a documentDelete mutation that's publicly available,
    // so we leave the test document in place. It can be cleaned up manually.
}
