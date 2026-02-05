//! Attachment management commands.
//!
//! Commands for listing, uploading, and viewing attachments on Linear issues.

use crate::api::{queries, GraphQLClient};
use crate::commands::issue::is_uuid;
use crate::error::LinError;
use crate::models::{
    AttachmentCreateResponse, AttachmentResponse, FileUploadResponse, IssueAttachmentsResponse,
    IssuesResponse,
};
use crate::output::{output, OutputFormat};
use crate::Result;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::fs;
use std::path::Path;

/// Get the MIME type based on file extension.
fn get_mime_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .as_deref()
    {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("svg") => "image/svg+xml",
        Some("pdf") => "application/pdf",
        Some("txt") => "text/plain",
        Some("json") => "application/json",
        Some("xml") => "application/xml",
        Some("html") | Some("htm") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("zip") => "application/zip",
        Some("tar") => "application/x-tar",
        Some("gz") | Some("gzip") => "application/gzip",
        Some("mp4") => "video/mp4",
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        Some("doc") => "application/msword",
        Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        Some("xls") => "application/vnd.ms-excel",
        Some("xlsx") => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        Some("ppt") => "application/vnd.ms-powerpoint",
        Some("pptx") => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        _ => "application/octet-stream",
    }
}

/// Resolve an issue identifier or UUID to an issue ID.
///
/// If the identifier is a UUID, it's returned as-is.
/// If it's an identifier like "ENG-123", we look it up via the API.
fn resolve_issue_id(client: &GraphQLClient, identifier: &str) -> Result<String> {
    if is_uuid(identifier) {
        return Ok(identifier.to_string());
    }

    // Look up by identifier (e.g., "ENG-123")
    let variables = serde_json::json!({
        "filter": {
            "identifier": { "eq": identifier }
        }
    });

    let response: IssuesResponse = client.query(queries::ISSUE_BY_IDENTIFIER_QUERY, variables)?;

    response
        .issues
        .nodes
        .first()
        .map(|issue| issue.id.clone())
        .ok_or_else(|| LinError::api(format!("Issue '{}' not found", identifier)))
}

/// List attachments on an issue.
///
/// Fetches all attachments for a given issue and outputs them.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `issue_identifier` - The issue ID or identifier (e.g., "ENG-123")
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use lin::api::GraphQLClient;
/// use lin::commands::attachment::list_attachments;
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// list_attachments(&client, "ENG-123", OutputFormat::Human)?;
/// # Ok(())
/// # }
/// ```
pub fn list_attachments(
    client: &GraphQLClient,
    issue_identifier: &str,
    format: OutputFormat,
) -> Result<()> {
    let issue_id = resolve_issue_id(client, issue_identifier)?;

    let variables = serde_json::json!({
        "id": issue_id
    });

    let response: IssueAttachmentsResponse =
        client.query(queries::ISSUE_ATTACHMENTS_QUERY, variables)?;
    output(&response.issue.attachments.nodes, format);
    Ok(())
}

/// Get details of a specific attachment by ID.
///
/// Fetches a single attachment from the Linear API and outputs it.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `id` - The attachment's unique identifier
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use lin::api::GraphQLClient;
/// use lin::commands::attachment::get_attachment;
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// get_attachment(&client, "attachment-123", OutputFormat::Human)?;
/// # Ok(())
/// # }
/// ```
pub fn get_attachment(client: &GraphQLClient, id: &str, format: OutputFormat) -> Result<()> {
    let variables = serde_json::json!({
        "id": id
    });
    let response: AttachmentResponse = client.query(queries::ATTACHMENT_QUERY, variables)?;
    output(&response.attachment, format);
    Ok(())
}

