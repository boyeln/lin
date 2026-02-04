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

/// Query to list workflow states for a team.
///
/// Variables:
/// - `id` (String!): The team's unique identifier (UUID or key)
///
/// Returns: `WorkflowStatesResponse`
pub const WORKFLOW_STATES_QUERY: &str = r#"
query WorkflowStates($id: String!) {
    team(id: $id) {
        id
        states {
            nodes {
                id
                name
                color
                type
            }
        }
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

/// Query to list cycles for a team.
///
/// Variables:
/// - `teamId` (String!): The team's unique identifier
/// - `first` (Int, optional): Number of cycles to fetch (default: 50)
///
/// Returns: `CyclesResponse`
pub const CYCLES_QUERY: &str = r#"
query Cycles($teamId: String!, $first: Int) {
    team(id: $teamId) {
        id
        cycles(first: $first, orderBy: createdAt) {
            nodes {
                id
                number
                name
                description
                startsAt
                endsAt
                completedAt
                progress
                completedScopeHistory
                scopeHistory
            }
        }
    }
}
"#;

/// Query to get a single cycle by ID with its issues.
///
/// Variables:
/// - `id` (String!): The cycle's unique identifier
///
/// Returns: `CycleResponse`
pub const CYCLE_QUERY: &str = r#"
query Cycle($id: String!) {
    cycle(id: $id) {
        id
        number
        name
        description
        startsAt
        endsAt
        completedAt
        progress
        completedScopeHistory
        scopeHistory
        issues {
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
}
"#;

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

/// Query to list all documents in the organization.
///
/// Variables:
/// - `first` (Int, optional): Number of documents to fetch (default: 50)
/// - `filter` (DocumentFilter, optional): Filter criteria for documents
///
/// Returns: `DocumentsResponse`
pub const DOCUMENTS_QUERY: &str = r#"
query Documents($first: Int, $filter: DocumentFilter) {
    documents(first: $first, filter: $filter) {
        nodes {
            id
            title
            icon
            color
            createdAt
            updatedAt
            creator {
                id
                name
                email
                displayName
                active
            }
            project {
                id
                name
            }
        }
    }
}
"#;

/// Query to get a single document by ID (with content).
///
/// Variables:
/// - `id` (String!): The document's unique identifier
///
/// Returns: `DocumentResponse`
pub const DOCUMENT_QUERY: &str = r#"
query Document($id: String!) {
    document(id: $id) {
        id
        title
        content
        icon
        color
        createdAt
        updatedAt
        creator {
            id
            name
            email
            displayName
            active
        }
        project {
            id
            name
        }
    }
}
"#;

/// Mutation to create a new document.
///
/// Variables:
/// - `input` (DocumentCreateInput!): Document creation input containing:
///   - `title` (String!): Document title
///   - `content` (String, optional): Document content (markdown)
///   - `projectId` (String, optional): Project ID to associate with
///
/// Returns: `DocumentCreateResponse`
pub const DOCUMENT_CREATE_MUTATION: &str = r#"
mutation DocumentCreate($input: DocumentCreateInput!) {
    documentCreate(input: $input) {
        success
        document {
            id
            title
            content
            icon
            color
            createdAt
            updatedAt
            creator {
                id
                name
                email
                displayName
                active
            }
            project {
                id
                name
            }
        }
    }
}
"#;

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

/// Query to search issues by full-text search.
///
/// Variables:
/// - `query` (String!): The search query string
/// - `first` (Int, optional): Number of issues to fetch (default: 50)
/// - `filter` (IssueFilter, optional): Additional filters to apply
///
/// Returns: `IssueSearchResponse`
pub const ISSUE_SEARCH_QUERY: &str = r#"
query IssueSearch($query: String!, $first: Int, $filter: IssueFilter) {
    issueSearch(query: $query, first: $first, filter: $filter) {
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
    fn test_workflow_states_query_is_valid() {
        assert!(WORKFLOW_STATES_QUERY.contains("query WorkflowStates"));
        assert!(WORKFLOW_STATES_QUERY.contains("$id: String!"));
        assert!(WORKFLOW_STATES_QUERY.contains("team(id: $id)"));
        assert!(WORKFLOW_STATES_QUERY.contains("states"));
        assert!(WORKFLOW_STATES_QUERY.contains("nodes"));
        assert!(WORKFLOW_STATES_QUERY.contains("type"));
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

    #[test]
    fn test_projects_query_is_valid() {
        assert!(PROJECTS_QUERY.contains("query Projects"));
        assert!(PROJECTS_QUERY.contains("$first: Int"));
        assert!(PROJECTS_QUERY.contains("$filter: ProjectFilter"));
        assert!(PROJECTS_QUERY.contains("projects"));
        assert!(PROJECTS_QUERY.contains("nodes"));
        assert!(PROJECTS_QUERY.contains("progress"));
    }

    #[test]
    fn test_project_query_is_valid() {
        assert!(PROJECT_QUERY.contains("query Project"));
        assert!(PROJECT_QUERY.contains("$id: String!"));
        assert!(PROJECT_QUERY.contains("project(id: $id)"));
        assert!(PROJECT_QUERY.contains("progress"));
    }

    #[test]
    fn test_cycles_query_is_valid() {
        assert!(CYCLES_QUERY.contains("query Cycles"));
        assert!(CYCLES_QUERY.contains("$teamId: String!"));
        assert!(CYCLES_QUERY.contains("team(id: $teamId)"));
        assert!(CYCLES_QUERY.contains("cycles"));
        assert!(CYCLES_QUERY.contains("nodes"));
        assert!(CYCLES_QUERY.contains("number"));
        assert!(CYCLES_QUERY.contains("progress"));
        assert!(CYCLES_QUERY.contains("startsAt"));
        assert!(CYCLES_QUERY.contains("endsAt"));
    }

    #[test]
    fn test_cycle_query_is_valid() {
        assert!(CYCLE_QUERY.contains("query Cycle"));
        assert!(CYCLE_QUERY.contains("$id: String!"));
        assert!(CYCLE_QUERY.contains("cycle(id: $id)"));
        assert!(CYCLE_QUERY.contains("issues"));
        assert!(CYCLE_QUERY.contains("nodes"));
        assert!(CYCLE_QUERY.contains("identifier"));
    }

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

    #[test]
    fn test_documents_query_is_valid() {
        assert!(DOCUMENTS_QUERY.contains("query Documents"));
        assert!(DOCUMENTS_QUERY.contains("$first: Int"));
        assert!(DOCUMENTS_QUERY.contains("$filter: DocumentFilter"));
        assert!(DOCUMENTS_QUERY.contains("documents"));
        assert!(DOCUMENTS_QUERY.contains("nodes"));
        assert!(DOCUMENTS_QUERY.contains("title"));
        assert!(DOCUMENTS_QUERY.contains("creator"));
    }

    #[test]
    fn test_document_query_is_valid() {
        assert!(DOCUMENT_QUERY.contains("query Document"));
        assert!(DOCUMENT_QUERY.contains("$id: String!"));
        assert!(DOCUMENT_QUERY.contains("document(id: $id)"));
        assert!(DOCUMENT_QUERY.contains("title"));
        assert!(DOCUMENT_QUERY.contains("content"));
        assert!(DOCUMENT_QUERY.contains("creator"));
    }

    #[test]
    fn test_document_create_mutation_is_valid() {
        assert!(DOCUMENT_CREATE_MUTATION.contains("mutation DocumentCreate"));
        assert!(DOCUMENT_CREATE_MUTATION.contains("$input: DocumentCreateInput!"));
        assert!(DOCUMENT_CREATE_MUTATION.contains("documentCreate"));
        assert!(DOCUMENT_CREATE_MUTATION.contains("success"));
        assert!(DOCUMENT_CREATE_MUTATION.contains("document"));
        assert!(DOCUMENT_CREATE_MUTATION.contains("content"));
    }

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

    #[test]
    fn test_issue_search_query_is_valid() {
        assert!(ISSUE_SEARCH_QUERY.contains("query IssueSearch"));
        assert!(ISSUE_SEARCH_QUERY.contains("$query: String!"));
        assert!(ISSUE_SEARCH_QUERY.contains("$first: Int"));
        assert!(ISSUE_SEARCH_QUERY.contains("$filter: IssueFilter"));
        assert!(ISSUE_SEARCH_QUERY.contains("issueSearch"));
        assert!(ISSUE_SEARCH_QUERY.contains("nodes"));
        assert!(ISSUE_SEARCH_QUERY.contains("identifier"));
        assert!(ISSUE_SEARCH_QUERY.contains("state"));
        assert!(ISSUE_SEARCH_QUERY.contains("team"));
        assert!(ISSUE_SEARCH_QUERY.contains("assignee"));
    }
}
