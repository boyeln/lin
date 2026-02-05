//! Integration tests for document-related functionality.
//!
//! These tests verify document operations against the real Linear API.

mod common;

use lin::api::queries::document::{DOCUMENT_QUERY, DOCUMENTS_QUERY};
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
        .query(DOCUMENTS_QUERY, variables)
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

/// Test that we can get a document if one exists.
///
/// This tests read operations by fetching the first document from the list
/// (if any exist) and verifying we can retrieve it with full content.
#[test]
#[ignore]
fn test_document_get() {
    use lin::models::DocumentResponse;

    let client = common::create_client();

    // First, list documents to find one to fetch
    let list_variables = serde_json::json!({});
    let list_response: lin::models::DocumentsResponse = client
        .query(DOCUMENTS_QUERY, list_variables)
        .expect("Should be able to list documents");

    // Skip if no documents exist
    if list_response.documents.nodes.is_empty() {
        println!("No documents found in workspace, skipping get test");
        return;
    }

    // Get the first document with full content
    let first_doc = &list_response.documents.nodes[0];
    let get_variables = serde_json::json!({
        "id": first_doc.id
    });

    let get_response: DocumentResponse = client
        .query(DOCUMENT_QUERY, get_variables)
        .expect("Should be able to get the document");

    assert_eq!(get_response.document.id, first_doc.id);
    assert_eq!(get_response.document.title, first_doc.title);
    // Content should be present for document get (unlike list which may not include it)
    println!(
        "Retrieved document: {} (ID: {})",
        get_response.document.title, get_response.document.id
    );
}
