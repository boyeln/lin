//! User-related GraphQL queries.

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
