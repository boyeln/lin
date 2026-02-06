//! Team-related types for the Linear API.
//!
//! This module contains types for representing Linear teams and
//! team-related API responses.

use serde::{Deserialize, Serialize};

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
    /// Issue estimation type (e.g., "linear", "fibonacci", "tShirt", "exponential", "notUsed").
    #[serde(rename = "issueEstimationType")]
    pub issue_estimate_type: Option<String>,
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
            "issueEstimationType": "linear"
        }"#;
        let team: Team = serde_json::from_str(json).unwrap();
        assert_eq!(team.id, "team-456");
        assert_eq!(team.key, "ENG");
        assert_eq!(team.name, "Engineering");
        assert_eq!(team.description, Some("The engineering team".to_string()));
        assert_eq!(team.issue_estimate_type, Some("linear".to_string()));
    }

    #[test]
    fn test_team_with_null_description() {
        let json = r#"{
            "id": "team-456",
            "key": "ENG",
            "name": "Engineering",
            "description": null,
            "issueEstimationType": "notUsed"
        }"#;
        let team: Team = serde_json::from_str(json).unwrap();
        assert!(team.description.is_none());
        assert_eq!(team.issue_estimate_type, Some("notUsed".to_string()));
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
                        "issueEstimationType": "fibonacci"
                    }
                ]
            }
        }"#;
        let response: TeamsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.teams.nodes.len(), 1);
        assert_eq!(response.teams.nodes[0].key, "ENG");
        assert_eq!(
            response.teams.nodes[0].issue_estimate_type,
            Some("fibonacci".to_string())
        );
    }

    #[test]
    fn test_team_with_tshirt_estimates() {
        let json = r#"{
            "id": "team-456",
            "key": "ENG",
            "name": "Engineering",
            "description": null,
            "issueEstimationType": "tShirt"
        }"#;
        let team: Team = serde_json::from_str(json).unwrap();
        assert_eq!(team.issue_estimate_type, Some("tShirt".to_string()));
    }

    #[test]
    fn test_team_with_fibonacci_estimates() {
        let json = r#"{
            "id": "team-789",
            "key": "PROD",
            "name": "Product",
            "description": null,
            "issueEstimationType": "fibonacci"
        }"#;
        let team: Team = serde_json::from_str(json).unwrap();
        assert_eq!(team.issue_estimate_type, Some("fibonacci".to_string()));
    }
}
