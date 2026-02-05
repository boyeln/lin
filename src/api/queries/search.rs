//! Search-related GraphQL queries.

/// Query to search issues using the issues query with searchableContent filter.
///
/// Variables:
/// - `first` (Int, optional): Number of issues to fetch (default: 50)
/// - `filter` (IssueFilter, optional): Filters including searchableContent for text search
///
/// Returns: `IssueSearchResponse`
pub const ISSUE_SEARCH_QUERY: &str = r#"
query IssueSearch($first: Int, $filter: IssueFilter) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_search_query_is_valid() {
        assert!(ISSUE_SEARCH_QUERY.contains("query IssueSearch"));
        assert!(ISSUE_SEARCH_QUERY.contains("$first: Int"));
        assert!(ISSUE_SEARCH_QUERY.contains("$filter: IssueFilter"));
        assert!(ISSUE_SEARCH_QUERY.contains("issues(first: $first, filter: $filter)"));
        assert!(ISSUE_SEARCH_QUERY.contains("nodes"));
        assert!(ISSUE_SEARCH_QUERY.contains("identifier"));
        assert!(ISSUE_SEARCH_QUERY.contains("state"));
        assert!(ISSUE_SEARCH_QUERY.contains("team"));
        assert!(ISSUE_SEARCH_QUERY.contains("assignee"));
    }
}
