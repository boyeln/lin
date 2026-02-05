//! Delete and archive operations for issues.

use crate::Result;
use crate::api::GraphQLClient;
use crate::api::queries::issue::{
    ISSUE_ARCHIVE_MUTATION, ISSUE_DELETE_MUTATION, ISSUE_UNARCHIVE_MUTATION,
};
use crate::error::LinError;
use crate::models::{IssueArchiveResponse, IssueDeleteResponse, IssueUnarchiveResponse};
use crate::output::{OutputFormat, output};

use super::{MessageResponse, resolve_issue_id};

/// Delete an issue in Linear.
///
/// Deletes an issue identified by ID or identifier (e.g., "ENG-123").
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `id_or_identifier` - The issue's UUID or human-readable identifier
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use lin::api::GraphQLClient;
/// use lin::commands::issue::delete::delete_issue;
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// delete_issue(&client, "ENG-123", OutputFormat::Human)?;
/// # Ok(())
/// # }
/// ```
pub fn delete_issue(
    client: &GraphQLClient,
    id_or_identifier: &str,
    format: OutputFormat,
) -> Result<()> {
    let issue_id = resolve_issue_id(client, id_or_identifier)?;

    let variables = serde_json::json!({ "id": issue_id });
    let response: IssueDeleteResponse = client.query(ISSUE_DELETE_MUTATION, variables)?;

    if !response.issue_delete.success {
        return Err(LinError::api("Failed to delete issue"));
    }

    let message = MessageResponse {
        message: format!("Issue '{}' deleted successfully", id_or_identifier),
    };
    output(&message, format);
    Ok(())
}

/// Archive an issue in Linear.
///
/// Archives an issue identified by ID or identifier (e.g., "ENG-123").
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `id_or_identifier` - The issue's UUID or human-readable identifier
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use lin::api::GraphQLClient;
/// use lin::commands::issue::delete::archive_issue;
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// archive_issue(&client, "ENG-123", OutputFormat::Human)?;
/// # Ok(())
/// # }
/// ```
pub fn archive_issue(
    client: &GraphQLClient,
    id_or_identifier: &str,
    format: OutputFormat,
) -> Result<()> {
    let issue_id = resolve_issue_id(client, id_or_identifier)?;

    let variables = serde_json::json!({ "id": issue_id });
    let response: IssueArchiveResponse = client.query(ISSUE_ARCHIVE_MUTATION, variables)?;

    if !response.issue_archive.success {
        return Err(LinError::api("Failed to archive issue"));
    }

    let message = MessageResponse {
        message: format!("Issue '{}' archived successfully", id_or_identifier),
    };
    output(&message, format);
    Ok(())
}