/// Upload a file as an attachment to an issue.
///
/// This function:
/// 1. Reads the file from disk
/// 2. Requests a presigned upload URL from Linear
/// 3. Uploads the file to that URL
/// 4. Creates an attachment record linking the file to the issue
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `issue_identifier` - The issue ID or identifier (e.g., "ENG-123")
/// * `file_path` - Path to the file to upload
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use lin::api::GraphQLClient;
/// use lin::commands::attachment::upload_attachment;
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// upload_attachment(&client, "ENG-123", "/path/to/file.png", OutputFormat::Human)?;
/// # Ok(())
/// # }
/// ```
pub fn upload_attachment(
    client: &GraphQLClient,
    issue_identifier: &str,
    file_path: &str,
    format: OutputFormat,
) -> Result<()> {
    let path = Path::new(file_path);

    // Check if file exists
    if !path.exists() {
        return Err(LinError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("File not found: {}", file_path),
        )));
    }

    // Read the file
    let file_content = fs::read(path)?;
    let file_size = file_content.len() as i32;

    // Get filename
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| LinError::api("Invalid filename"))?
        .to_string();

    // Get MIME type
    let content_type = get_mime_type(path);

    // Resolve the issue ID
    let issue_id = resolve_issue_id(client, issue_identifier)?;

    // Step 1: Get a presigned upload URL
    let upload_variables = serde_json::json!({
        "contentType": content_type,
        "filename": filename,
        "size": file_size
    });

    let upload_response: FileUploadResponse =
        client.query(queries::FILE_UPLOAD_CREATE_MUTATION, upload_variables)?;

    let upload_file = &upload_response.file_upload.upload_file;

    // Step 2: Upload the file to the presigned URL
    let http_client = Client::new();

    let mut headers = HeaderMap::new();
    for header in &upload_file.headers {
        if let (Ok(name), Ok(value)) = (
            HeaderName::try_from(header.key.as_str()),
            HeaderValue::from_str(&header.value),
        ) {
            headers.insert(name, value);
        }
    }

    let upload_result = http_client
        .put(&upload_file.upload_url)
        .headers(headers)
        .body(file_content)
        .send()
        .map_err(|e| LinError::api(format!("Failed to upload file: {}", e)))?;

    if !upload_result.status().is_success() {
        let error_text = upload_result
            .text()
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(LinError::api(format!("File upload failed: {}", error_text)));
    }

    // Step 3: Create an attachment record
    let attachment_variables = serde_json::json!({
        "input": {
            "issueId": issue_id,
            "title": filename,
            "url": upload_file.asset_url
        }
    });

    let attachment_response: AttachmentCreateResponse =
        client.query(queries::ATTACHMENT_CREATE_MUTATION, attachment_variables)?;

    if let Some(attachment) = attachment_response.attachment_create.attachment {
        output(&attachment, format);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::GraphQLClient;

    #[test]
    fn test_get_mime_type() {
        assert_eq!(get_mime_type(Path::new("image.png")), "image/png");
        assert_eq!(get_mime_type(Path::new("image.jpg")), "image/jpeg");
        assert_eq!(get_mime_type(Path::new("image.JPEG")), "image/jpeg");
        assert_eq!(get_mime_type(Path::new("doc.pdf")), "application/pdf");
        assert_eq!(get_mime_type(Path::new("file.txt")), "text/plain");
        assert_eq!(
            get_mime_type(Path::new("unknown.xyz")),
            "application/octet-stream"
        );
        assert_eq!(
            get_mime_type(Path::new("no_extension")),
            "application/octet-stream"
        );
    }

    #[test]
    fn test_list_attachments_success() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Mock for issue identifier resolution
        let issue_mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .match_header("content-type", "application/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issues": {
                            "nodes": [
                                {
                                    "id": "issue-123",
                                    "identifier": "ENG-456",
                                    "title": "Test Issue",
                                    "description": null,
                                    "priority": 0,
                                    "state": null,
                                    "team": null,
                                    "assignee": null,
                                    "createdAt": "2024-01-01T00:00:00.000Z",
                                    "updatedAt": "2024-01-01T00:00:00.000Z"
                                }
                            ]
                        }
                    }
                }"#,
            )
            .create();

        // Mock for attachments query
        let attachments_mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issue": {
                            "id": "issue-123",
                            "identifier": "ENG-456",
                            "attachments": {
                                "nodes": [
                                    {
                                        "id": "attach-1",
                                        "title": "screenshot.png",
                                        "subtitle": "Bug screenshot",
                                        "url": "https://example.com/screenshot.png",
                                        "metadata": null,
                                        "createdAt": "2024-01-01T00:00:00.000Z",
                                        "updatedAt": "2024-01-01T00:00:00.000Z",
                                        "creator": null
                                    }
                                ]
                            }
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = list_attachments(&client, "ENG-456", OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mocks were called
        issue_mock.assert();
        attachments_mock.assert();
    }

    #[test]
    fn test_list_attachments_with_uuid() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Mock for attachments query (no identifier resolution needed for UUID)
        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issue": {
                            "id": "550e8400-e29b-41d4-a716-446655440000",
                            "identifier": "ENG-789",
                            "attachments": {
                                "nodes": []
                            }
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request with UUID (should skip identifier resolution)
        let result = list_attachments(
            &client,
            "550e8400-e29b-41d4-a716-446655440000",
            OutputFormat::Human,
        );

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called only once (no identifier resolution)
        mock.assert();
    }

    #[test]
    fn test_list_attachments_issue_not_found() {
        // Start mock server
        let mut server = mockito::Server::new();

        // Mock for issue identifier resolution - returns empty
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issues": {
                            "nodes": []
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = list_attachments(&client, "NONEXISTENT-999", OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not found"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_list_attachments_api_error() {
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
        let result = list_attachments(&client, "ENG-123", OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Not authenticated"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_get_attachment_success() {
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
                        "attachment": {
                            "id": "attach-123",
                            "title": "screenshot.png",
                            "subtitle": "Bug screenshot",
                            "url": "https://example.com/screenshot.png",
                            "metadata": null,
                            "createdAt": "2024-01-01T00:00:00.000Z",
                            "updatedAt": "2024-01-15T00:00:00.000Z",
                            "creator": {
                                "id": "user-1",
                                "name": "John Doe",
                                "email": "john@example.com",
                                "displayName": "JD",
                                "active": true
                            },
                            "issue": {
                                "id": "issue-123",
                                "identifier": "ENG-456"
                            }
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = get_attachment(&client, "attach-123", OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_get_attachment_not_found() {
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
        let result = get_attachment(&client, "nonexistent-attach", OutputFormat::Human);

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Entity not found"));

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_get_attachment_with_null_fields() {
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
                        "attachment": {
                            "id": "attach-456",
                            "title": "document.pdf",
                            "subtitle": null,
                            "url": "https://example.com/doc.pdf",
                            "metadata": null,
                            "createdAt": "2024-01-01T00:00:00.000Z",
                            "updatedAt": "2024-01-01T00:00:00.000Z",
                            "creator": null,
                            "issue": null
                        }
                    }
                }"#,
            )
            .create();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Make request
        let result = get_attachment(&client, "attach-456", OutputFormat::Human);

        // Verify success
        assert!(result.is_ok());

        // Verify mock was called
        mock.assert();
    }

    #[test]
    fn test_upload_attachment_file_not_found() {
        let server = mockito::Server::new();

        // Create client pointing to mock server
        let client = GraphQLClient::with_url("test-token", &server.url());

        // Try to upload a non-existent file
        let result = upload_attachment(
            &client,
            "ENG-123",
            "/nonexistent/path/file.txt",
            OutputFormat::Human,
        );

        // Verify error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not found") || err.to_string().contains("I/O error"));
    }
}
