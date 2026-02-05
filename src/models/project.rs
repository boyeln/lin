//! Project-related types for the Linear API.
//!
//! This module contains types for representing Linear projects and
//! project-related API responses.

use serde::{Deserialize, Serialize};

/// A Linear project.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    /// Unique identifier for the project.
    pub id: String,
    /// The project's name.
    pub name: String,
    /// Optional description of the project.
    pub description: Option<String>,
    /// The project's state (planned, started, paused, completed, canceled).
    pub state: String,
    /// ISO 8601 timestamp of when the project was created.
    pub created_at: String,
    /// ISO 8601 timestamp of when the project was last updated.
    pub updated_at: String,
    /// Target date for the project (optional).
    pub target_date: Option<String>,
    /// Start date for the project (optional).
    pub start_date: Option<String>,
    /// Progress percentage of the project (0-100).
    pub progress: f64,
}

/// A paginated list of projects.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConnection {
    /// List of projects.
    pub nodes: Vec<Project>,
}

/// Response wrapper for a single project query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectResponse {
    /// The requested project.
    pub project: Project,
}

/// Response wrapper for the projects query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectsResponse {
    /// Paginated list of projects.
    pub projects: ProjectConnection,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_deserialization() {
        let json = r#"{
            "id": "project-123",
            "name": "Q1 Roadmap",
            "description": "Quarterly roadmap project",
            "state": "started",
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-15T00:00:00.000Z",
            "targetDate": "2024-03-31",
            "startDate": "2024-01-01",
            "progress": 25.5
        }"#;
        let project: Project = serde_json::from_str(json).unwrap();
        assert_eq!(project.id, "project-123");
        assert_eq!(project.name, "Q1 Roadmap");
        assert_eq!(
            project.description,
            Some("Quarterly roadmap project".to_string())
        );
        assert_eq!(project.state, "started");
        assert_eq!(project.target_date, Some("2024-03-31".to_string()));
        assert_eq!(project.start_date, Some("2024-01-01".to_string()));
        assert!((project.progress - 25.5).abs() < 0.001);
    }

    #[test]
    fn test_project_with_null_optional_fields() {
        let json = r#"{
            "id": "project-456",
            "name": "Simple Project",
            "description": null,
            "state": "planned",
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-01T00:00:00.000Z",
            "targetDate": null,
            "startDate": null,
            "progress": 0.0
        }"#;
        let project: Project = serde_json::from_str(json).unwrap();
        assert!(project.description.is_none());
        assert!(project.target_date.is_none());
        assert!(project.start_date.is_none());
    }

    #[test]
    fn test_projects_response_deserialization() {
        let json = r#"{
            "projects": {
                "nodes": [
                    {
                        "id": "project-1",
                        "name": "Project Alpha",
                        "description": null,
                        "state": "started",
                        "createdAt": "2024-01-01T00:00:00.000Z",
                        "updatedAt": "2024-01-15T00:00:00.000Z",
                        "targetDate": null,
                        "startDate": null,
                        "progress": 50.0
                    }
                ]
            }
        }"#;
        let response: ProjectsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.projects.nodes.len(), 1);
        assert_eq!(response.projects.nodes[0].name, "Project Alpha");
    }
}
