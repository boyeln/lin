//! Organization and viewer-related GraphQL queries.

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
