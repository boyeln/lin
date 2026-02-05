//! Team-related GraphQL queries.

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
