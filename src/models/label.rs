//! Label-related types for the Linear API.
//!
//! This module contains types for representing Linear labels and
//! label-related API responses.

use serde::{Deserialize, Serialize};

/// A Linear label.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Label {
    /// Unique identifier for the label.
    pub id: String,
    /// The label's name.
    pub name: String,
    /// The label's description (optional).
    pub description: Option<String>,
    /// The label's color (hex color code).
    pub color: String,
    /// Whether this is a group label (parent label).
    pub is_group: bool,
    /// ISO 8601 timestamp of when the label was created.
    pub created_at: String,
    /// ISO 8601 timestamp of when the label was last updated.
    pub updated_at: String,
}

/// A paginated list of labels.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelConnection {
    /// List of labels.
    pub nodes: Vec<Label>,
}

/// Team with labels for the team labels query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamWithLabels {
    /// Unique identifier for the team.
    pub id: String,
    /// The team's labels.
    pub labels: LabelConnection,
}

/// Response wrapper for labels query (workspace-level).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelsResponse {
    /// Paginated list of labels.
    pub issue_labels: LabelConnection,
}

/// Response wrapper for team labels query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamLabelsResponse {
    /// The team with its labels.
    pub team: TeamWithLabels,
}

/// Response wrapper for a single label query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelResponse {
    /// The requested label.
    pub issue_label: Label,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_deserialization() {
        let json = r##"{
            "id": "label-123",
            "name": "Bug",
            "description": "Bug reports",
            "color": "#ff0000",
            "isGroup": false,
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-15T00:00:00.000Z"
        }"##;
        let label: Label = serde_json::from_str(json).unwrap();
        assert_eq!(label.id, "label-123");
        assert_eq!(label.name, "Bug");
        assert_eq!(label.description, Some("Bug reports".to_string()));
        assert_eq!(label.color, "#ff0000");
        assert!(!label.is_group);
    }

    #[test]
    fn test_label_with_null_description() {
        let json = r##"{
            "id": "label-456",
            "name": "Feature",
            "description": null,
            "color": "#00ff00",
            "isGroup": true,
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-01T00:00:00.000Z"
        }"##;
        let label: Label = serde_json::from_str(json).unwrap();
        assert_eq!(label.id, "label-456");
        assert_eq!(label.name, "Feature");
        assert!(label.description.is_none());
        assert!(label.is_group);
    }

    #[test]
    fn test_labels_response_deserialization() {
        let json = r##"{
            "issueLabels": {
                "nodes": [
                    {
                        "id": "label-1",
                        "name": "Bug",
                        "description": null,
                        "color": "#ff0000",
                        "isGroup": false,
                        "createdAt": "2024-01-01T00:00:00.000Z",
                        "updatedAt": "2024-01-01T00:00:00.000Z"
                    },
                    {
                        "id": "label-2",
                        "name": "Feature",
                        "description": "Feature requests",
                        "color": "#00ff00",
                        "isGroup": false,
                        "createdAt": "2024-01-02T00:00:00.000Z",
                        "updatedAt": "2024-01-02T00:00:00.000Z"
                    }
                ]
            }
        }"##;
        let response: LabelsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.issue_labels.nodes.len(), 2);
        assert_eq!(response.issue_labels.nodes[0].name, "Bug");
        assert_eq!(response.issue_labels.nodes[1].name, "Feature");
    }

    #[test]
    fn test_label_response_deserialization() {
        let json = r##"{
            "issueLabel": {
                "id": "label-123",
                "name": "Bug",
                "description": "Bug reports",
                "color": "#ff0000",
                "isGroup": false,
                "createdAt": "2024-01-01T00:00:00.000Z",
                "updatedAt": "2024-01-15T00:00:00.000Z"
            }
        }"##;
        let response: LabelResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.issue_label.id, "label-123");
        assert_eq!(response.issue_label.name, "Bug");
    }
}
