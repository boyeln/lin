//! Comment-related GraphQL queries and mutations.

/// Mutation to create a comment on an issue.
///
/// Variables:
/// - `input` (CommentCreateInput!): Comment creation input containing:
///   - `issueId` (String!): The issue ID to comment on
///   - `body` (String!): The comment body
///
/// Returns: `CommentCreateResponse`
pub const COMMENT_CREATE_MUTATION: &str = r#"
mutation CommentCreate($input: CommentCreateInput!) {
    commentCreate(input: $input) {
        success
        comment {
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
"#;
