//! Attachment-related types for the Linear API.
//!
//! This module contains types for representing Linear attachments,
//! file uploads, and attachment-related API responses.

use serde::{Deserialize, Serialize};

use super::user::User;

/// A simplified issue reference for attachments.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentIssue {
    /// Unique identifier for the issue.
    pub id: String,
    /// Human-readable identifier (e.g., "ENG-123").
    pub identifier: String,
}

/// A Linear attachment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    /// Unique identifier for the attachment.
    pub id: String,
    /// The attachment's title.
    pub title: String,
    /// The attachment's subtitle/description (optional).
    pub subtitle: Option<String>,
    /// URL where the attachment can be accessed.
    pub url: String,
    /// Additional metadata about the attachment.
    pub metadata: Option<serde_json::Value>,
    /// ISO 8601 timestamp of when the attachment was created.
    pub created_at: String,
    /// ISO 8601 timestamp of when the attachment was last updated.
    pub updated_at: String,
    /// The user who created the attachment.
    pub creator: Option<User>,
}

/// A Linear attachment with its associated issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentWithIssue {
    /// Unique identifier for the attachment.
    pub id: String,
    /// The attachment's title.
    pub title: String,
    /// The attachment's subtitle/description (optional).
    pub subtitle: Option<String>,
    /// URL where the attachment can be accessed.
    pub url: String,
    /// Additional metadata about the attachment.
    pub metadata: Option<serde_json::Value>,
    /// ISO 8601 timestamp of when the attachment was created.
    pub created_at: String,
    /// ISO 8601 timestamp of when the attachment was last updated.
    pub updated_at: String,
    /// The user who created the attachment.
    pub creator: Option<User>,
    /// The issue this attachment belongs to.
    pub issue: Option<AttachmentIssue>,
}

/// A paginated list of attachments.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentConnection {
    /// List of attachments.
    pub nodes: Vec<Attachment>,
}

/// Response wrapper for a single attachment query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentResponse {
    /// The requested attachment.
    pub attachment: AttachmentWithIssue,
}

/// Response for attachment creation mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentCreatePayload {
    /// Whether the mutation was successful.
    pub success: bool,
    /// The created attachment.
    pub attachment: Option<Attachment>,
}

/// Response wrapper for attachment creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentCreateResponse {
    /// The mutation payload.
    pub attachment_create: AttachmentCreatePayload,
}

/// Header for file upload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadHeader {
    /// Header key.
    pub key: String,
    /// Header value.
    pub value: String,
}

/// Upload file details returned by file upload mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadFile {
    /// Presigned URL where the file should be uploaded.
    pub upload_url: String,
    /// Final URL where the file will be available after upload.
    pub asset_url: String,
    /// Headers to include in the upload request.
    pub headers: Vec<UploadHeader>,
}

/// Response for file upload mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileUploadPayload {
    /// The upload file details.
    pub upload_file: UploadFile,
}

/// Response wrapper for file upload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileUploadResponse {
    /// The mutation payload.
    pub file_upload: FileUploadPayload,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attachment_deserialization() {
        let json = r#"{
            "id": "attach-123",
            "title": "Screenshot.png",
            "subtitle": "Bug screenshot",
            "url": "https://example.com/screenshot.png",
            "metadata": {"size": 1024},
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-15T00:00:00.000Z",
            "creator": {
                "id": "user-1",
                "name": "John Doe",
                "email": "john@example.com",
                "displayName": "JD",
                "active": true
            }
        }"#;
        let attachment: Attachment = serde_json::from_str(json).unwrap();
        assert_eq!(attachment.id, "attach-123");
        assert_eq!(attachment.title, "Screenshot.png");
        assert_eq!(attachment.subtitle, Some("Bug screenshot".to_string()));
        assert_eq!(attachment.url, "https://example.com/screenshot.png");
        assert!(attachment.metadata.is_some());
        assert!(attachment.creator.is_some());
    }

    #[test]
    fn test_attachment_with_null_optional_fields() {
        let json = r#"{
            "id": "attach-456",
            "title": "Document.pdf",
            "subtitle": null,
            "url": "https://example.com/doc.pdf",
            "metadata": null,
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-01T00:00:00.000Z",
            "creator": null
        }"#;
        let attachment: Attachment = serde_json::from_str(json).unwrap();
        assert_eq!(attachment.id, "attach-456");
        assert!(attachment.subtitle.is_none());
        assert!(attachment.metadata.is_none());
        assert!(attachment.creator.is_none());
    }

    #[test]
    fn test_attachment_with_issue_deserialization() {
        let json = r#"{
            "id": "attach-789",
            "title": "Log file",
            "subtitle": null,
            "url": "https://example.com/log.txt",
            "metadata": null,
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-01T00:00:00.000Z",
            "creator": null,
            "issue": {
                "id": "issue-123",
                "identifier": "ENG-456"
            }
        }"#;
        let attachment: AttachmentWithIssue = serde_json::from_str(json).unwrap();
        assert_eq!(attachment.id, "attach-789");
        assert!(attachment.issue.is_some());
        assert_eq!(attachment.issue.as_ref().unwrap().identifier, "ENG-456");
    }

    #[test]
    fn test_attachment_create_response_deserialization() {
        let json = r#"{
            "attachmentCreate": {
                "success": true,
                "attachment": {
                    "id": "attach-new",
                    "title": "Uploaded.png",
                    "subtitle": null,
                    "url": "https://example.com/uploaded.png",
                    "metadata": null,
                    "createdAt": "2024-01-01T00:00:00.000Z",
                    "updatedAt": "2024-01-01T00:00:00.000Z",
                    "creator": null
                }
            }
        }"#;
        let response: AttachmentCreateResponse = serde_json::from_str(json).unwrap();
        assert!(response.attachment_create.success);
        assert!(response.attachment_create.attachment.is_some());
        assert_eq!(
            response.attachment_create.attachment.unwrap().title,
            "Uploaded.png"
        );
    }

    #[test]
    fn test_file_upload_response_deserialization() {
        let json = r#"{
            "fileUpload": {
                "uploadFile": {
                    "uploadUrl": "https://upload.example.com/presigned",
                    "assetUrl": "https://assets.example.com/file.png",
                    "headers": [
                        {
                            "key": "Content-Type",
                            "value": "image/png"
                        }
                    ]
                }
            }
        }"#;
        let response: FileUploadResponse = serde_json::from_str(json).unwrap();
        assert_eq!(
            response.file_upload.upload_file.upload_url,
            "https://upload.example.com/presigned"
        );
        assert_eq!(
            response.file_upload.upload_file.asset_url,
            "https://assets.example.com/file.png"
        );
        assert_eq!(response.file_upload.upload_file.headers.len(), 1);
        assert_eq!(
            response.file_upload.upload_file.headers[0].key,
            "Content-Type"
        );
    }
}
