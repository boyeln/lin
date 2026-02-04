//! GraphQL query and mutation strings for Linear's API.
//!
//! This module contains all the GraphQL queries and mutations used by the CLI.
//! Each query is defined as a constant string that can be passed to the
//! [`GraphQLClient::query`](super::client::GraphQLClient::query) method.

/// Query to get the current authenticated user's information.
///
/// Returns: `ViewerResponse`
pub const VIEWER_QUERY: &str = r#"
query Viewer {
    viewer {
        id
        name
        email
        displayName
        active
    }
}
"#;

/// Query to list all teams in the organization.
///
/// Variables:
/// - `first` (Int, optional): Number of teams to fetch (default: 50)
///
/// Returns: `TeamsResponse`
pub const TEAMS_QUERY: &str = r#"
query Teams($first: Int) {
    teams(first: $first) {
        nodes {
            id
            key
            name
            description
        }
    }
}
"#;

/// Query to get a single team by ID.
///
/// Variables:
/// - `id` (String!): The team's unique identifier
///
/// Returns: `TeamResponse`
pub const TEAM_QUERY: &str = r#"
query Team($id: String!) {
    team(id: $id) {
        id
        key
        name
        description
    }
}
"#;

/// Query to list users in the organization.
///
/// Variables:
/// - `first` (Int, optional): Number of users to fetch (default: 50)
///
/// Returns: `UsersResponse`
pub const USERS_QUERY: &str = r#"
query Users($first: Int) {
    users(first: $first) {
        nodes {
            id
            name
            email
            displayName
            active
        }
    }
}
"#;

/// Query to list issues with optional filters.
///
/// Variables:
/// - `first` (Int, optional): Number of issues to fetch
/// - `teamId` (ID, optional): Filter by team ID
/// - `assigneeId` (ID, optional): Filter by assignee ID
/// - `stateId` (ID, optional): Filter by state ID
///
/// Returns: `IssuesResponse`
pub const ISSUES_QUERY: &str = r#"
query Issues($first: Int, $filter: IssueFilter) {
    issues(first: $first, filter: $filter) {
        nodes {
            id
            identifier
            title
            description
            priority
            createdAt
            updatedAt
            state {
                id
                name
                color
                type
            }
            team {
                id
                key
                name
                description
            }
            assignee {
                id
                name
                email
                displayName
                active
            }
        }
    }
}
"#;

/// Query to get a single issue by ID.
///
/// Variables:
/// - `id` (String!): The issue's unique identifier
///
/// Returns: `IssueResponse`
pub const ISSUE_QUERY: &str = r#"
query Issue($id: String!) {
    issue(id: $id) {
        id
        identifier
        title
        description
        priority
        createdAt
        updatedAt
        state {
            id
            name
            color
            type
        }
        team {
            id
            key
            name
            description
        }
        assignee {
            id
            name
            email
            displayName
            active
        }
    }
}
"#;

/// Query to get an issue by its identifier (e.g., "ENG-123").
///
/// This uses the `issueSearch` query with an exact filter to find issues
/// by their human-readable identifier.
///
/// Variables:
/// - `filter` (IssueFilter!): Filter containing the identifier
///
/// Returns: `IssuesResponse` (with single result)
pub const ISSUE_BY_IDENTIFIER_QUERY: &str = r#"
query IssueByIdentifier($filter: IssueFilter!) {
    issues(filter: $filter, first: 1) {
        nodes {
            id
            identifier
            title
            description
            priority
            createdAt
            updatedAt
            state {
                id
                name
                color
                type
            }
            team {
                id
                key
                name
                description
            }
            assignee {
                id
                name
                email
                displayName
                active
            }
        }
    }
}
"#;

/// Mutation to create a new issue.
///
/// Variables:
/// - `input` (IssueCreateInput!): Issue creation input containing:
///   - `title` (String!): Issue title
///   - `teamId` (String!): Team ID
///   - `description` (String, optional): Issue description
///   - `priority` (Int, optional): Priority (0-4)
///   - `assigneeId` (String, optional): Assignee user ID
///   - `stateId` (String, optional): Initial state ID
///
/// Returns: `IssueCreateResponse`
pub const ISSUE_CREATE_MUTATION: &str = r#"
mutation IssueCreate($input: IssueCreateInput!) {
    issueCreate(input: $input) {
        success
        issue {
            id
            identifier
            title
            description
            priority
            createdAt
            updatedAt
            state {
                id
                name
                color
                type
            }
            team {
                id
                key
                name
                description
            }
            assignee {
                id
                name
                email
                displayName
                active
            }
        }
    }
}
"#;

/// Mutation to update an existing issue.
///
/// Variables:
/// - `id` (String!): The issue's unique identifier
/// - `input` (IssueUpdateInput!): Issue update input containing:
///   - `title` (String, optional): New title
///   - `description` (String, optional): New description
///   - `priority` (Int, optional): New priority (0-4)
///   - `assigneeId` (String, optional): New assignee user ID
///   - `stateId` (String, optional): New state ID
///
/// Returns: `IssueUpdateResponse`
pub const ISSUE_UPDATE_MUTATION: &str = r#"
mutation IssueUpdate($id: String!, $input: IssueUpdateInput!) {
    issueUpdate(id: $id, input: $input) {
        success
        issue {
            id
            identifier
            title
            description
            priority
            createdAt
            updatedAt
            state {
                id
                name
                color
                type
            }
            team {
                id
                key
                name
                description
            }
            assignee {
                id
                name
                email
                displayName
                active
            }
        }
    }
}
"#;

