//! Document-related types for the Linear API.
//!
//! This module contains types for representing Linear documents and
//! document-related API responses.

use serde::{Deserialize, Serialize};

use super::user::User;

/// A simplified project reference for documents.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentProject {
    /// Unique identifier for the project.
    pub id: String,
    /// The project's name.
    pub name: String,
}

/// A Linear document (used in list view, without content).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    /// Unique identifier for the document.
    pub id: String,
    /// The document's title.
    pub title: String,
    /// Optional icon for the document.
    pub icon: Option<String>,
    /// Optional color for the document.
    pub color: Option<String>,
    /// ISO 8601 timestamp of when the document was created.
    pub created_at: String,
    /// ISO 8601 timestamp of when the document was last updated.
    pub updated_at: String,
    /// The user who created the document.
    pub creator: Option<User>,
    /// The project this document belongs to (optional).
    pub project: Option<DocumentProject>,
}

/// A Linear document with content (used in detail view).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentWithContent {
    /// Unique identifier for the document.
    pub id: String,
    /// The document's title.
    pub title: String,
    /// The document's content (markdown).
    pub content: Option<String>,
    /// Optional icon for the document.
    pub icon: Option<String>,
    /// Optional color for the document.
    pub color: Option<String>,
    /// ISO 8601 timestamp of when the document was created.
    pub created_at: String,
    /// ISO 8601 timestamp of when the document was last updated.
    pub updated_at: String,
    /// The user who created the document.
    pub creator: Option<User>,
    /// The project this document belongs to (optional).
    pub project: Option<DocumentProject>,
}

/// A paginated list of documents.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentConnection {
    /// List of documents.
    pub nodes: Vec<Document>,
}

/// Response wrapper for documents query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentsResponse {
    /// Paginated list of documents.
    pub documents: DocumentConnection,
}

/// Response wrapper for a single document query (with content).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentResponse {
    /// The requested document with content.
    pub document: DocumentWithContent,
}

/// Response for document creation mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentCreatePayload {
    /// Whether the mutation was successful.
    pub success: bool,
    /// The created document.
    pub document: Option<DocumentWithContent>,
}

/// Response wrapper for document creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentCreateResponse {
    /// The mutation payload.
    pub document_create: DocumentCreatePayload,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_deserialization() {
        let json = r##"{
            "id": "doc-123",
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
        }"##;
        let document: Document = serde_json::from_str(json).unwrap();
        assert_eq!(document.id, "doc-123");
        assert_eq!(document.title, "Project Overview");
        assert_eq!(document.icon, Some("document".to_string()));
        assert!(document.creator.is_some());
        assert!(document.project.is_some());
    }

    #[test]
    fn test_document_with_null_optional_fields() {
        let json = r#"{
            "id": "doc-456",
            "title": "Simple Doc",
            "icon": null,
            "color": null,
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-01T00:00:00.000Z",
            "creator": null,
            "project": null
        }"#;
        let document: Document = serde_json::from_str(json).unwrap();
        assert_eq!(document.id, "doc-456");
        assert!(document.icon.is_none());
        assert!(document.color.is_none());
        assert!(document.creator.is_none());
        assert!(document.project.is_none());
    }

    #[test]
    fn test_document_with_content_deserialization() {
        let json = r#"{
            "id": "doc-789",
            "title": "Technical Spec",
            "content": "Overview content here.",
            "icon": null,
            "color": null,
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-15T00:00:00.000Z",
            "creator": null,
            "project": null
        }"#;
        let document: DocumentWithContent = serde_json::from_str(json).unwrap();
        assert_eq!(document.id, "doc-789");
        assert_eq!(document.title, "Technical Spec");
        assert_eq!(document.content, Some("Overview content here.".to_string()));
    }

    #[test]
    fn test_documents_response_deserialization() {
        let json = r#"{
            "documents": {
                "nodes": [
                    {
                        "id": "doc-1",
                        "title": "Doc One",
                        "icon": null,
                        "color": null,
                        "createdAt": "2024-01-01T00:00:00.000Z",
                        "updatedAt": "2024-01-01T00:00:00.000Z",
                        "creator": null,
                        "project": null
                    }
                ]
            }
        }"#;
        let response: DocumentsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.documents.nodes.len(), 1);
        assert_eq!(response.documents.nodes[0].title, "Doc One");
    }

    #[test]
    fn test_document_create_response_deserialization() {
        let json = r#"{
            "documentCreate": {
                "success": true,
                "document": {
                    "id": "doc-new",
                    "title": "New Document",
                    "content": "Content here",
                    "icon": null,
                    "color": null,
                    "createdAt": "2024-01-01T00:00:00.000Z",
                    "updatedAt": "2024-01-01T00:00:00.000Z",
                    "creator": null,
                    "project": null
                }
            }
        }"#;
        let response: DocumentCreateResponse = serde_json::from_str(json).unwrap();
        assert!(response.document_create.success);
        assert!(response.document_create.document.is_some());
        assert_eq!(
            response.document_create.document.unwrap().title,
            "New Document"
        );
    }
}
