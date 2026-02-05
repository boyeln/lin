//! Document management commands.
//!
//! Commands for listing, viewing, and creating documents in Linear.

use crate::api::queries::document::{DOCUMENTS_QUERY, DOCUMENT_CREATE_MUTATION, DOCUMENT_QUERY};
use crate::api::GraphQLClient;
use crate::models::{DocumentCreateResponse, DocumentResponse, DocumentsResponse};
use crate::output::{output, OutputFormat};
use crate::Result;

/// Options for listing documents.
#[derive(Debug, Clone, Default)]
pub struct DocumentListOptions {
    /// Filter by project ID (optional).
    pub project_id: Option<String>,
}

/// Options for creating a document.
#[derive(Debug, Clone)]
pub struct DocumentCreateOptions {
    /// The document title.
    pub title: String,
    /// The document content (markdown).
    pub content: String,
    /// Project ID to associate the document with (optional).
    pub project_id: Option<String>,
}

/// List all documents in the organization.
///
/// Fetches documents from the Linear API and outputs them.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `options` - Filter options for the query
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use lin::api::GraphQLClient;
/// use lin::commands::document::{list_documents, DocumentListOptions};
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// let options = DocumentListOptions::default();
/// list_documents(&client, options, OutputFormat::Human)?;
/// # Ok(())
/// # }
/// ```
pub fn list_documents(
    client: &GraphQLClient,
    options: DocumentListOptions,
    format: OutputFormat,
) -> Result<()> {
    // Build variables
    let mut variables = serde_json::Map::new();

    // Build the filter object if we have a project filter
    if let Some(project_id) = &options.project_id {
        let filter = serde_json::json!({
            "project": {
                "id": { "eq": project_id }
            }
        });
        variables.insert("filter".to_string(), filter);
    }

    let response: DocumentsResponse =
        client.query(DOCUMENTS_QUERY, serde_json::Value::Object(variables))?;
    output(&response.documents.nodes, format);
    Ok(())
}

/// Get details of a specific document by ID (including content).
///
/// Fetches a single document from the Linear API and outputs it.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `id` - The document's unique identifier
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use lin::api::GraphQLClient;
/// use lin::commands::document::get_document;
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// get_document(&client, "doc-123", OutputFormat::Human)?;
/// # Ok(())
/// # }
/// ```
pub fn get_document(client: &GraphQLClient, id: &str, format: OutputFormat) -> Result<()> {
    let variables = serde_json::json!({
        "id": id
    });
    let response: DocumentResponse = client.query(DOCUMENT_QUERY, variables)?;
    output(&response.document, format);
    Ok(())
}

