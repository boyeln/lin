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
}