/// Mutation to delete an issue.
///
/// Variables:
/// - `id` (String!): The issue's unique identifier
///
/// Returns: `IssueDeleteResponse`
pub const ISSUE_DELETE_MUTATION: &str = r#"
mutation IssueDelete($id: String!) {
    issueDelete(id: $id) {
        success
    }
}
"#;

/// Mutation to archive an issue.
///
/// Variables:
/// - `id` (String!): The issue's unique identifier
///
/// Returns: `IssueArchiveResponse`
pub const ISSUE_ARCHIVE_MUTATION: &str = r#"
mutation IssueArchive($id: String!) {
    issueArchive(id: $id) {
        success
    }
}
"#;

/// Mutation to unarchive an issue.
///
/// Variables:
/// - `id` (String!): The issue's unique identifier
///
/// Returns: `IssueUnarchiveResponse`
pub const ISSUE_UNARCHIVE_MUTATION: &str = r#"
mutation IssueUnarchive($id: String!) {
    issueUnarchive(id: $id) {
        success
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_users_query_is_valid() {
        assert!(USERS_QUERY.contains("query Users"));
        assert!(USERS_QUERY.contains("users"));
        assert!(USERS_QUERY.contains("nodes"));
    }

    #[test]
    fn test_issues_query_is_valid() {
        assert!(ISSUES_QUERY.contains("query Issues"));
        assert!(ISSUES_QUERY.contains("$first: Int"));
        assert!(ISSUES_QUERY.contains("$filter: IssueFilter"));
        assert!(ISSUES_QUERY.contains("issues"));
        assert!(ISSUES_QUERY.contains("nodes"));
        assert!(ISSUES_QUERY.contains("identifier"));
        assert!(ISSUES_QUERY.contains("state"));
        assert!(ISSUES_QUERY.contains("team"));
        assert!(ISSUES_QUERY.contains("assignee"));
    }

    #[test]
    fn test_issue_query_is_valid() {
        assert!(ISSUE_QUERY.contains("query Issue"));
        assert!(ISSUE_QUERY.contains("$id: String!"));
        assert!(ISSUE_QUERY.contains("issue(id: $id)"));
    }

    #[test]
    fn test_issue_by_identifier_query_is_valid() {
        assert!(ISSUE_BY_IDENTIFIER_QUERY.contains("query IssueByIdentifier"));
        assert!(ISSUE_BY_IDENTIFIER_QUERY.contains("$filter: IssueFilter!"));
        assert!(ISSUE_BY_IDENTIFIER_QUERY.contains("first: 1"));
    }

    #[test]
    fn test_issue_create_mutation_is_valid() {
        assert!(ISSUE_CREATE_MUTATION.contains("mutation IssueCreate"));
        assert!(ISSUE_CREATE_MUTATION.contains("$input: IssueCreateInput!"));
        assert!(ISSUE_CREATE_MUTATION.contains("issueCreate"));
        assert!(ISSUE_CREATE_MUTATION.contains("success"));
        assert!(ISSUE_CREATE_MUTATION.contains("issue"));
    }

    #[test]
    fn test_issue_update_mutation_is_valid() {
        assert!(ISSUE_UPDATE_MUTATION.contains("mutation IssueUpdate"));
        assert!(ISSUE_UPDATE_MUTATION.contains("$id: String!"));
        assert!(ISSUE_UPDATE_MUTATION.contains("$input: IssueUpdateInput!"));
        assert!(ISSUE_UPDATE_MUTATION.contains("issueUpdate"));
        assert!(ISSUE_UPDATE_MUTATION.contains("success"));
    }

    #[test]
    fn test_issue_delete_mutation_is_valid() {
        assert!(ISSUE_DELETE_MUTATION.contains("mutation IssueDelete"));
        assert!(ISSUE_DELETE_MUTATION.contains("$id: String!"));
        assert!(ISSUE_DELETE_MUTATION.contains("issueDelete"));
        assert!(ISSUE_DELETE_MUTATION.contains("success"));
    }

    #[test]
    fn test_issue_archive_mutation_is_valid() {
        assert!(ISSUE_ARCHIVE_MUTATION.contains("mutation IssueArchive"));
        assert!(ISSUE_ARCHIVE_MUTATION.contains("$id: String!"));
        assert!(ISSUE_ARCHIVE_MUTATION.contains("issueArchive"));
        assert!(ISSUE_ARCHIVE_MUTATION.contains("success"));
    }

    #[test]
    fn test_issue_unarchive_mutation_is_valid() {
        assert!(ISSUE_UNARCHIVE_MUTATION.contains("mutation IssueUnarchive"));
        assert!(ISSUE_UNARCHIVE_MUTATION.contains("$id: String!"));
        assert!(ISSUE_UNARCHIVE_MUTATION.contains("issueUnarchive"));
        assert!(ISSUE_UNARCHIVE_MUTATION.contains("success"));
    }
}
