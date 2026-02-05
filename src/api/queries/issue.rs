//! Issue-related GraphQL queries and mutations.

/// Query to list issues with optional filters and sorting.
///
/// Variables:
/// - `first` (Int, optional): Number of issues to fetch
/// - `filter` (IssueFilter, optional): Filter criteria for issues
/// - `orderBy` (PaginationOrderBy, optional): Field to sort by (createdAt, updatedAt, priority, title)
/// - `sortDirection` (String, optional): Sort direction passed via filter's sortDirection
///
/// Returns: `IssuesResponse`
pub const ISSUES_QUERY: &str = r#"
query Issues($first: Int, $filter: IssueFilter, $orderBy: PaginationOrderBy) {
    issues(first: $first, filter: $filter, orderBy: $orderBy) {
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

/// Query to get a single issue with its comments by ID.
///
/// Variables:
/// - `id` (String!): The issue's unique identifier
///
/// Returns: `IssueWithCommentsResponse`
pub const ISSUE_WITH_COMMENTS_QUERY: &str = r#"
query IssueWithComments($id: String!) {
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
        comments {
            nodes {
                id
                body
                createdAt
                updatedAt
                user {
                    id
                    name
                    email
                    displayName
                    active
                }
            }
        }
    }
}
"#;

/// Query to get an issue with comments by its identifier (e.g., "ENG-123").
///
/// Variables:
/// - `filter` (IssueFilter!): Filter containing the identifier
///
/// Returns: `IssuesWithCommentsResponse` (with single result)
pub const ISSUE_BY_IDENTIFIER_WITH_COMMENTS_QUERY: &str = r#"
query IssueByIdentifierWithComments($filter: IssueFilter!) {
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
            comments {
                nodes {
                    id
                    body
                    createdAt
                    updatedAt
                    user {
                        id
                        name
                        email
                        displayName
                        active
                    }
                }
            }
        }
    }
}
"#;

/// Query to get comments for an issue by ID.
///
/// Variables:
/// - `id` (String!): The issue's unique identifier
///
/// Returns: `IssueCommentsResponse`
pub const ISSUE_COMMENTS_QUERY: &str = r#"
query IssueComments($id: String!) {
    issue(id: $id) {
        id
        identifier
        comments {
            nodes {
                id
                body
                createdAt
                updatedAt
                user {
                    id
                    name
                    email
                    displayName
                    active
                }
            }
        }
    }
}
"#;

/// Query to get all relations for an issue.
///
/// Variables:
/// - `id` (String!): The issue's unique identifier
///
/// Returns: `IssueRelationsResponse`
pub const ISSUE_RELATIONS_QUERY: &str = r#"
query IssueRelations($id: String!) {
    issue(id: $id) {
        id
        identifier
        relations {
            nodes {
                id
                type
                relatedIssue {
                    id
                    identifier
                    title
                }
            }
        }
        inverseRelations {
            nodes {
                id
                type
                issue {
                    id
                    identifier
                    title
                }
            }
        }
        parent {
            id
            identifier
            title
        }
        children {
            nodes {
                id
                identifier
                title
            }
        }
    }
}
"#;

/// Mutation to create an issue relation.
///
/// Variables:
/// - `input` (IssueRelationCreateInput!): Relation creation input containing:
///   - `issueId` (String!): The source issue ID
///   - `relatedIssueId` (String!): The target issue ID
///   - `type` (String!): The relation type (blocks, duplicate, related)
///
/// Returns: `IssueRelationCreateResponse`
pub const ISSUE_RELATION_CREATE_MUTATION: &str = r#"
mutation IssueRelationCreate($input: IssueRelationCreateInput!) {
    issueRelationCreate(input: $input) {
        success
        issueRelation {
            id
            type
            issue {
                id
                identifier
                title
            }
            relatedIssue {
                id
                identifier
                title
            }
        }
    }
}
"#;

/// Mutation to delete an issue relation.
///
/// Variables:
/// - `id` (String!): The relation's unique identifier
///
/// Returns: `IssueRelationDeleteResponse`
pub const ISSUE_RELATION_DELETE_MUTATION: &str = r#"
mutation IssueRelationDelete($id: String!) {
    issueRelationDelete(id: $id) {
        success
    }
}
"#;

/// Mutation to update an issue's parent (for parent/child relations).
///
/// Variables:
/// - `id` (String!): The child issue's unique identifier
/// - `input` (IssueUpdateInput!): Update input containing parentId
///
/// Returns: `IssueUpdateResponse`
pub const ISSUE_SET_PARENT_MUTATION: &str = r#"
mutation IssueSetParent($id: String!, $input: IssueUpdateInput!) {
    issueUpdate(id: $id, input: $input) {
        success
        issue {
            id
            identifier
            title
            parent {
                id
                identifier
                title
            }
        }
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_issue_relations_query_is_valid() {
        assert!(ISSUE_RELATIONS_QUERY.contains("query IssueRelations"));
        assert!(ISSUE_RELATIONS_QUERY.contains("$id: String!"));
        assert!(ISSUE_RELATIONS_QUERY.contains("issue(id: $id)"));
        assert!(ISSUE_RELATIONS_QUERY.contains("relations"));
        assert!(ISSUE_RELATIONS_QUERY.contains("inverseRelations"));
        assert!(ISSUE_RELATIONS_QUERY.contains("parent"));
        assert!(ISSUE_RELATIONS_QUERY.contains("children"));
        assert!(ISSUE_RELATIONS_QUERY.contains("relatedIssue"));
    }

    #[test]
    fn test_issue_relation_create_mutation_is_valid() {
        assert!(ISSUE_RELATION_CREATE_MUTATION.contains("mutation IssueRelationCreate"));
        assert!(ISSUE_RELATION_CREATE_MUTATION.contains("$input: IssueRelationCreateInput!"));
        assert!(ISSUE_RELATION_CREATE_MUTATION.contains("issueRelationCreate"));
        assert!(ISSUE_RELATION_CREATE_MUTATION.contains("success"));
        assert!(ISSUE_RELATION_CREATE_MUTATION.contains("issueRelation"));
        assert!(ISSUE_RELATION_CREATE_MUTATION.contains("type"));
    }

    #[test]
    fn test_issue_relation_delete_mutation_is_valid() {
        assert!(ISSUE_RELATION_DELETE_MUTATION.contains("mutation IssueRelationDelete"));
        assert!(ISSUE_RELATION_DELETE_MUTATION.contains("$id: String!"));
        assert!(ISSUE_RELATION_DELETE_MUTATION.contains("issueRelationDelete"));
        assert!(ISSUE_RELATION_DELETE_MUTATION.contains("success"));
    }

    #[test]
    fn test_issue_set_parent_mutation_is_valid() {
        assert!(ISSUE_SET_PARENT_MUTATION.contains("mutation IssueSetParent"));
        assert!(ISSUE_SET_PARENT_MUTATION.contains("$id: String!"));
        assert!(ISSUE_SET_PARENT_MUTATION.contains("$input: IssueUpdateInput!"));
        assert!(ISSUE_SET_PARENT_MUTATION.contains("issueUpdate"));
        assert!(ISSUE_SET_PARENT_MUTATION.contains("success"));
        assert!(ISSUE_SET_PARENT_MUTATION.contains("parent"));
    }
}
