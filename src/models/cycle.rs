//! Cycle-related types for the Linear API.
//!
//! This module contains types for representing Linear cycles (sprints) and
//! cycle-related API responses.

use serde::{Deserialize, Serialize};

use super::issue::IssueConnection;

/// A Linear cycle (sprint).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cycle {
    /// Unique identifier for the cycle.
    pub id: String,
    /// The cycle's number within the team.
    pub number: i32,
    /// The cycle's name (optional).
    pub name: Option<String>,
    /// Optional description of the cycle.
    pub description: Option<String>,
    /// ISO 8601 timestamp of when the cycle starts.
    pub starts_at: Option<String>,
    /// ISO 8601 timestamp of when the cycle ends.
    pub ends_at: Option<String>,
    /// ISO 8601 timestamp of when the cycle was completed (optional).
    pub completed_at: Option<String>,
    /// Progress percentage of the cycle (0-100).
    pub progress: f64,
    /// Completed scope history for the cycle.
    pub completed_scope_history: Vec<f64>,
    /// Scope history for the cycle.
    pub scope_history: Vec<f64>,
}

/// A paginated list of cycles.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CycleConnection {
    /// List of cycles.
    pub nodes: Vec<Cycle>,
}

/// A Linear cycle with its issues included.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CycleWithIssues {
    /// Unique identifier for the cycle.
    pub id: String,
    /// The cycle's number within the team.
    pub number: i32,
    /// The cycle's name (optional).
    pub name: Option<String>,
    /// Optional description of the cycle.
    pub description: Option<String>,
    /// ISO 8601 timestamp of when the cycle starts.
    pub starts_at: Option<String>,
    /// ISO 8601 timestamp of when the cycle ends.
    pub ends_at: Option<String>,
    /// ISO 8601 timestamp of when the cycle was completed (optional).
    pub completed_at: Option<String>,
    /// Progress percentage of the cycle (0-100).
    pub progress: f64,
    /// Completed scope history for the cycle.
    pub completed_scope_history: Vec<f64>,
    /// Scope history for the cycle.
    pub scope_history: Vec<f64>,
    /// Issues in this cycle.
    pub issues: IssueConnection,
}

/// Team with cycles for the cycles query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamWithCycles {
    /// Unique identifier for the team.
    pub id: String,
    /// The team's cycles.
    pub cycles: CycleConnection,
}

/// Response wrapper for the cycles query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CyclesResponse {
    /// The team with its cycles.
    pub team: TeamWithCycles,
}

/// Response wrapper for a single cycle query (with issues).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CycleResponse {
    /// The requested cycle with its issues.
    pub cycle: CycleWithIssues,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cycle_deserialization() {
        let json = r#"{
            "id": "cycle-123",
            "number": 5,
            "name": "Sprint 5",
            "description": "Q1 Sprint",
            "startsAt": "2024-01-01T00:00:00.000Z",
            "endsAt": "2024-01-14T00:00:00.000Z",
            "completedAt": null,
            "progress": 50.5,
            "completedScopeHistory": [0.0, 10.0, 25.0],
            "scopeHistory": [100.0, 100.0, 100.0]
        }"#;
        let cycle: Cycle = serde_json::from_str(json).unwrap();
        assert_eq!(cycle.id, "cycle-123");
        assert_eq!(cycle.number, 5);
        assert_eq!(cycle.name, Some("Sprint 5".to_string()));
        assert_eq!(cycle.description, Some("Q1 Sprint".to_string()));
        assert!(cycle.starts_at.is_some());
        assert!(cycle.ends_at.is_some());
        assert!(cycle.completed_at.is_none());
        assert!((cycle.progress - 50.5).abs() < 0.001);
        assert_eq!(cycle.completed_scope_history.len(), 3);
        assert_eq!(cycle.scope_history.len(), 3);
    }

    #[test]
    fn test_cycle_with_null_optional_fields() {
        let json = r#"{
            "id": "cycle-456",
            "number": 1,
            "name": null,
            "description": null,
            "startsAt": null,
            "endsAt": null,
            "completedAt": null,
            "progress": 0.0,
            "completedScopeHistory": [],
            "scopeHistory": []
        }"#;
        let cycle: Cycle = serde_json::from_str(json).unwrap();
        assert_eq!(cycle.id, "cycle-456");
        assert_eq!(cycle.number, 1);
        assert!(cycle.name.is_none());
        assert!(cycle.description.is_none());
        assert!(cycle.starts_at.is_none());
        assert!(cycle.ends_at.is_none());
    }

    #[test]
    fn test_cycles_response_deserialization() {
        let json = r#"{
            "team": {
                "id": "team-123",
                "cycles": {
                    "nodes": [
                        {
                            "id": "cycle-1",
                            "number": 1,
                            "name": "Sprint 1",
                            "description": null,
                            "startsAt": "2024-01-01T00:00:00.000Z",
                            "endsAt": "2024-01-14T00:00:00.000Z",
                            "completedAt": null,
                            "progress": 75.0,
                            "completedScopeHistory": [],
                            "scopeHistory": []
                        }
                    ]
                }
            }
        }"#;
        let response: CyclesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.team.id, "team-123");
        assert_eq!(response.team.cycles.nodes.len(), 1);
        assert_eq!(response.team.cycles.nodes[0].number, 1);
    }
}
