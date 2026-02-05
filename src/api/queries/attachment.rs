//! Attachment-related GraphQL queries and mutations.

/// Query to get attachments for an issue by ID.
///
/// Variables:
/// - `id` (String!): The issue's unique identifier
///
/// Returns: `IssueAttachmentsResponse`
pub const ISSUE_ATTACHMENTS_QUERY: &str = r#"
query IssueAttachments($id: String!) {
    issue(id: $id) {
        id
        identifier
        attachments {
            nodes {
                id
                title
                subtitle
                url
                metadata
                createdAt
                updatedAt
                creator {
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

/// Query to get a single attachment by ID.
///
/// Variables:
/// - `id` (String!): The attachment's unique identifier
///
/// Returns: `AttachmentResponse`
pub const ATTACHMENT_QUERY: &str = r#"
query Attachment($id: String!) {
    attachment(id: $id) {
        id
        title
        subtitle
        url
        metadata
        createdAt
        updatedAt
        creator {
            id
            name
            email
            displayName
            active
        }
        issue {
            id
            identifier
        }
    }
}
"#;

/// Mutation to create an attachment on an issue.
///
/// Variables:
/// - `input` (AttachmentCreateInput!): Attachment creation input containing:
///   - `issueId` (String!): The issue ID to attach to
///   - `title` (String!): Attachment title
///   - `url` (String!): URL of the attachment
///   - `subtitle` (String, optional): Subtitle/description
///
/// Returns: `AttachmentCreateResponse`
pub const ATTACHMENT_CREATE_MUTATION: &str = r#"
mutation AttachmentCreate($input: AttachmentCreateInput!) {
    attachmentCreate(input: $input) {
        success
        attachment {
            id
            title
            subtitle
            url
            metadata
            createdAt
            updatedAt
            creator {
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

/// Query to list attachments filtered by URL pattern.
///
/// This is used to find git-related attachments (branches, PRs) for an issue.
/// Attachments with URLs matching GitHub/GitLab patterns are considered git links.
///
/// Variables:
/// - `id` (String!): The issue's unique identifier
///
/// Returns: `IssueAttachmentsResponse`
pub const ISSUE_GIT_LINKS_QUERY: &str = r#"
query IssueGitLinks($id: String!) {
    issue(id: $id) {
        id
        identifier
        attachments {
            nodes {
                id
                title
                subtitle
                url
                metadata
                createdAt
                updatedAt
                creator {
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

/// Mutation to create a file upload URL.
///
/// This returns a presigned URL where the file should be uploaded,
/// and the final asset URL where the file will be available after upload.
///
/// Variables:
/// - `contentType` (String!): MIME type of the file
/// - `filename` (String!): Name of the file
/// - `size` (Int!): Size of the file in bytes
///
/// Returns: `FileUploadResponse`
pub const FILE_UPLOAD_CREATE_MUTATION: &str = r#"
mutation FileUploadCreate($contentType: String!, $filename: String!, $size: Int!) {
    fileUpload(contentType: $contentType, filename: $filename, size: $size) {
        uploadFile {
            uploadUrl
            assetUrl
            headers {
                key
                value
            }
        }
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_attachments_query_is_valid() {
        assert!(ISSUE_ATTACHMENTS_QUERY.contains("query IssueAttachments"));
        assert!(ISSUE_ATTACHMENTS_QUERY.contains("$id: String!"));
        assert!(ISSUE_ATTACHMENTS_QUERY.contains("issue(id: $id)"));
        assert!(ISSUE_ATTACHMENTS_QUERY.contains("attachments"));
        assert!(ISSUE_ATTACHMENTS_QUERY.contains("nodes"));
    }

    #[test]
    fn test_attachment_query_is_valid() {
        assert!(ATTACHMENT_QUERY.contains("query Attachment"));
        assert!(ATTACHMENT_QUERY.contains("$id: String!"));
        assert!(ATTACHMENT_QUERY.contains("attachment(id: $id)"));
        assert!(ATTACHMENT_QUERY.contains("url"));
    }

    #[test]
    fn test_attachment_create_mutation_is_valid() {
        assert!(ATTACHMENT_CREATE_MUTATION.contains("mutation AttachmentCreate"));
        assert!(ATTACHMENT_CREATE_MUTATION.contains("$input: AttachmentCreateInput!"));
        assert!(ATTACHMENT_CREATE_MUTATION.contains("attachmentCreate"));
        assert!(ATTACHMENT_CREATE_MUTATION.contains("success"));
        assert!(ATTACHMENT_CREATE_MUTATION.contains("attachment"));
    }

    #[test]
    fn test_file_upload_create_mutation_is_valid() {
        assert!(FILE_UPLOAD_CREATE_MUTATION.contains("mutation FileUploadCreate"));
        assert!(FILE_UPLOAD_CREATE_MUTATION.contains("$contentType: String!"));
        assert!(FILE_UPLOAD_CREATE_MUTATION.contains("$filename: String!"));
        assert!(FILE_UPLOAD_CREATE_MUTATION.contains("$size: Int!"));
        assert!(FILE_UPLOAD_CREATE_MUTATION.contains("uploadUrl"));
        assert!(FILE_UPLOAD_CREATE_MUTATION.contains("assetUrl"));
    }

    #[test]
    fn test_issue_git_links_query_is_valid() {
        assert!(ISSUE_GIT_LINKS_QUERY.contains("query IssueGitLinks"));
        assert!(ISSUE_GIT_LINKS_QUERY.contains("$id: String!"));
        assert!(ISSUE_GIT_LINKS_QUERY.contains("issue(id: $id)"));
        assert!(ISSUE_GIT_LINKS_QUERY.contains("attachments"));
        assert!(ISSUE_GIT_LINKS_QUERY.contains("nodes"));
        assert!(ISSUE_GIT_LINKS_QUERY.contains("url"));
    }
}
