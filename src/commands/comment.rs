//! Comment management commands.
//!
//! Commands for listing and creating comments on Linear issues.

use crate::api::queries::comment::COMMENT_CREATE_MUTATION;
use crate::api::queries::issue::{ISSUE_BY_IDENTIFIER_QUERY, ISSUE_COMMENTS_QUERY};
use crate::api::GraphQLClient;
use crate::error::LinError;
use crate::models::{CommentCreateResponse, IssueCommentsResponse, IssuesResponse};
use crate::output::{output, OutputFormat};
use crate::Result;

use super::issue::{is_uuid, parse_identifier};

/// List comments for an issue.
///
/// Fetches comments for a specific issue and outputs them.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `id_or_identifier` - The issue's UUID or human-readable identifier
/// * `format` - The output format (Human or Json)
pub fn list_comments(
    client: &GraphQLClient,
    id_or_identifier: &str,
    format: OutputFormat,
) -> Result<()> {
    // First, resolve the issue ID if given an identifier
    let issue_id = resolve_issue_id(client, id_or_identifier)?;

    let variables = serde_json::json!({
        "id": issue_id
    });

    let response: IssueCommentsResponse = client.query(ISSUE_COMMENTS_QUERY, variables)?;

    output(&response.issue.comments.nodes, format);
    Ok(())
}

/// Create a new comment on an issue.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `id_or_identifier` - The issue's UUID or human-readable identifier
/// * `body` - The comment body/content
/// * `format` - The output format (Human or Json)
pub fn create_comment(
    client: &GraphQLClient,
    id_or_identifier: &str,
    body: &str,
    format: OutputFormat,
) -> Result<()> {
    // First, resolve the issue ID if given an identifier
    let issue_id = resolve_issue_id(client, id_or_identifier)?;

    let variables = serde_json::json!({
        "input": {
            "issueId": issue_id,
            "body": body
        }
    });

    let response: CommentCreateResponse = client.query(COMMENT_CREATE_MUTATION, variables)?;

    if !response.comment_create.success {
        return Err(LinError::api("Failed to create comment"));
    }

    match response.comment_create.comment {
        Some(comment) => {
            output(&comment, format);
            Ok(())
        }
        None => Err(LinError::api(
            "Comment creation succeeded but no comment returned",
        )),
    }
}

/// Resolve an issue ID from either a UUID or an identifier like "ENG-123".
fn resolve_issue_id(client: &GraphQLClient, id_or_identifier: &str) -> Result<String> {
    if is_uuid(id_or_identifier) {
        Ok(id_or_identifier.to_string())
    } else {
        // Parse the identifier and look up the issue to get its UUID
        let (team_key, number) = parse_identifier(id_or_identifier)?;

        let lookup_variables = serde_json::json!({
            "filter": {
                "team": { "key": { "eq": team_key } },
                "number": { "eq": number }
            }
        });

        let lookup_response: IssuesResponse =
            client.query(ISSUE_BY_IDENTIFIER_QUERY, lookup_variables)?;

        if lookup_response.issues.nodes.is_empty() {
            return Err(LinError::api(format!(
                "Issue '{}' not found",
                id_or_identifier
            )));
        }

        Ok(lookup_response.issues.nodes[0].id.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::GraphQLClient;
    use crate::output::OutputFormat;

    #[test]
    fn test_list_comments_success() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "issue": {
                            "id": "issue-123",
                            "identifier": "ENG-123",
                            "comments": {
                                "nodes": [
                                    {
                                        "id": "comment-1",
                                        "body": "First comment",
                                        "createdAt": "2024-01-01T00:00:00.000Z",
                                        "updatedAt": "2024-01-01T00:00:00.000Z",
                                        "user": {
                                            "id": "user-1",
                                            "name": "John Doe",
                                            "email": "john@example.com",
                                            "displayName": "JD",
                                            "active": true
                                        }
                                    },
                                    {
                                        "id": "comment-2",
                                        "body": "Second comment",
                                        "createdAt": "2024-01-02T00:00:00.000Z",
                                        "updatedAt": "2024-01-02T00:00:00.000Z",
                                        "user": null
                                    }
                                ]
                            }
                        }
                    }
                }"##,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = list_comments(
            &client,
            "550e8400-e29b-41d4-a716-446655440000",
            OutputFormat::Human,
        );

        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_list_comments_by_identifier() {
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
                }"##,
            )
            .create();

        // Second mock: get comments
        let comments_mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "issue": {
                            "id": "issue-uuid-123",
                            "identifier": "ENG-123",
                            "comments": {
                                "nodes": []
                            }
                        }
                    }
                }"##,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = list_comments(&client, "ENG-123", OutputFormat::Human);

        assert!(result.is_ok());
        lookup_mock.assert();
        comments_mock.assert();
    }

    #[test]
    fn test_list_comments_issue_not_found() {
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
        let result = list_comments(&client, "ENG-99999", OutputFormat::Human);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not found"));
        mock.assert();
    }

    #[test]
    fn test_create_comment_success() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "commentCreate": {
                            "success": true,
                            "comment": {
                                "id": "comment-new",
                                "body": "This is my comment",
                                "createdAt": "2024-01-01T00:00:00.000Z",
                                "updatedAt": "2024-01-01T00:00:00.000Z",
                                "user": {
                                    "id": "user-1",
                                    "name": "John Doe",
                                    "email": "john@example.com",
                                    "displayName": "JD",
                                    "active": true
                                }
                            }
                        }
                    }
                }"##,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = create_comment(
            &client,
            "550e8400-e29b-41d4-a716-446655440000",
            "This is my comment",
            OutputFormat::Human,
        );

        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_create_comment_failure() {
        let mut server = mockito::Server::new();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "commentCreate": {
                            "success": false,
                            "comment": null
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = create_comment(
            &client,
            "550e8400-e29b-41d4-a716-446655440000",
            "My comment",
            OutputFormat::Human,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Failed to create comment"));
        mock.assert();
    }

    #[test]
    fn test_create_comment_by_identifier() {
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
                }"##,
            )
            .create();

        // Second mock: create comment
        let create_mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r##"{
                    "data": {
                        "commentCreate": {
                            "success": true,
                            "comment": {
                                "id": "comment-new",
                                "body": "Test comment",
                                "createdAt": "2024-01-01T00:00:00.000Z",
                                "updatedAt": "2024-01-01T00:00:00.000Z",
                                "user": null
                            }
                        }
                    }
                }"##,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());
        let result = create_comment(&client, "ENG-123", "Test comment", OutputFormat::Human);

        assert!(result.is_ok());
        lookup_mock.assert();
        create_mock.assert();
    }
}
