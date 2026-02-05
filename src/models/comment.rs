//! Comment-related types for the Linear API.
//!
//! This module contains types for representing Linear issue comments and
//! comment-related API responses.

use serde::{Deserialize, Serialize};

use super::user::User;

/// A comment on a Linear issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    /// Unique identifier for the comment.
    pub id: String,
    /// The comment body/content.
    pub body: String,
    /// The user who created the comment.
    pub user: Option<User>,
    /// ISO 8601 timestamp of when the comment was created.
    pub created_at: String,
    /// ISO 8601 timestamp of when the comment was last updated.
    pub updated_at: String,
}

/// A paginated list of comments.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentConnection {
    /// List of comments.
    pub nodes: Vec<Comment>,
}

/// Response for comment creation mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentCreatePayload {
    /// Whether the mutation was successful.
    pub success: bool,
    /// The created comment.
    pub comment: Option<Comment>,
}

/// Response wrapper for comment creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentCreateResponse {
    /// The mutation payload.
    pub comment_create: CommentCreatePayload,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_deserialization() {
        let json = r#"{
            "id": "comment-123",
            "body": "This is a comment",
            "user": {
                "id": "user-1",
                "name": "John Doe",
                "email": "john@example.com",
                "displayName": "JD",
                "active": true
            },
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-01T00:00:00.000Z"
        }"#;
        let comment: Comment = serde_json::from_str(json).unwrap();
        assert_eq!(comment.id, "comment-123");
        assert_eq!(comment.body, "This is a comment");
        assert!(comment.user.is_some());
    }

    #[test]
    fn test_comment_with_null_user() {
        let json = r#"{
            "id": "comment-456",
            "body": "Anonymous comment",
            "user": null,
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-01T00:00:00.000Z"
        }"#;
        let comment: Comment = serde_json::from_str(json).unwrap();
        assert!(comment.user.is_none());
    }
}
