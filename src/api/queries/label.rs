//! Label-related GraphQL queries.

/// Query to list all labels in the workspace.
///
/// Variables:
/// - `first` (Int, optional): Number of labels to fetch (default: 50)
///
/// Returns: `LabelsResponse`
pub const LABELS_QUERY: &str = r#"
query Labels($first: Int) {
    issueLabels(first: $first) {
        nodes {
            id
            name
            description
            color
            isGroup
            createdAt
            updatedAt
        }
    }
}
"#;

/// Query to list labels for a specific team.
///
/// Variables:
/// - `teamId` (String!): The team's unique identifier
/// - `first` (Int, optional): Number of labels to fetch (default: 50)
///
/// Returns: `TeamLabelsResponse`
pub const TEAM_LABELS_QUERY: &str = r#"
query TeamLabels($teamId: String!, $first: Int) {
    team(id: $teamId) {
        id
        labels(first: $first) {
            nodes {
                id
                name
                description
                color
                isGroup
                createdAt
                updatedAt
            }
        }
    }
}
"#;

/// Query to get a single label by ID.
///
/// Variables:
/// - `id` (String!): The label's unique identifier
///
/// Returns: `LabelResponse`
pub const LABEL_QUERY: &str = r#"
query Label($id: String!) {
    issueLabel(id: $id) {
        id
        name
        description
        color
        isGroup
        createdAt
        updatedAt
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_labels_query_is_valid() {
        assert!(LABELS_QUERY.contains("query Labels"));
        assert!(LABELS_QUERY.contains("$first: Int"));
        assert!(LABELS_QUERY.contains("issueLabels"));
        assert!(LABELS_QUERY.contains("nodes"));
        assert!(LABELS_QUERY.contains("name"));
        assert!(LABELS_QUERY.contains("color"));
        assert!(LABELS_QUERY.contains("isGroup"));
    }

    #[test]
    fn test_team_labels_query_is_valid() {
        assert!(TEAM_LABELS_QUERY.contains("query TeamLabels"));
        assert!(TEAM_LABELS_QUERY.contains("$teamId: String!"));
        assert!(TEAM_LABELS_QUERY.contains("team(id: $teamId)"));
        assert!(TEAM_LABELS_QUERY.contains("labels"));
        assert!(TEAM_LABELS_QUERY.contains("nodes"));
        assert!(TEAM_LABELS_QUERY.contains("name"));
    }

    #[test]
    fn test_label_query_is_valid() {
        assert!(LABEL_QUERY.contains("query Label"));
        assert!(LABEL_QUERY.contains("$id: String!"));
        assert!(LABEL_QUERY.contains("issueLabel(id: $id)"));
        assert!(LABEL_QUERY.contains("name"));
        assert!(LABEL_QUERY.contains("color"));
        assert!(LABEL_QUERY.contains("isGroup"));
    }
}
