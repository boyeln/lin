//! Issue-related types for the Linear API.
//!
//! This module contains types for representing Linear issues and
//! issue-related API responses and mutations.

use serde::{Deserialize, Serialize};

use super::attachment::AttachmentConnection;
use super::comment::CommentConnection;
use super::team::Team;
use super::user::User;
use super::workflow::WorkflowState;

/// A Linear issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    /// Unique identifier for the issue.
    pub id: String,
    /// Human-readable identifier (e.g., "ENG-123").
    pub identifier: String,
    /// Issue title.
    pub title: String,
    /// Issue description (may be empty or null).
    pub description: Option<String>,
    /// Priority level (0 = no priority, 1 = urgent, 2 = high, 3 = normal, 4 = low).
    pub priority: i32,
    /// Current workflow state.
    pub state: Option<WorkflowState>,
    /// Team the issue belongs to.
    pub team: Option<Team>,
    /// User assigned to the issue.
    pub assignee: Option<User>,
    /// ISO 8601 timestamp of when the issue was created.
    pub created_at: String,
    /// ISO 8601 timestamp of when the issue was last updated.
    pub updated_at: String,
    /// Estimate of the issue (e.g., story points).
    pub estimate: Option<f64>,
}

/// A paginated list of issues.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueConnection {
    /// List of issues.
    pub nodes: Vec<Issue>,
}

/// An issue with its comments included.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueWithComments {
    /// Unique identifier for the issue.
    pub id: String,
    /// Human-readable identifier (e.g., "ENG-123").
    pub identifier: String,
    /// Issue title.
    pub title: String,
    /// Issue description (may be empty or null).
    pub description: Option<String>,
    /// Priority level (0 = no priority, 1 = urgent, 2 = high, 3 = normal, 4 = low).
    pub priority: i32,
    /// Current workflow state.
    pub state: Option<WorkflowState>,
    /// Team the issue belongs to.
    pub team: Option<Team>,
    /// User assigned to the issue.
    pub assignee: Option<User>,
    /// ISO 8601 timestamp of when the issue was created.
    pub created_at: String,
    /// ISO 8601 timestamp of when the issue was last updated.
    pub updated_at: String,
    /// Estimate of the issue (e.g., story points).
    pub estimate: Option<f64>,
    /// Comments on the issue.
    pub comments: CommentConnection,
}

/// A paginated list of issues with comments.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueWithCommentsConnection {
    /// List of issues with comments.
    pub nodes: Vec<IssueWithComments>,
}

/// Issue with only comments (used for fetching just comments).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueWithCommentsOnly {
    /// Unique identifier for the issue.
    pub id: String,
    /// Human-readable identifier (e.g., "ENG-123").
    pub identifier: String,
    /// Comments on the issue.
    pub comments: CommentConnection,
}

/// Issue with attachments for the attachments query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueWithAttachments {
    /// Unique identifier for the issue.
    pub id: String,
    /// Human-readable identifier (e.g., "ENG-123").
    pub identifier: String,
    /// The issue's attachments.
    pub attachments: AttachmentConnection,
}

// =============================================================================
// Query Response Types
// =============================================================================

/// Response wrapper for the issues query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssuesResponse {
    /// Paginated list of issues.
    pub issues: IssueConnection,
}

/// Response wrapper for the issue search query (using issues query with filter).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueSearchResponse {
    /// Search results as paginated list of issues.
    pub issues: IssueConnection,
}

/// Response wrapper for a single issue query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueResponse {
    /// The requested issue.
    pub issue: Issue,
}

/// Response wrapper for a single issue with comments query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueWithCommentsResponse {
    /// The requested issue with comments.
    pub issue: IssueWithComments,
}

/// Response wrapper for issues query returning issues with comments.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssuesWithCommentsResponse {
    /// Paginated list of issues with comments.
    pub issues: IssueWithCommentsConnection,
}

/// Response wrapper for comments query on an issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueCommentsResponse {
    /// The issue containing the comments.
    pub issue: IssueWithCommentsOnly,
}

/// Response wrapper for issue attachments query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueAttachmentsResponse {
    /// The issue with its attachments.
    pub issue: IssueWithAttachments,
}

// =============================================================================
// Mutation Response Types
// =============================================================================

/// Response for issue creation mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueCreatePayload {
    /// Whether the mutation was successful.
    pub success: bool,
    /// The created issue.
    pub issue: Option<Issue>,
}

/// Response wrapper for issue creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueCreateResponse {
    /// The mutation payload.
    pub issue_create: IssueCreatePayload,
}

/// Response for issue update mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueUpdatePayload {
    /// Whether the mutation was successful.
    pub success: bool,
    /// The updated issue.
    pub issue: Option<Issue>,
}

/// Response wrapper for issue update.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueUpdateResponse {
    /// The mutation payload.
    pub issue_update: IssueUpdatePayload,
}

/// Response for issue deletion mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueDeletePayload {
    /// Whether the mutation was successful.
    pub success: bool,
}

/// Response wrapper for issue deletion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueDeleteResponse {
    /// The mutation payload.
    pub issue_delete: IssueDeletePayload,
}

/// Response for issue archive mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueArchivePayload {
    /// Whether the mutation was successful.
    pub success: bool,
}

/// Response wrapper for issue archive.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueArchiveResponse {
    /// The mutation payload.
    pub issue_archive: IssueArchivePayload,
}

/// Response for issue unarchive mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueUnarchivePayload {
    /// Whether the mutation was successful.
    pub success: bool,
}

