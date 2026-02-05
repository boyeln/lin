//! GraphQL query and mutation strings for Linear's API.
//!
//! This module contains all the GraphQL queries and mutations used by the CLI.
//! Each query is defined as a constant string that can be passed to the
//! [`GraphQLClient::query`](super::client::GraphQLClient::query) method.
//!
//! Queries are organized by domain:
//! - [`organization`] - Viewer and organization queries
//! - [`team`] - Team queries
//! - [`user`] - User queries
//! - [`project`] - Project queries
//! - [`issue`] - Issue queries and mutations
//! - [`comment`] - Comment mutations
//! - [`cycle`] - Cycle/sprint queries
//! - [`label`] - Label queries
//! - [`document`] - Document queries and mutations
//! - [`attachment`] - Attachment queries and mutations
//! - [`workflow`] - Workflow state queries
//! - [`search`] - Search queries

pub mod attachment;
pub mod comment;
pub mod cycle;
pub mod document;
pub mod issue;
pub mod label;
pub mod organization;
pub mod project;
pub mod search;
pub mod team;
pub mod user;
pub mod workflow;

#[cfg(test)]
mod tests {
    use super::organization::VIEWER_QUERY;
    use super::project::{PROJECT_QUERY, PROJECTS_QUERY};
    use super::team::{TEAM_QUERY, TEAMS_QUERY};
    use super::user::USERS_QUERY;
    use super::workflow::WORKFLOW_STATES_QUERY;

    #[test]
    fn test_viewer_query_is_valid() {
        assert!(VIEWER_QUERY.contains("query Viewer"));
        assert!(VIEWER_QUERY.contains("viewer"));
        assert!(VIEWER_QUERY.contains("id"));
        assert!(VIEWER_QUERY.contains("name"));
        assert!(VIEWER_QUERY.contains("email"));
    }

    #[test]
    fn test_teams_query_is_valid() {
        assert!(TEAMS_QUERY.contains("query Teams"));
        assert!(TEAMS_QUERY.contains("$first: Int"));
        assert!(TEAMS_QUERY.contains("nodes"));
        assert!(TEAMS_QUERY.contains("key"));
    }

    #[test]
    fn test_team_query_is_valid() {
        assert!(TEAM_QUERY.contains("query Team"));
        assert!(TEAM_QUERY.contains("$id: String!"));
        assert!(TEAM_QUERY.contains("team(id: $id)"));
    }

    #[test]
    fn test_workflow_states_query_is_valid() {
        assert!(WORKFLOW_STATES_QUERY.contains("query WorkflowStates"));
        assert!(WORKFLOW_STATES_QUERY.contains("$id: String!"));
        assert!(WORKFLOW_STATES_QUERY.contains("team(id: $id)"));
        assert!(WORKFLOW_STATES_QUERY.contains("states"));
        assert!(WORKFLOW_STATES_QUERY.contains("nodes"));
        assert!(WORKFLOW_STATES_QUERY.contains("type"));
    }

    #[test]
    fn test_users_query_is_valid() {
        assert!(USERS_QUERY.contains("query Users"));
        assert!(USERS_QUERY.contains("users"));
        assert!(USERS_QUERY.contains("nodes"));
    }

    #[test]
    fn test_projects_query_is_valid() {
        assert!(PROJECTS_QUERY.contains("query Projects"));
        assert!(PROJECTS_QUERY.contains("$first: Int"));
        assert!(PROJECTS_QUERY.contains("$filter: ProjectFilter"));
        assert!(PROJECTS_QUERY.contains("projects"));
        assert!(PROJECTS_QUERY.contains("nodes"));
        assert!(PROJECTS_QUERY.contains("progress"));
    }

    #[test]
    fn test_project_query_is_valid() {
        assert!(PROJECT_QUERY.contains("query Project"));
        assert!(PROJECT_QUERY.contains("$id: String!"));
        assert!(PROJECT_QUERY.contains("project(id: $id)"));
        assert!(PROJECT_QUERY.contains("progress"));
    }
}
