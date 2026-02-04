//! Domain model types for Linear API entities.
//!
//! These types map to Linear's GraphQL schema and are used for both
//! serialization (JSON output) and deserialization (API responses).

use serde::{Deserialize, Serialize};

// =============================================================================
// Core Domain Models
// =============================================================================

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

/// A Linear team.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    /// Unique identifier for the team.
    pub id: String,
    /// The team's key/prefix (e.g., "ENG").
    pub key: String,
    /// The team's name.
    pub name: String,
    /// Optional description of the team.
    pub description: Option<String>,
}

/// A workflow state for issues.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowState {
    /// Unique identifier for the state.
    pub id: String,
    /// The state's name (e.g., "In Progress", "Done").
    pub name: String,
    /// The state's color (hex color code).
    pub color: String,
    /// The type of state (backlog, unstarted, started, completed, canceled).
    /// Note: `type` is a reserved keyword in Rust, so we use `type_` with serde rename.
    #[serde(rename = "type")]
    pub type_: String,
}

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
}

// =============================================================================
// Connection Types (for paginated results)
// =============================================================================

/// A paginated list of issues.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueConnection {
    /// List of issues.
    pub nodes: Vec<Issue>,
}

/// A paginated list of teams.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamConnection {
    /// List of teams.
    pub nodes: Vec<Team>,
}

/// A paginated list of users.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserConnection {
    /// List of users.
    pub nodes: Vec<User>,
}

// =============================================================================
// GraphQL Response Wrappers
// =============================================================================

/// Response wrapper for the viewer query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewerResponse {
    /// The authenticated user.
    pub viewer: User,
}

/// Response wrapper for the teams query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamsResponse {
    /// Paginated list of teams.
    pub teams: TeamConnection,
}

/// Response wrapper for a single team query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamResponse {
    /// The requested team.
    pub team: Team,
}

/// Response wrapper for the users query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsersResponse {
    /// Paginated list of users.
    pub users: UserConnection,
}

/// Response wrapper for the issues query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssuesResponse {
    /// Paginated list of issues.
    pub issues: IssueConnection,
}

/// Response wrapper for a single issue query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueResponse {
    /// The requested issue.
    pub issue: Issue,
}

// =============================================================================
// Mutation Response Wrappers
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
    fn test_team_deserialization() {
        let json = r#"{
            "id": "team-456",
            "key": "ENG",
            "name": "Engineering",
            "description": "The engineering team"
        }"#;
        let team: Team = serde_json::from_str(json).unwrap();
        assert_eq!(team.id, "team-456");
        assert_eq!(team.key, "ENG");
        assert_eq!(team.name, "Engineering");
        assert_eq!(team.description, Some("The engineering team".to_string()));
    }

    #[test]
    fn test_team_with_null_description() {
        let json = r#"{
            "id": "team-456",
            "key": "ENG",
            "name": "Engineering",
            "description": null
        }"#;
        let team: Team = serde_json::from_str(json).unwrap();
        assert!(team.description.is_none());
    }

    #[test]
    fn test_workflow_state_deserialization() {
        let json = r##"{
            "id": "state-789",
            "name": "In Progress",
            "color": "#0066ff",
            "type": "started"
        }"##;
        let state: WorkflowState = serde_json::from_str(json).unwrap();
        assert_eq!(state.id, "state-789");
        assert_eq!(state.name, "In Progress");
        assert_eq!(state.color, "#0066ff");
        assert_eq!(state.type_, "started");
    }

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

    #[test]
    fn test_teams_response_deserialization() {
        let json = r#"{
            "teams": {
                "nodes": [
                    {
                        "id": "team-1",
                        "key": "ENG",
                        "name": "Engineering",
                        "description": null
                    }
                ]
            }
        }"#;
        let response: TeamsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.teams.nodes.len(), 1);
        assert_eq!(response.teams.nodes[0].key, "ENG");
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
}