/// Unarchive an issue in Linear.
///
/// Unarchives an issue identified by ID or identifier (e.g., "ENG-123").
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `id_or_identifier` - The issue's UUID or human-readable identifier
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use lin::api::GraphQLClient;
/// use lin::commands::issue::delete::unarchive_issue;
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// unarchive_issue(&client, "ENG-123", OutputFormat::Human)?;
/// # Ok(())
/// # }
/// ```
pub fn unarchive_issue(
    client: &GraphQLClient,
    id_or_identifier: &str,
    format: OutputFormat,
) -> Result<()> {
    let issue_id = resolve_issue_id(client, id_or_identifier)?;

    let variables = serde_json::json!({ "id": issue_id });
    let response: IssueUnarchiveResponse = client.query(ISSUE_UNARCHIVE_MUTATION, variables)?;

    if !response.issue_unarchive.success {
        return Err(LinError::api("Failed to unarchive issue"));
    }

    let message = MessageResponse {
        message: format!("Issue '{}' unarchived successfully", id_or_identifier),
    };
    output(&message, format);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::GraphQLClient;
    use crate::output::OutputFormat;

    // =============================================================================
    // delete_issue tests
    // =============================================================================

    #[test]
    fn test_delete_issue_by_uuid() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issueDelete": {
                            "success": true
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = delete_issue(
            &client,
            "550e8400-e29b-41d4-a716-446655440000",
            OutputFormat::Human,
        );

        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_delete_issue_by_identifier() {
        let mut server = mockito::Server::new();

        // First mock: lookup by identifier
        let lookup_mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "issues": {
                            "nodes": [
                                {
                                    "id": "issue-uuid-123",
                                    "identifier": "ENG-123",
                                    "title": "Issue to delete",
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
                }"##,
            )
            .create();

        // Second mock: delete mutation
        let delete_mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issueDelete": {
                            "success": true
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = delete_issue(&client, "ENG-123", OutputFormat::Human);

        assert!(result.is_ok());
        lookup_mock.assert();
        delete_mock.assert();
    }

    #[test]
    fn test_delete_issue_not_found() {
        let mut server = mockito::Server::new();

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

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = delete_issue(&client, "ENG-99999", OutputFormat::Human);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not found"));
        mock.assert();
    }

    #[test]
    fn test_delete_issue_failure() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issueDelete": {
                            "success": false
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = delete_issue(
            &client,
            "550e8400-e29b-41d4-a716-446655440000",
            OutputFormat::Human,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Failed to delete issue"));
        mock.assert();
    }

    #[test]
    fn test_delete_issue_invalid_identifier() {
        let server = mockito::Server::new();
        let client = GraphQLClient::with_url("test-token", &server.url());

        let result = delete_issue(&client, "invalid-identifier", OutputFormat::Human);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid team key"));
    }

    // =============================================================================
    // archive_issue tests
    // =============================================================================

    #[test]
    fn test_archive_issue_by_uuid() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issueArchive": {
                            "success": true
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = archive_issue(
            &client,
            "550e8400-e29b-41d4-a716-446655440000",
            OutputFormat::Human,
        );

        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_archive_issue_by_identifier() {
        let mut server = mockito::Server::new();

        // First mock: lookup by identifier
        let lookup_mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "issues": {
                            "nodes": [
                                {
                                    "id": "issue-uuid-456",
                                    "identifier": "ENG-456",
                                    "title": "Issue to archive",
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
                }"##,
            )
            .create();

        // Second mock: archive mutation
        let archive_mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issueArchive": {
                            "success": true
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = archive_issue(&client, "ENG-456", OutputFormat::Human);

        assert!(result.is_ok());
        lookup_mock.assert();
        archive_mock.assert();
    }

    #[test]
    fn test_archive_issue_failure() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issueArchive": {
                            "success": false
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = archive_issue(
            &client,
            "550e8400-e29b-41d4-a716-446655440000",
            OutputFormat::Human,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Failed to archive issue"));
        mock.assert();
    }

    #[test]
    fn test_archive_issue_not_found() {
        let mut server = mockito::Server::new();

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

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = archive_issue(&client, "ENG-99999", OutputFormat::Human);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not found"));
        mock.assert();
    }

    // =============================================================================
    // unarchive_issue tests
    // =============================================================================

    #[test]
    fn test_unarchive_issue_by_uuid() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issueUnarchive": {
                            "success": true
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = unarchive_issue(
            &client,
            "550e8400-e29b-41d4-a716-446655440000",
            OutputFormat::Human,
        );

        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_unarchive_issue_by_identifier() {
        let mut server = mockito::Server::new();

        // First mock: lookup by identifier
        let lookup_mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "issues": {
                            "nodes": [
                                {
                                    "id": "issue-uuid-789",
                                    "identifier": "ENG-789",
                                    "title": "Issue to unarchive",
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
                }"##,
            )
            .create();

        // Second mock: unarchive mutation
        let unarchive_mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issueUnarchive": {
                            "success": true
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = unarchive_issue(&client, "ENG-789", OutputFormat::Human);

        assert!(result.is_ok());
        lookup_mock.assert();
        unarchive_mock.assert();
    }

    #[test]
    fn test_unarchive_issue_failure() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issueUnarchive": {
                            "success": false
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = unarchive_issue(
            &client,
            "550e8400-e29b-41d4-a716-446655440000",
            OutputFormat::Human,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Failed to unarchive issue"));
        mock.assert();
    }

    #[test]
    fn test_unarchive_issue_not_found() {
        let mut server = mockito::Server::new();

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

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = unarchive_issue(&client, "ENG-99999", OutputFormat::Human);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not found"));
        mock.assert();
    }
}
