//! Team-related types for the Linear API.
//!
//! This module contains types for representing Linear teams and
//! team-related API responses.

use serde::{Deserialize, Serialize};

/// Issue estimate type configuration for a team.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueEstimationType {
    /// The estimate type ID (e.g., "linear", "fibonacci", "tshirt", "exponential", "none").
    pub id: String,
    /// Display name of the estimate type.
    pub name: String,
    /// Numeric values for the estimate scale.
    pub values: Vec<f64>,
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
    /// Issue estimate type configuration.
    #[serde(rename = "issueEstimationType")]
    pub issue_estimate_type: Option<IssueEstimationType>,
}

/// Basic team information (ID and key only).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamBasic {
    /// Unique identifier for the team.
    pub id: String,
    /// The team's key/prefix (e.g., "ENG").
    pub key: String,
}

/// A paginated list of teams.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamConnection {
    /// List of teams.
    pub nodes: Vec<Team>,
}

/// Response wrapper for a single team query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamResponse {
    /// The requested team.
    pub team: Team,
}

/// Response wrapper for the teams query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamsResponse {
    /// Paginated list of teams.
    pub teams: TeamConnection,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_deserialization() {
        let json = r#"{
            "id": "team-456",
            "key": "ENG",
            "name": "Engineering",
            "description": "The engineering team",
            "issueEstimationType": null
        }"#;
        let team: Team = serde_json::from_str(json).unwrap();
        assert_eq!(team.id, "team-456");
        assert_eq!(team.key, "ENG");
        assert_eq!(team.name, "Engineering");
        assert_eq!(team.description, Some("The engineering team".to_string()));
        assert!(team.issue_estimate_type.is_none());
    }

    #[test]
    fn test_team_with_null_description() {
        let json = r#"{
            "id": "team-456",
            "key": "ENG",
            "name": "Engineering",
            "description": null,
            "issueEstimationType": null
        }"#;
        let team: Team = serde_json::from_str(json).unwrap();
        assert!(team.description.is_none());
        assert!(team.issue_estimate_type.is_none());
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
                        "description": null,
                        "issueEstimationType": null
                    }
                ]
            }
        }"#;
        let response: TeamsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.teams.nodes.len(), 1);
        assert_eq!(response.teams.nodes[0].key, "ENG");
    }

    #[test]
    fn test_team_with_tshirt_estimates() {
        let json = r#"{
            "id": "team-456",
            "key": "ENG",
            "name": "Engineering",
            "description": null,
            "issueEstimationType": {
                "id": "tshirt",
                "name": "T-Shirt Sizes",
                "values": [1, 2, 3, 5, 8]
            }
        }"#;
        let team: Team = serde_json::from_str(json).unwrap();
        let est_type = team.issue_estimate_type.unwrap();
        assert_eq!(est_type.id, "tshirt");
        assert_eq!(est_type.name, "T-Shirt Sizes");
        assert_eq!(est_type.values, vec![1.0, 2.0, 3.0, 5.0, 8.0]);
    }

    #[test]
    fn test_team_with_fibonacci_estimates() {
        let json = r#"{
            "id": "team-789",
            "key": "PROD",
            "name": "Product",
            "description": null,
            "issueEstimationType": {
                "id": "fibonacci",
                "name": "Fibonacci",
                "values": [1, 2, 3, 5, 8, 13, 21]
            }
        }"#;
        let team: Team = serde_json::from_str(json).unwrap();
        let est_type = team.issue_estimate_type.unwrap();
        assert_eq!(est_type.id, "fibonacci");
        assert_eq!(est_type.values, vec![1.0, 2.0, 3.0, 5.0, 8.0, 13.0, 21.0]);
    }
}
