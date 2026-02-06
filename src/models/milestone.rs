//! Project milestone-related types for the Linear API.
//!
//! This module contains types for representing Linear project milestones and
//! milestone-related API responses.

use serde::{Deserialize, Serialize};

/// A Linear project milestone.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMilestone {
    /// Unique identifier for the milestone.
    pub id: String,
    /// The milestone's name.
    pub name: String,
    /// Optional description of the milestone.
    pub description: Option<String>,
    /// Target date for the milestone (optional).
    pub target_date: Option<String>,
    /// Sort order of the milestone.
    pub sort_order: f64,
    /// The milestone's status (done, next, overdue, unstarted).
    pub status: String,
    /// Progress percentage of the milestone (0-100).
    pub progress: f64,
    /// ISO 8601 timestamp of when the milestone was created.
    pub created_at: String,
    /// ISO 8601 timestamp of when the milestone was last updated.
    pub updated_at: String,
}

/// A paginated list of project milestones.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMilestoneConnection {
    /// List of milestones.
    pub nodes: Vec<ProjectMilestone>,
}

/// Response wrapper for a single milestone query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMilestoneResponse {
    /// The requested milestone.
    pub project_milestone: ProjectMilestone,
}

/// Response wrapper for the project milestones query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMilestonesResponse {
    /// Paginated list of milestones.
    pub project_milestones: ProjectMilestoneConnection,
}

/// Response wrapper for milestone mutation operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMilestonePayload {
    /// Indicates if the operation was successful.
    pub success: bool,
    /// The created or updated milestone.
    pub project_milestone: Option<ProjectMilestone>,
}

/// Response wrapper for milestone create mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMilestoneCreateResponse {
    /// The mutation payload.
    pub project_milestone_create: ProjectMilestonePayload,
}

/// Response wrapper for milestone update mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMilestoneUpdateResponse {
    /// The mutation payload.
    pub project_milestone_update: ProjectMilestonePayload,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_milestone_deserialization() {
        let json = r#"{
            "id": "milestone-123",
            "name": "Sprint 1",
            "description": "First sprint milestone",
            "targetDate": "2024-03-31",
            "sortOrder": 100.0,
            "status": "next",
            "progress": 45.5,
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-15T00:00:00.000Z"
        }"#;
        let milestone: ProjectMilestone = serde_json::from_str(json).unwrap();
        assert_eq!(milestone.id, "milestone-123");
        assert_eq!(milestone.name, "Sprint 1");
        assert_eq!(
            milestone.description,
            Some("First sprint milestone".to_string())
        );
        assert_eq!(milestone.target_date, Some("2024-03-31".to_string()));
        assert!((milestone.sort_order - 100.0).abs() < 0.001);
        assert_eq!(milestone.status, "next");
        assert!((milestone.progress - 45.5).abs() < 0.001);
    }

    #[test]
    fn test_milestone_with_null_optional_fields() {
        let json = r#"{
            "id": "milestone-456",
            "name": "Simple Milestone",
            "description": null,
            "targetDate": null,
            "sortOrder": 0.0,
            "status": "unstarted",
            "progress": 0.0,
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-01T00:00:00.000Z"
        }"#;
        let milestone: ProjectMilestone = serde_json::from_str(json).unwrap();
        assert!(milestone.description.is_none());
        assert!(milestone.target_date.is_none());
    }

    #[test]
    fn test_milestones_response_deserialization() {
        let json = r#"{
            "projectMilestones": {
                "nodes": [
                    {
                        "id": "milestone-1",
                        "name": "Milestone Alpha",
                        "description": null,
                        "targetDate": null,
                        "sortOrder": 1.0,
                        "status": "unstarted",
                        "progress": 0.0,
                        "createdAt": "2024-01-01T00:00:00.000Z",
                        "updatedAt": "2024-01-15T00:00:00.000Z"
                    }
                ]
            }
        }"#;
        let response: ProjectMilestonesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.project_milestones.nodes.len(), 1);
        assert_eq!(response.project_milestones.nodes[0].name, "Milestone Alpha");
    }

    #[test]
    fn test_milestone_payload_deserialization() {
        let json = r#"{
            "success": true,
            "projectMilestone": {
                "id": "milestone-789",
                "name": "Created Milestone",
                "description": null,
                "targetDate": "2024-06-30",
                "sortOrder": 50.0,
                "status": "unstarted",
                "progress": 0.0,
                "createdAt": "2024-02-01T00:00:00.000Z",
                "updatedAt": "2024-02-01T00:00:00.000Z"
            }
        }"#;
        let payload: ProjectMilestonePayload = serde_json::from_str(json).unwrap();
        assert!(payload.success);
        assert!(payload.project_milestone.is_some());
        assert_eq!(payload.project_milestone.unwrap().id, "milestone-789");
    }
}
