//! Workflow state types for the Linear API.
//!
//! This module contains types for representing Linear workflow states
//! (e.g., "In Progress", "Done") and workflow-related API responses.

use serde::{Deserialize, Serialize};

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

/// A paginated list of workflow states.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowStateConnection {
    /// List of workflow states.
    pub nodes: Vec<WorkflowState>,
}

/// Team with workflow states for the workflow states query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamWithWorkflowStates {
    /// Unique identifier for the team.
    pub id: String,
    /// The team's workflow states.
    pub states: WorkflowStateConnection,
}

/// Response wrapper for the workflow states query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowStatesResponse {
    /// The team with its workflow states.
    pub team: TeamWithWorkflowStates,
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