/// Create a new document.
///
/// Creates a document in the Linear API and outputs the result.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `options` - Options for creating the document
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use lin::api::GraphQLClient;
/// use lin::commands::document::{create_document, DocumentCreateOptions};
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// let options = DocumentCreateOptions {
///     title: "My Document".to_string(),
///     content: "# Hello\n\nWorld".to_string(),
///     project_id: None,
/// };
/// create_document(&client, options, OutputFormat::Human)?;
/// # Ok(())
/// # }
/// ```
pub fn create_document(
    client: &GraphQLClient,
    options: DocumentCreateOptions,
    format: OutputFormat,
) -> Result<()> {
    // Build the input object
    let mut input = serde_json::Map::new();
    input.insert("title".to_string(), serde_json::json!(options.title));
    input.insert("content".to_string(), serde_json::json!(options.content));

    if let Some(project_id) = options.project_id {
        input.insert("projectId".to_string(), serde_json::json!(project_id));
    }

    let variables = serde_json::json!({
        "input": serde_json::Value::Object(input)
    });

    let response: DocumentCreateResponse = client.query(DOCUMENT_CREATE_MUTATION, variables)?;

    if let Some(document) = response.document_create.document {
        output(&document, format);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::GraphQLClient;

    #[test]
    fn test_list_documents_success() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response
        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .match_header("content-type", "application/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "documents": {
                            "nodes": [
                                {
                                    "id": "doc-1",
                                    "title": "Project Overview",
                                    "icon": "document",
                                    "color": "#0066ff",
                                    "createdAt": "2024-01-01T00:00:00.000Z",
                                    "updatedAt": "2024-01-15T00:00:00.000Z",
                                    "creator": {
                                        "id": "user-1",
                                        "name": "John Doe",
                                        "email": "john@example.com",
                                        "displayName": "JD",
                                        "active": true
                                    },
                                    "project": {
                                        "id": "project-1",
                                        "name": "Q1 Roadmap"
                                    }
                                },
                                {
                                    "id": "doc-2",
                                    "title": "Technical Spec",
                                    "icon": null,
                                    "color": null,
                                    "createdAt": "2024-02-01T00:00:00.000Z",
                                    "updatedAt": "2024-02-01T00:00:00.000Z",
                                    "creator": null,
                                    "project": null
                                }
                            ]
                        }
                    }
                }"##,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let options = DocumentListOptions::default();
        let result = list_documents(&client, options, OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_documents_with_project_filter() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "documents": {
                            "nodes": [
                                {
                                    "id": "doc-1",
                                    "title": "Project Doc",
                                    "icon": null,
                                    "color": null,
                                    "createdAt": "2024-01-01T00:00:00.000Z",
                                    "updatedAt": "2024-01-15T00:00:00.000Z",
                                    "creator": null,
                                    "project": {
                                        "id": "project-123",
                                        "name": "Test Project"
                                    }
                                }
                            ]
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request with project filter
        let options = DocumentListOptions {
            project_id: Some("project-123".to_string()),
        };
        let result = list_documents(&client, options, OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_documents_empty() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with empty documents
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "documents": {
                            "nodes": []
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let options = DocumentListOptions::default();
        let result = list_documents(&client, options, OutputFormat::Human);

        // Verify success (empty list is still valid)
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_documents_api_error() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with GraphQL error
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": null,
                    "errors": [
                        {
                            "message": "Not authenticated"
                        }
                    ]
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("invalid-token", &server.url());

        // Make request
        let options = DocumentListOptions::default();
        let result = list_documents(&client, options, OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Not authenticated"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_documents_http_error() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with HTTP error
        let mock = server
            .mock("POST", "/")
            .with_status(401)
            .with_body(r#"{"error": "Unauthorized"}"#)
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("invalid-token", &server.url());

        // Make request
        let options = DocumentListOptions::default();
        let result = list_documents(&client, options, OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("HTTP 401"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_get_document_success() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response
        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .match_header("content-type", "application/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "document": {
                            "id": "doc-123",
                            "title": "Project Overview",
                            "content": "# Overview\n\nThis is the content.",
                            "icon": "document",
                            "color": "#0066ff",
                            "createdAt": "2024-01-01T00:00:00.000Z",
                            "updatedAt": "2024-01-15T00:00:00.000Z",
                            "creator": {
                                "id": "user-1",
                                "name": "John Doe",
                                "email": "john@example.com",
                                "displayName": "JD",
                                "active": true
                            },
                            "project": {
                                "id": "project-1",
                                "name": "Q1 Roadmap"
                            }
                        }
                    }
                }"##,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = get_document(&client, "doc-123", OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_get_document_with_null_fields() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with null optional fields
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "document": {
                            "id": "doc-456",
                            "title": "Simple Doc",
                            "content": null,
                            "icon": null,
                            "color": null,
                            "createdAt": "2024-01-01T00:00:00.000Z",
                            "updatedAt": "2024-01-01T00:00:00.000Z",
                            "creator": null,
                            "project": null
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = get_document(&client, "doc-456", OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_get_document_not_found() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with GraphQL error for not found
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": null,
                    "errors": [
                        {
                            "message": "Entity not found"
                        }
                    ]
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = get_document(&client, "nonexistent-doc", OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Entity not found"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_get_document_api_error() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with authentication error
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": null,
                    "errors": [
                        {
                            "message": "Not authenticated"
                        }
                    ]
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("invalid-token", &server.url());

        // Make request
        let result = get_document(&client, "doc-123", OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Not authenticated"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_get_document_http_error() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with HTTP 500
        let mock = server
            .mock("POST", "/")
            .with_status(500)
            .with_body("Internal Server Error")
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = get_document(&client, "doc-123", OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("HTTP 500"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_create_document_success() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response
        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .match_header("content-type", "application/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "documentCreate": {
                            "success": true,
                            "document": {
                                "id": "doc-new",
                                "title": "New Document",
                                "content": "Hello World content",
                                "icon": null,
                                "color": null,
                                "createdAt": "2024-01-01T00:00:00.000Z",
                                "updatedAt": "2024-01-01T00:00:00.000Z",
                                "creator": {
                                    "id": "user-1",
                                    "name": "John Doe",
                                    "email": "john@example.com",
                                    "displayName": "JD",
                                    "active": true
                                },
                                "project": null
                            }
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let options = DocumentCreateOptions {
            title: "New Document".to_string(),
            content: "Hello World content".to_string(),
            project_id: None,
        };
        let result = create_document(&client, options, OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_create_document_with_project() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "documentCreate": {
                            "success": true,
                            "document": {
                                "id": "doc-project",
                                "title": "Project Doc",
                                "content": "Content",
                                "icon": null,
                                "color": null,
                                "createdAt": "2024-01-01T00:00:00.000Z",
                                "updatedAt": "2024-01-01T00:00:00.000Z",
                                "creator": null,
                                "project": {
                                    "id": "project-123",
                                    "name": "Test Project"
                                }
                            }
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request with project
        let options = DocumentCreateOptions {
            title: "Project Doc".to_string(),
            content: "Content".to_string(),
            project_id: Some("project-123".to_string()),
        };
        let result = create_document(&client, options, OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_create_document_api_error() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with GraphQL error
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": null,
                    "errors": [
                        {
                            "message": "Not authenticated"
                        }
                    ]
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("invalid-token", &server.url());

        // Make request
        let options = DocumentCreateOptions {
            title: "Test".to_string(),
            content: "Content".to_string(),
            project_id: None,
        };
        let result = create_document(&client, options, OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Not authenticated"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_create_document_http_error() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Set up mock response with HTTP error
        let mock = server
            .mock("POST", "/")
            .with_status(401)
            .with_body(r#"{"error": "Unauthorized"}"#)
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("invalid-token", &server.url());

        // Make request
        let options = DocumentCreateOptions {
            title: "Test".to_string(),
            content: "Content".to_string(),
            project_id: None,
        };
        let result = create_document(&client, options, OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("HTTP 401"));

        // Verify mock was called
        mock.assert();
    }
}
