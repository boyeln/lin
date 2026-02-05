//! User-related types for the Linear API.
//!
//! This module contains types for representing Linear users and
//! user-related API responses.

use serde::{Deserialize, Serialize};

/// A Linear user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    /// Unique identifier for the user.
    pub id: String,
    /// The user's full name.
    pub name: String,
    /// The user's email address.
    pub email: String,
    /// The user's display name (may differ from full name).
    pub display_name: Option<String>,
    /// Whether the user account is active.
    pub active: bool,
}

/// A paginated list of users.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserConnection {
    /// List of users.
    pub nodes: Vec<User>,
}

/// Response wrapper for the viewer query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewerResponse {
    /// The authenticated user.
    pub viewer: User,
}

/// Response wrapper for the users query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsersResponse {
    /// Paginated list of users.
    pub users: UserConnection,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_deserialization() {
        let json = r#"{
            "id": "user-123",
            "name": "John Doe",
            "email": "john@example.com",
            "displayName": "JD",
            "active": true
        }"#;
        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.id, "user-123");
        assert_eq!(user.name, "John Doe");
        assert_eq!(user.email, "john@example.com");
        assert_eq!(user.display_name, Some("JD".to_string()));
        assert!(user.active);
    }

    #[test]
    fn test_user_serialization() {
        let user = User {
            id: "user-123".to_string(),
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            display_name: Some("JD".to_string()),
            active: true,
        };
        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("\"id\":\"user-123\""));
        assert!(json.contains("\"displayName\":\"JD\""));
    }

    #[test]
    fn test_viewer_response_deserialization() {
        let json = r#"{
            "viewer": {
                "id": "user-123",
                "name": "Test User",
                "email": "test@example.com",
                "displayName": "TU",
                "active": true
            }
        }"#;
        let response: ViewerResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.viewer.id, "user-123");
        assert_eq!(response.viewer.name, "Test User");
    }
}
