//! Project-related GraphQL queries.

/// Query to list projects in the organization.
///
/// Variables:
/// - `first` (Int, optional): Number of projects to fetch (default: 50)
/// - `filter` (ProjectFilter, optional): Filter criteria for projects
///
/// Returns: `ProjectsResponse`
pub const PROJECTS_QUERY: &str = r#"
query Projects($first: Int, $filter: ProjectFilter) {
    projects(first: $first, filter: $filter) {
        nodes {
            id
            name
            description
            content
            state
            createdAt
            updatedAt
            targetDate
            startDate
            progress
        }
    }
}
"#;

/// Query to get a single project by ID.
///
/// Variables:
/// - `id` (String!): The project's unique identifier
///
/// Returns: `ProjectResponse`
pub const PROJECT_QUERY: &str = r#"
query Project($id: String!) {
    project(id: $id) {
        id
        name
        description
        state
        createdAt
        updatedAt
        targetDate
        startDate
        progress
    }
}
"#;
