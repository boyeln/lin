//! Workflow state-related GraphQL queries.

/// Query to list workflow states for a team.
///
/// Variables:
/// - `id` (String!): The team's unique identifier (UUID or key)
///
/// Returns: `WorkflowStatesResponse`
pub const WORKFLOW_STATES_QUERY: &str = r#"
query WorkflowStates($id: String!) {
    team(id: $id) {
        id
        states {
            nodes {
                id
                name
                color
                type
            }
        }
    }
}
"#;