/// Response wrapper for issue unarchive.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueUnarchiveResponse {
    /// The mutation payload.
    pub issue_unarchive: IssueUnarchivePayload,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_deserialization() {
        let json = r##"{
            "id": "issue-abc",
            "identifier": "ENG-123",
            "title": "Fix the bug",
            "description": "This is a bug description",
            "priority": 2,
            "state": {
                "id": "state-1",
                "name": "In Progress",
                "color": "#0066ff",
                "type": "started"
            },
            "team": {
                "id": "team-1",
                "key": "ENG",
                "name": "Engineering",
                "description": null
            },
            "assignee": {
                "id": "user-1",
                "name": "John Doe",
                "email": "john@example.com",
                "displayName": null,
                "active": true
            },
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-02T00:00:00.000Z"
        }"##;
        let issue: Issue = serde_json::from_str(json).unwrap();
        assert_eq!(issue.id, "issue-abc");
        assert_eq!(issue.identifier, "ENG-123");
        assert_eq!(issue.title, "Fix the bug");
        assert_eq!(issue.priority, 2);
        assert!(issue.state.is_some());
        assert!(issue.team.is_some());
        assert!(issue.assignee.is_some());
    }

    #[test]
    fn test_issue_with_null_optional_fields() {
        let json = r#"{
            "id": "issue-abc",
            "identifier": "ENG-123",
            "title": "Fix the bug",
            "description": null,
            "priority": 0,
            "state": null,
            "team": null,
            "assignee": null,
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-02T00:00:00.000Z"
        }"#;
        let issue: Issue = serde_json::from_str(json).unwrap();
        assert!(issue.description.is_none());
        assert!(issue.state.is_none());
        assert!(issue.team.is_none());
        assert!(issue.assignee.is_none());
    }

    #[test]
    fn test_issue_connection_deserialization() {
        let json = r#"{
            "nodes": [
                {
                    "id": "issue-1",
                    "identifier": "ENG-1",
                    "title": "Issue 1",
                    "description": null,
                    "priority": 1,
                    "state": null,
                    "team": null,
                    "assignee": null,
                    "createdAt": "2024-01-01T00:00:00.000Z",
                    "updatedAt": "2024-01-01T00:00:00.000Z"
                },
                {
                    "id": "issue-2",
                    "identifier": "ENG-2",
                    "title": "Issue 2",
                    "description": null,
                    "priority": 2,
                    "state": null,
                    "team": null,
                    "assignee": null,
                    "createdAt": "2024-01-02T00:00:00.000Z",
                    "updatedAt": "2024-01-02T00:00:00.000Z"
                }
            ]
        }"#;
        let connection: IssueConnection = serde_json::from_str(json).unwrap();
        assert_eq!(connection.nodes.len(), 2);
        assert_eq!(connection.nodes[0].identifier, "ENG-1");
        assert_eq!(connection.nodes[1].identifier, "ENG-2");
    }

    #[test]
    fn test_issue_create_response_deserialization() {
        let json = r#"{
            "issueCreate": {
                "success": true,
                "issue": {
                    "id": "issue-new",
                    "identifier": "ENG-999",
                    "title": "New Issue",
                    "description": null,
                    "priority": 0,
                    "state": null,
                    "team": null,
                    "assignee": null,
                    "createdAt": "2024-01-01T00:00:00.000Z",
                    "updatedAt": "2024-01-01T00:00:00.000Z"
                }
            }
        }"#;
        let response: IssueCreateResponse = serde_json::from_str(json).unwrap();
        assert!(response.issue_create.success);
        assert!(response.issue_create.issue.is_some());
        assert_eq!(response.issue_create.issue.unwrap().identifier, "ENG-999");
    }

    #[test]
    fn test_issue_update_response_deserialization() {
        let json = r#"{
            "issueUpdate": {
                "success": true,
                "issue": {
                    "id": "issue-abc",
                    "identifier": "ENG-123",
                    "title": "Updated Title",
                    "description": null,
                    "priority": 1,
                    "state": null,
                    "team": null,
                    "assignee": null,
                    "createdAt": "2024-01-01T00:00:00.000Z",
                    "updatedAt": "2024-01-02T00:00:00.000Z"
                }
            }
        }"#;
        let response: IssueUpdateResponse = serde_json::from_str(json).unwrap();
        assert!(response.issue_update.success);
        assert_eq!(response.issue_update.issue.unwrap().title, "Updated Title");
    }

    #[test]
    fn test_issue_search_response_deserialization() {
        let json = r#"{
            "issues": {
                "nodes": [
                    {
                        "id": "issue-1",
                        "identifier": "ENG-123",
                        "title": "Search result issue",
                        "description": "Found via search",
                        "priority": 2,
                        "state": null,
                        "team": null,
                        "assignee": null,
                        "createdAt": "2024-01-01T00:00:00.000Z",
                        "updatedAt": "2024-01-01T00:00:00.000Z"
                    }
                ]
            }
        }"#;
        let response: IssueSearchResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.issues.nodes.len(), 1);
        assert_eq!(response.issues.nodes[0].identifier, "ENG-123");
        assert_eq!(response.issues.nodes[0].title, "Search result issue");
    }

    #[test]
    fn test_issue_attachments_response_deserialization() {
        let json = r#"{
            "issue": {
                "id": "issue-123",
                "identifier": "ENG-456",
                "attachments": {
                    "nodes": [
                        {
                            "id": "attach-1",
                            "title": "File1.png",
                            "subtitle": null,
                            "url": "https://example.com/file1.png",
                            "metadata": null,
                            "createdAt": "2024-01-01T00:00:00.000Z",
                            "updatedAt": "2024-01-01T00:00:00.000Z",
                            "creator": null
                        }
                    ]
                }
            }
        }"#;
        let response: IssueAttachmentsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.issue.identifier, "ENG-456");
        assert_eq!(response.issue.attachments.nodes.len(), 1);
        assert_eq!(response.issue.attachments.nodes[0].title, "File1.png");
    }
}
