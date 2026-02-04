//! Domain model types for Linear API entities.
//!
//! These types map to Linear's GraphQL schema and are used for both
//! serialization (JSON output) and deserialization (API responses).

use serde::{Deserialize, Serialize};

// =============================================================================
// Core Domain Models
// =============================================================================

/// A Linear user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    /// Unique identifier for the user.
    pub id: String,
    /// The user's full name.
    pub name: String,
    /// The user's email address.
    pub email: String,
    /// The user's display name (may differ from full name).
    pub display_name: Option<String>,
    /// Whether the user account is active.
    pub active: bool,
}

/// A Linear team.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    /// Unique identifier for the team.
    pub id: String,
    /// The team's key/prefix (e.g., "ENG").
    pub key: String,
    /// The team's name.
    pub name: String,
    /// Optional description of the team.
    pub description: Option<String>,
}

/// A workflow state for issues.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowState {
    /// Unique identifier for the state.
    pub id: String,
    /// The state's name (e.g., "In Progress", "Done").
    pub name: String,
    /// The state's color (hex color code).
    pub color: String,
    /// The type of state (backlog, unstarted, started, completed, canceled).
    /// Note: `type` is a reserved keyword in Rust, so we use `type_` with serde rename.
    #[serde(rename = "type")]
    pub type_: String,
}

/// A Linear project.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    /// Unique identifier for the project.
    pub id: String,
    /// The project's name.
    pub name: String,
    /// Optional description of the project.
    pub description: Option<String>,
    /// The project's state (planned, started, paused, completed, canceled).
    pub state: String,
    /// ISO 8601 timestamp of when the project was created.
    pub created_at: String,
    /// ISO 8601 timestamp of when the project was last updated.
    pub updated_at: String,
    /// Target date for the project (optional).
    pub target_date: Option<String>,
    /// Start date for the project (optional).
    pub start_date: Option<String>,
    /// Progress percentage of the project (0-100).
    pub progress: f64,
}

/// A Linear label.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Label {
    /// Unique identifier for the label.
    pub id: String,
    /// The label's name.
    pub name: String,
    /// The label's description (optional).
    pub description: Option<String>,
    /// The label's color (hex color code).
    pub color: String,
    /// Whether this is a group label (parent label).
    pub is_group: bool,
    /// ISO 8601 timestamp of when the label was created.
    pub created_at: String,
    /// ISO 8601 timestamp of when the label was last updated.
    pub updated_at: String,
}

/// A Linear document (used in list view, without content).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    /// Unique identifier for the document.
    pub id: String,
    /// The document's title.
    pub title: String,
    /// Optional icon for the document.
    pub icon: Option<String>,
    /// Optional color for the document.
    pub color: Option<String>,
    /// ISO 8601 timestamp of when the document was created.
    pub created_at: String,
    /// ISO 8601 timestamp of when the document was last updated.
    pub updated_at: String,
    /// The user who created the document.
    pub creator: Option<User>,
    /// The project this document belongs to (optional).
    pub project: Option<DocumentProject>,
}

/// A Linear document with content (used in detail view).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentWithContent {
    /// Unique identifier for the document.
    pub id: String,
    /// The document's title.
    pub title: String,
    /// The document's content (markdown).
    pub content: Option<String>,
    /// Optional icon for the document.
    pub icon: Option<String>,
    /// Optional color for the document.
    pub color: Option<String>,
    /// ISO 8601 timestamp of when the document was created.
    pub created_at: String,
    /// ISO 8601 timestamp of when the document was last updated.
    pub updated_at: String,
    /// The user who created the document.
    pub creator: Option<User>,
    /// The project this document belongs to (optional).
    pub project: Option<DocumentProject>,
}

/// A simplified project reference for documents.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentProject {
    /// Unique identifier for the project.
    pub id: String,
    /// The project's name.
    pub name: String,
}

/// A Linear attachment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    /// Unique identifier for the attachment.
    pub id: String,
    /// The attachment's title.
    pub title: String,
    /// The attachment's subtitle/description (optional).
    pub subtitle: Option<String>,
    /// URL where the attachment can be accessed.
    pub url: String,
    /// Additional metadata about the attachment.
    pub metadata: Option<serde_json::Value>,
    /// ISO 8601 timestamp of when the attachment was created.
    pub created_at: String,
    /// ISO 8601 timestamp of when the attachment was last updated.
    pub updated_at: String,
    /// The user who created the attachment.
    pub creator: Option<User>,
}

/// A Linear attachment with its associated issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentWithIssue {
    /// Unique identifier for the attachment.
    pub id: String,
    /// The attachment's title.
    pub title: String,
    /// The attachment's subtitle/description (optional).
    pub subtitle: Option<String>,
    /// URL where the attachment can be accessed.
    pub url: String,
    /// Additional metadata about the attachment.
    pub metadata: Option<serde_json::Value>,
    /// ISO 8601 timestamp of when the attachment was created.
    pub created_at: String,
    /// ISO 8601 timestamp of when the attachment was last updated.
    pub updated_at: String,
    /// The user who created the attachment.
    pub creator: Option<User>,
    /// The issue this attachment belongs to.
    pub issue: Option<AttachmentIssue>,
}

/// A simplified issue reference for attachments.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentIssue {
    /// Unique identifier for the issue.
    pub id: String,
    /// Human-readable identifier (e.g., "ENG-123").
    pub identifier: String,
}

/// A Linear cycle (sprint).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cycle {
    /// Unique identifier for the cycle.
    pub id: String,
    /// The cycle's number within the team.
    pub number: i32,
    /// The cycle's name (optional).
    pub name: Option<String>,
    /// Optional description of the cycle.
    pub description: Option<String>,
    /// ISO 8601 timestamp of when the cycle starts.
    pub starts_at: Option<String>,
    /// ISO 8601 timestamp of when the cycle ends.
    pub ends_at: Option<String>,
    /// ISO 8601 timestamp of when the cycle was completed (optional).
    pub completed_at: Option<String>,
    /// Progress percentage of the cycle (0-100).
    pub progress: f64,
    /// Completed scope history for the cycle.
    pub completed_scope_history: Vec<f64>,
    /// Scope history for the cycle.
    pub scope_history: Vec<f64>,
}

/// A Linear cycle with its issues included.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CycleWithIssues {
    /// Unique identifier for the cycle.
    pub id: String,
    /// The cycle's number within the team.
    pub number: i32,
    /// The cycle's name (optional).
    pub name: Option<String>,
    /// Optional description of the cycle.
    pub description: Option<String>,
    /// ISO 8601 timestamp of when the cycle starts.
    pub starts_at: Option<String>,
    /// ISO 8601 timestamp of when the cycle ends.
    pub ends_at: Option<String>,
    /// ISO 8601 timestamp of when the cycle was completed (optional).
    pub completed_at: Option<String>,
    /// Progress percentage of the cycle (0-100).
    pub progress: f64,
    /// Completed scope history for the cycle.
    pub completed_scope_history: Vec<f64>,
    /// Scope history for the cycle.
    pub scope_history: Vec<f64>,
    /// Issues in this cycle.
    pub issues: IssueConnection,
}

/// A Linear issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    /// Unique identifier for the issue.
    pub id: String,
    /// Human-readable identifier (e.g., "ENG-123").
    pub identifier: String,
    /// Issue title.
    pub title: String,
    /// Issue description (may be empty or null).
    pub description: Option<String>,
    /// Priority level (0 = no priority, 1 = urgent, 2 = high, 3 = normal, 4 = low).
    pub priority: i32,
    /// Current workflow state.
    pub state: Option<WorkflowState>,
    /// Team the issue belongs to.
    pub team: Option<Team>,
    /// User assigned to the issue.
    pub assignee: Option<User>,
    /// ISO 8601 timestamp of when the issue was created.
    pub created_at: String,
    /// ISO 8601 timestamp of when the issue was last updated.
    pub updated_at: String,
}

/// A comment on a Linear issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    /// Unique identifier for the comment.
    pub id: String,
    /// The comment body/content.
    pub body: String,
    /// The user who created the comment.
    pub user: Option<User>,
    /// ISO 8601 timestamp of when the comment was created.
    pub created_at: String,
    /// ISO 8601 timestamp of when the comment was last updated.
    pub updated_at: String,
}

/// An issue with its comments included.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueWithComments {
    /// Unique identifier for the issue.
    pub id: String,
    /// Human-readable identifier (e.g., "ENG-123").
    pub identifier: String,
    /// Issue title.
    pub title: String,
    /// Issue description (may be empty or null).
    pub description: Option<String>,
    /// Priority level (0 = no priority, 1 = urgent, 2 = high, 3 = normal, 4 = low).
    pub priority: i32,
    /// Current workflow state.
    pub state: Option<WorkflowState>,
    /// Team the issue belongs to.
    pub team: Option<Team>,
    /// User assigned to the issue.
    pub assignee: Option<User>,
    /// ISO 8601 timestamp of when the issue was created.
    pub created_at: String,
    /// ISO 8601 timestamp of when the issue was last updated.
    pub updated_at: String,
    /// Comments on the issue.
    pub comments: CommentConnection,
}

// =============================================================================
// Connection Types (for paginated results)
// =============================================================================

/// A paginated list of comments.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentConnection {
    /// List of comments.
    pub nodes: Vec<Comment>,
}

/// A paginated list of workflow states.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowStateConnection {
    /// List of workflow states.
    pub nodes: Vec<WorkflowState>,
}

/// A paginated list of issues.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueConnection {
    /// List of issues.
    pub nodes: Vec<Issue>,
}

/// A paginated list of teams.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamConnection {
    /// List of teams.
    pub nodes: Vec<Team>,
}

/// A paginated list of users.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserConnection {
    /// List of users.
    pub nodes: Vec<User>,
}

/// A paginated list of projects.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConnection {
    /// List of projects.
    pub nodes: Vec<Project>,
}

/// A paginated list of cycles.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CycleConnection {
    /// List of cycles.
    pub nodes: Vec<Cycle>,
}

/// A paginated list of labels.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelConnection {
    /// List of labels.
    pub nodes: Vec<Label>,
}

/// A paginated list of documents.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentConnection {
    /// List of documents.
    pub nodes: Vec<Document>,
}

/// A paginated list of attachments.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentConnection {
    /// List of attachments.
    pub nodes: Vec<Attachment>,
}

// =============================================================================
// GraphQL Response Wrappers
// =============================================================================

/// Response wrapper for the viewer query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewerResponse {
    /// The authenticated user.
    pub viewer: User,
}

/// Response wrapper for the teams query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamsResponse {
    /// Paginated list of teams.
    pub teams: TeamConnection,
}

/// Response wrapper for a single team query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamResponse {
    /// The requested team.
    pub team: Team,
}

/// Response wrapper for the users query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsersResponse {
    /// Paginated list of users.
    pub users: UserConnection,
}

/// Response wrapper for the projects query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectsResponse {
    /// Paginated list of projects.
    pub projects: ProjectConnection,
}

/// Response wrapper for a single project query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectResponse {
    /// The requested project.
    pub project: Project,
}

/// Team with cycles for the cycles query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamWithCycles {
    /// Unique identifier for the team.
    pub id: String,
    /// The team's cycles.
    pub cycles: CycleConnection,
}

/// Response wrapper for the cycles query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CyclesResponse {
    /// The team with its cycles.
    pub team: TeamWithCycles,
}

/// Response wrapper for a single cycle query (with issues).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CycleResponse {
    /// The requested cycle with its issues.
    pub cycle: CycleWithIssues,
}

/// Response wrapper for labels query (workspace-level).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelsResponse {
    /// Paginated list of labels.
    pub issue_labels: LabelConnection,
}

/// Team with labels for the team labels query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamWithLabels {
    /// Unique identifier for the team.
    pub id: String,
    /// The team's labels.
    pub labels: LabelConnection,
}

/// Response wrapper for team labels query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamLabelsResponse {
    /// The team with its labels.
    pub team: TeamWithLabels,
}

/// Response wrapper for a single label query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelResponse {
    /// The requested label.
    pub issue_label: Label,
}

/// Response wrapper for documents query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentsResponse {
    /// Paginated list of documents.
    pub documents: DocumentConnection,
}

/// Response wrapper for a single document query (with content).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentResponse {
    /// The requested document with content.
    pub document: DocumentWithContent,
}

/// Response for document creation mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentCreatePayload {
    /// Whether the mutation was successful.
    pub success: bool,
    /// The created document.
    pub document: Option<DocumentWithContent>,
}

/// Response wrapper for document creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentCreateResponse {
    /// The mutation payload.
    pub document_create: DocumentCreatePayload,
}

/// Issue with attachments for the attachments query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueWithAttachments {
    /// Unique identifier for the issue.
    pub id: String,
    /// Human-readable identifier (e.g., "ENG-123").
    pub identifier: String,
    /// The issue's attachments.
    pub attachments: AttachmentConnection,
}

/// Response wrapper for issue attachments query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueAttachmentsResponse {
    /// The issue with its attachments.
    pub issue: IssueWithAttachments,
}

/// Response wrapper for a single attachment query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentResponse {
    /// The requested attachment.
    pub attachment: AttachmentWithIssue,
}

/// Response for attachment creation mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentCreatePayload {
    /// Whether the mutation was successful.
    pub success: bool,
    /// The created attachment.
    pub attachment: Option<Attachment>,
}

/// Response wrapper for attachment creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentCreateResponse {
    /// The mutation payload.
    pub attachment_create: AttachmentCreatePayload,
}

/// Header for file upload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadHeader {
    /// Header key.
    pub key: String,
    /// Header value.
    pub value: String,
}

/// Upload file details returned by file upload mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadFile {
    /// Presigned URL where the file should be uploaded.
    pub upload_url: String,
    /// Final URL where the file will be available after upload.
    pub asset_url: String,
    /// Headers to include in the upload request.
    pub headers: Vec<UploadHeader>,
}

/// Response for file upload mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileUploadPayload {
    /// The upload file details.
    pub upload_file: UploadFile,
}

/// Response wrapper for file upload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileUploadResponse {
    /// The mutation payload.
    pub file_upload: FileUploadPayload,
}

/// Response wrapper for the issues query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssuesResponse {
    /// Paginated list of issues.
    pub issues: IssueConnection,
}

/// Team with workflow states for the workflow states query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamWithWorkflowStates {
    /// Unique identifier for the team.
    pub id: String,
    /// The team's workflow states.
    pub states: WorkflowStateConnection,
}

/// Response wrapper for the workflow states query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowStatesResponse {
    /// The team with its workflow states.
    pub team: TeamWithWorkflowStates,
}

/// Response wrapper for a single issue query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueResponse {
    /// The requested issue.
    pub issue: Issue,
}

/// Response wrapper for a single issue with comments query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueWithCommentsResponse {
    /// The requested issue with comments.
    pub issue: IssueWithComments,
}

/// Response wrapper for issues query returning issues with comments.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssuesWithCommentsResponse {
    /// Paginated list of issues with comments.
    pub issues: IssueWithCommentsConnection,
}

/// A paginated list of issues with comments.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueWithCommentsConnection {
    /// List of issues with comments.
    pub nodes: Vec<IssueWithComments>,
}

/// Response wrapper for comments query on an issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueCommentsResponse {
    /// The issue containing the comments.
    pub issue: IssueWithCommentsOnly,
}

/// Issue with only comments (used for fetching just comments).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueWithCommentsOnly {
    /// Unique identifier for the issue.
    pub id: String,
    /// Human-readable identifier (e.g., "ENG-123").
    pub identifier: String,
    /// Comments on the issue.
    pub comments: CommentConnection,
}

// =============================================================================
// Mutation Response Wrappers
// =============================================================================

/// Response for issue creation mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueCreatePayload {
    /// Whether the mutation was successful.
    pub success: bool,
    /// The created issue.
    pub issue: Option<Issue>,
}

/// Response wrapper for issue creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueCreateResponse {
    /// The mutation payload.
    pub issue_create: IssueCreatePayload,
}

/// Response for issue update mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueUpdatePayload {
    /// Whether the mutation was successful.
    pub success: bool,
    /// The updated issue.
    pub issue: Option<Issue>,
}

/// Response wrapper for issue update.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueUpdateResponse {
    /// The mutation payload.
    pub issue_update: IssueUpdatePayload,
}

/// Response for issue deletion mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueDeletePayload {
    /// Whether the mutation was successful.
    pub success: bool,
}

/// Response wrapper for issue deletion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueDeleteResponse {
    /// The mutation payload.
    pub issue_delete: IssueDeletePayload,
}

/// Response for issue archive mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueArchivePayload {
    /// Whether the mutation was successful.
    pub success: bool,
}

/// Response wrapper for issue archive.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueArchiveResponse {
    /// The mutation payload.
    pub issue_archive: IssueArchivePayload,
}

/// Response for issue unarchive mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueUnarchivePayload {
    /// Whether the mutation was successful.
    pub success: bool,
}

/// Response wrapper for issue unarchive.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueUnarchiveResponse {
    /// The mutation payload.
    pub issue_unarchive: IssueUnarchivePayload,
}

/// Response for comment creation mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentCreatePayload {
    /// Whether the mutation was successful.
    pub success: bool,
    /// The created comment.
    pub comment: Option<Comment>,
}

/// Response wrapper for comment creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentCreateResponse {
    /// The mutation payload.
    pub comment_create: CommentCreatePayload,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_deserialization() {
        let json = r#"{
            "id": "user-123",
            "name": "John Doe",
            "email": "john@example.com",
            "displayName": "JD",
            "active": true
        }"#;
        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.id, "user-123");
        assert_eq!(user.name, "John Doe");
        assert_eq!(user.email, "john@example.com");
        assert_eq!(user.display_name, Some("JD".to_string()));
        assert!(user.active);
    }

    #[test]
    fn test_user_serialization() {
        let user = User {
            id: "user-123".to_string(),
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            display_name: Some("JD".to_string()),
            active: true,
        };
        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("\"id\":\"user-123\""));
        assert!(json.contains("\"displayName\":\"JD\""));
    }

    #[test]
    fn test_team_deserialization() {
        let json = r#"{
            "id": "team-456",
            "key": "ENG",
            "name": "Engineering",
            "description": "The engineering team"
        }"#;
        let team: Team = serde_json::from_str(json).unwrap();
        assert_eq!(team.id, "team-456");
        assert_eq!(team.key, "ENG");
        assert_eq!(team.name, "Engineering");
        assert_eq!(team.description, Some("The engineering team".to_string()));
    }

    #[test]
    fn test_team_with_null_description() {
        let json = r#"{
            "id": "team-456",
            "key": "ENG",
            "name": "Engineering",
            "description": null
        }"#;
        let team: Team = serde_json::from_str(json).unwrap();
        assert!(team.description.is_none());
    }

    #[test]
    fn test_workflow_state_deserialization() {
        let json = r##"{
            "id": "state-789",
            "name": "In Progress",
            "color": "#0066ff",
            "type": "started"
        }"##;
        let state: WorkflowState = serde_json::from_str(json).unwrap();
        assert_eq!(state.id, "state-789");
        assert_eq!(state.name, "In Progress");
        assert_eq!(state.color, "#0066ff");
        assert_eq!(state.type_, "started");
    }

    #[test]
    fn test_issue_deserialization() {
        let json = r##"{
            "id": "issue-abc",
            "identifier": "ENG-123",
            "title": "Fix the bug",
            "description": "This is a bug description",
            "priority": 2,
            "state": {
                "id": "state-1",
                "name": "In Progress",
                "color": "#0066ff",
                "type": "started"
            },
            "team": {
                "id": "team-1",
                "key": "ENG",
                "name": "Engineering",
                "description": null
            },
            "assignee": {
                "id": "user-1",
                "name": "John Doe",
                "email": "john@example.com",
                "displayName": null,
                "active": true
            },
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-02T00:00:00.000Z"
        }"##;
        let issue: Issue = serde_json::from_str(json).unwrap();
        assert_eq!(issue.id, "issue-abc");
        assert_eq!(issue.identifier, "ENG-123");
        assert_eq!(issue.title, "Fix the bug");
        assert_eq!(issue.priority, 2);
        assert!(issue.state.is_some());
        assert!(issue.team.is_some());
        assert!(issue.assignee.is_some());
    }

    #[test]
    fn test_issue_with_null_optional_fields() {
        let json = r#"{
            "id": "issue-abc",
            "identifier": "ENG-123",
            "title": "Fix the bug",
            "description": null,
            "priority": 0,
            "state": null,
            "team": null,
            "assignee": null,
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-02T00:00:00.000Z"
        }"#;
        let issue: Issue = serde_json::from_str(json).unwrap();
        assert!(issue.description.is_none());
        assert!(issue.state.is_none());
        assert!(issue.team.is_none());
        assert!(issue.assignee.is_none());
    }

    #[test]
    fn test_issue_connection_deserialization() {
        let json = r#"{
            "nodes": [
                {
                    "id": "issue-1",
                    "identifier": "ENG-1",
                    "title": "Issue 1",
                    "description": null,
                    "priority": 1,
                    "state": null,
                    "team": null,
                    "assignee": null,
                    "createdAt": "2024-01-01T00:00:00.000Z",
                    "updatedAt": "2024-01-01T00:00:00.000Z"
                },
                {
                    "id": "issue-2",
                    "identifier": "ENG-2",
                    "title": "Issue 2",
                    "description": null,
                    "priority": 2,
                    "state": null,
                    "team": null,
                    "assignee": null,
                    "createdAt": "2024-01-02T00:00:00.000Z",
                    "updatedAt": "2024-01-02T00:00:00.000Z"
                }
            ]
        }"#;
        let connection: IssueConnection = serde_json::from_str(json).unwrap();
        assert_eq!(connection.nodes.len(), 2);
        assert_eq!(connection.nodes[0].identifier, "ENG-1");
        assert_eq!(connection.nodes[1].identifier, "ENG-2");
    }

    #[test]
    fn test_viewer_response_deserialization() {
        let json = r#"{
            "viewer": {
                "id": "user-123",
                "name": "Test User",
                "email": "test@example.com",
                "displayName": "TU",
                "active": true
            }
        }"#;
        let response: ViewerResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.viewer.id, "user-123");
        assert_eq!(response.viewer.name, "Test User");
    }

    #[test]
    fn test_teams_response_deserialization() {
        let json = r#"{
            "teams": {
                "nodes": [
                    {
                        "id": "team-1",
                        "key": "ENG",
                        "name": "Engineering",
                        "description": null
                    }
                ]
            }
        }"#;
        let response: TeamsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.teams.nodes.len(), 1);
        assert_eq!(response.teams.nodes[0].key, "ENG");
    }

    #[test]
    fn test_issue_create_response_deserialization() {
        let json = r#"{
            "issueCreate": {
                "success": true,
                "issue": {
                    "id": "issue-new",
                    "identifier": "ENG-999",
                    "title": "New Issue",
                    "description": null,
                    "priority": 0,
                    "state": null,
                    "team": null,
                    "assignee": null,
                    "createdAt": "2024-01-01T00:00:00.000Z",
                    "updatedAt": "2024-01-01T00:00:00.000Z"
                }
            }
        }"#;
        let response: IssueCreateResponse = serde_json::from_str(json).unwrap();
        assert!(response.issue_create.success);
        assert!(response.issue_create.issue.is_some());
        assert_eq!(response.issue_create.issue.unwrap().identifier, "ENG-999");
    }

    #[test]
    fn test_issue_update_response_deserialization() {
        let json = r#"{
            "issueUpdate": {
                "success": true,
                "issue": {
                    "id": "issue-abc",
                    "identifier": "ENG-123",
                    "title": "Updated Title",
                    "description": null,
                    "priority": 1,
                    "state": null,
                    "team": null,
                    "assignee": null,
                    "createdAt": "2024-01-01T00:00:00.000Z",
                    "updatedAt": "2024-01-02T00:00:00.000Z"
                }
            }
        }"#;
        let response: IssueUpdateResponse = serde_json::from_str(json).unwrap();
        assert!(response.issue_update.success);
        assert_eq!(response.issue_update.issue.unwrap().title, "Updated Title");
    }

    #[test]
    fn test_project_deserialization() {
        let json = r#"{
            "id": "project-123",
            "name": "Q1 Roadmap",
            "description": "Quarterly roadmap project",
            "state": "started",
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-15T00:00:00.000Z",
            "targetDate": "2024-03-31",
            "startDate": "2024-01-01",
            "progress": 25.5
        }"#;
        let project: Project = serde_json::from_str(json).unwrap();
        assert_eq!(project.id, "project-123");
        assert_eq!(project.name, "Q1 Roadmap");
        assert_eq!(
            project.description,
            Some("Quarterly roadmap project".to_string())
        );
        assert_eq!(project.state, "started");
        assert_eq!(project.target_date, Some("2024-03-31".to_string()));
        assert_eq!(project.start_date, Some("2024-01-01".to_string()));
        assert!((project.progress - 25.5).abs() < 0.001);
    }

    #[test]
    fn test_project_with_null_optional_fields() {
        let json = r#"{
            "id": "project-456",
            "name": "Simple Project",
            "description": null,
            "state": "planned",
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-01T00:00:00.000Z",
            "targetDate": null,
            "startDate": null,
            "progress": 0.0
        }"#;
        let project: Project = serde_json::from_str(json).unwrap();
        assert!(project.description.is_none());
        assert!(project.target_date.is_none());
        assert!(project.start_date.is_none());
    }

    #[test]
    fn test_projects_response_deserialization() {
        let json = r#"{
            "projects": {
                "nodes": [
                    {
                        "id": "project-1",
                        "name": "Project Alpha",
                        "description": null,
                        "state": "started",
                        "createdAt": "2024-01-01T00:00:00.000Z",
                        "updatedAt": "2024-01-15T00:00:00.000Z",
                        "targetDate": null,
                        "startDate": null,
                        "progress": 50.0
                    }
                ]
            }
        }"#;
        let response: ProjectsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.projects.nodes.len(), 1);
        assert_eq!(response.projects.nodes[0].name, "Project Alpha");
    }

    #[test]
    fn test_cycle_deserialization() {
        let json = r#"{
            "id": "cycle-123",
            "number": 5,
            "name": "Sprint 5",
            "description": "Q1 Sprint",
            "startsAt": "2024-01-01T00:00:00.000Z",
            "endsAt": "2024-01-14T00:00:00.000Z",
            "completedAt": null,
            "progress": 50.5,
            "completedScopeHistory": [0.0, 10.0, 25.0],
            "scopeHistory": [100.0, 100.0, 100.0]
        }"#;
        let cycle: Cycle = serde_json::from_str(json).unwrap();
        assert_eq!(cycle.id, "cycle-123");
        assert_eq!(cycle.number, 5);
        assert_eq!(cycle.name, Some("Sprint 5".to_string()));
        assert_eq!(cycle.description, Some("Q1 Sprint".to_string()));
        assert!(cycle.starts_at.is_some());
        assert!(cycle.ends_at.is_some());
        assert!(cycle.completed_at.is_none());
        assert!((cycle.progress - 50.5).abs() < 0.001);
        assert_eq!(cycle.completed_scope_history.len(), 3);
        assert_eq!(cycle.scope_history.len(), 3);
    }

    #[test]
    fn test_cycle_with_null_optional_fields() {
        let json = r#"{
            "id": "cycle-456",
            "number": 1,
            "name": null,
            "description": null,
            "startsAt": null,
            "endsAt": null,
            "completedAt": null,
            "progress": 0.0,
            "completedScopeHistory": [],
            "scopeHistory": []
        }"#;
        let cycle: Cycle = serde_json::from_str(json).unwrap();
        assert_eq!(cycle.id, "cycle-456");
        assert_eq!(cycle.number, 1);
        assert!(cycle.name.is_none());
        assert!(cycle.description.is_none());
        assert!(cycle.starts_at.is_none());
        assert!(cycle.ends_at.is_none());
    }

    #[test]
    fn test_cycles_response_deserialization() {
        let json = r#"{
            "team": {
                "id": "team-123",
                "cycles": {
                    "nodes": [
                        {
                            "id": "cycle-1",
                            "number": 1,
                            "name": "Sprint 1",
                            "description": null,
                            "startsAt": "2024-01-01T00:00:00.000Z",
                            "endsAt": "2024-01-14T00:00:00.000Z",
                            "completedAt": null,
                            "progress": 75.0,
                            "completedScopeHistory": [],
                            "scopeHistory": []
                        }
                    ]
                }
            }
        }"#;
        let response: CyclesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.team.id, "team-123");
        assert_eq!(response.team.cycles.nodes.len(), 1);
        assert_eq!(response.team.cycles.nodes[0].number, 1);
    }

    #[test]
    fn test_label_deserialization() {
        let json = r##"{
            "id": "label-123",
            "name": "Bug",
            "description": "Bug reports",
            "color": "#ff0000",
            "isGroup": false,
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-15T00:00:00.000Z"
        }"##;
        let label: Label = serde_json::from_str(json).unwrap();
        assert_eq!(label.id, "label-123");
        assert_eq!(label.name, "Bug");
        assert_eq!(label.description, Some("Bug reports".to_string()));
        assert_eq!(label.color, "#ff0000");
        assert!(!label.is_group);
    }

    #[test]
    fn test_label_with_null_description() {
        let json = r##"{
            "id": "label-456",
            "name": "Feature",
            "description": null,
            "color": "#00ff00",
            "isGroup": true,
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-01T00:00:00.000Z"
        }"##;
        let label: Label = serde_json::from_str(json).unwrap();
        assert_eq!(label.id, "label-456");
        assert_eq!(label.name, "Feature");
        assert!(label.description.is_none());
        assert!(label.is_group);
    }

    #[test]
    fn test_labels_response_deserialization() {
        let json = r##"{
            "issueLabels": {
                "nodes": [
                    {
                        "id": "label-1",
                        "name": "Bug",
                        "description": null,
                        "color": "#ff0000",
                        "isGroup": false,
                        "createdAt": "2024-01-01T00:00:00.000Z",
                        "updatedAt": "2024-01-01T00:00:00.000Z"
                    },
                    {
                        "id": "label-2",
                        "name": "Feature",
                        "description": "Feature requests",
                        "color": "#00ff00",
                        "isGroup": false,
                        "createdAt": "2024-01-02T00:00:00.000Z",
                        "updatedAt": "2024-01-02T00:00:00.000Z"
                    }
                ]
            }
        }"##;
        let response: LabelsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.issue_labels.nodes.len(), 2);
        assert_eq!(response.issue_labels.nodes[0].name, "Bug");
        assert_eq!(response.issue_labels.nodes[1].name, "Feature");
    }

    #[test]
    fn test_label_response_deserialization() {
        let json = r##"{
            "issueLabel": {
                "id": "label-123",
                "name": "Bug",
                "description": "Bug reports",
                "color": "#ff0000",
                "isGroup": false,
                "createdAt": "2024-01-01T00:00:00.000Z",
                "updatedAt": "2024-01-15T00:00:00.000Z"
            }
        }"##;
        let response: LabelResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.issue_label.id, "label-123");
        assert_eq!(response.issue_label.name, "Bug");
    }

    #[test]
    fn test_document_deserialization() {
        let json = r##"{
            "id": "doc-123",
            "title": "Project Overview",
            "icon": "document",
            "color": "#0066ff",
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-15T00:00:00.000Z",
            "creator": {
                "id": "user-1",
                "name": "John Doe",
                "email": "john@example.com",
                "displayName": "JD",
                "active": true
            },
            "project": {
                "id": "project-1",
                "name": "Q1 Roadmap"
            }
        }"##;
        let document: Document = serde_json::from_str(json).unwrap();
        assert_eq!(document.id, "doc-123");
        assert_eq!(document.title, "Project Overview");
        assert_eq!(document.icon, Some("document".to_string()));
        assert!(document.creator.is_some());
        assert!(document.project.is_some());
    }

    #[test]
    fn test_document_with_null_optional_fields() {
        let json = r#"{
            "id": "doc-456",
            "title": "Simple Doc",
            "icon": null,
            "color": null,
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-01T00:00:00.000Z",
            "creator": null,
            "project": null
        }"#;
        let document: Document = serde_json::from_str(json).unwrap();
        assert_eq!(document.id, "doc-456");
        assert!(document.icon.is_none());
        assert!(document.color.is_none());
        assert!(document.creator.is_none());
        assert!(document.project.is_none());
    }

    #[test]
    fn test_document_with_content_deserialization() {
        let json = r#"{
            "id": "doc-789",
            "title": "Technical Spec",
            "content": "Overview content here.",
            "icon": null,
            "color": null,
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-15T00:00:00.000Z",
            "creator": null,
            "project": null
        }"#;
        let document: DocumentWithContent = serde_json::from_str(json).unwrap();
        assert_eq!(document.id, "doc-789");
        assert_eq!(document.title, "Technical Spec");
        assert_eq!(document.content, Some("Overview content here.".to_string()));
    }

    #[test]
    fn test_documents_response_deserialization() {
        let json = r#"{
            "documents": {
                "nodes": [
                    {
                        "id": "doc-1",
                        "title": "Doc One",
                        "icon": null,
                        "color": null,
                        "createdAt": "2024-01-01T00:00:00.000Z",
                        "updatedAt": "2024-01-01T00:00:00.000Z",
                        "creator": null,
                        "project": null
                    }
                ]
            }
        }"#;
        let response: DocumentsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.documents.nodes.len(), 1);
        assert_eq!(response.documents.nodes[0].title, "Doc One");
    }

    #[test]
    fn test_document_create_response_deserialization() {
        let json = r#"{
            "documentCreate": {
                "success": true,
                "document": {
                    "id": "doc-new",
                    "title": "New Document",
                    "content": "Content here",
                    "icon": null,
                    "color": null,
                    "createdAt": "2024-01-01T00:00:00.000Z",
                    "updatedAt": "2024-01-01T00:00:00.000Z",
                    "creator": null,
                    "project": null
                }
            }
        }"#;
        let response: DocumentCreateResponse = serde_json::from_str(json).unwrap();
        assert!(response.document_create.success);
        assert!(response.document_create.document.is_some());
        assert_eq!(
            response.document_create.document.unwrap().title,
            "New Document"
        );
    }

    #[test]
    fn test_attachment_deserialization() {
        let json = r#"{
            "id": "attach-123",
            "title": "Screenshot.png",
            "subtitle": "Bug screenshot",
            "url": "https://example.com/screenshot.png",
            "metadata": {"size": 1024},
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-15T00:00:00.000Z",
            "creator": {
                "id": "user-1",
                "name": "John Doe",
                "email": "john@example.com",
                "displayName": "JD",
                "active": true
            }
        }"#;
        let attachment: Attachment = serde_json::from_str(json).unwrap();
        assert_eq!(attachment.id, "attach-123");
        assert_eq!(attachment.title, "Screenshot.png");
        assert_eq!(attachment.subtitle, Some("Bug screenshot".to_string()));
        assert_eq!(attachment.url, "https://example.com/screenshot.png");
        assert!(attachment.metadata.is_some());
        assert!(attachment.creator.is_some());
    }

    #[test]
    fn test_attachment_with_null_optional_fields() {
        let json = r#"{
            "id": "attach-456",
            "title": "Document.pdf",
            "subtitle": null,
            "url": "https://example.com/doc.pdf",
            "metadata": null,
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-01T00:00:00.000Z",
            "creator": null
        }"#;
        let attachment: Attachment = serde_json::from_str(json).unwrap();
        assert_eq!(attachment.id, "attach-456");
        assert!(attachment.subtitle.is_none());
        assert!(attachment.metadata.is_none());
        assert!(attachment.creator.is_none());
    }

    #[test]
    fn test_attachment_with_issue_deserialization() {
        let json = r#"{
            "id": "attach-789",
            "title": "Log file",
            "subtitle": null,
            "url": "https://example.com/log.txt",
            "metadata": null,
            "createdAt": "2024-01-01T00:00:00.000Z",
            "updatedAt": "2024-01-01T00:00:00.000Z",
            "creator": null,
            "issue": {
                "id": "issue-123",
                "identifier": "ENG-456"
            }
        }"#;
        let attachment: AttachmentWithIssue = serde_json::from_str(json).unwrap();
        assert_eq!(attachment.id, "attach-789");
        assert!(attachment.issue.is_some());
        assert_eq!(attachment.issue.as_ref().unwrap().identifier, "ENG-456");
    }

    #[test]
    fn test_issue_attachments_response_deserialization() {
        let json = r#"{
            "issue": {
                "id": "issue-123",
                "identifier": "ENG-456",
                "attachments": {
                    "nodes": [
                        {
                            "id": "attach-1",
                            "title": "File1.png",
                            "subtitle": null,
                            "url": "https://example.com/file1.png",
                            "metadata": null,
                            "createdAt": "2024-01-01T00:00:00.000Z",
                            "updatedAt": "2024-01-01T00:00:00.000Z",
                            "creator": null
                        }
                    ]
                }
            }
        }"#;
        let response: IssueAttachmentsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.issue.identifier, "ENG-456");
        assert_eq!(response.issue.attachments.nodes.len(), 1);
        assert_eq!(response.issue.attachments.nodes[0].title, "File1.png");
    }

    #[test]
    fn test_attachment_create_response_deserialization() {
        let json = r#"{
            "attachmentCreate": {
                "success": true,
                "attachment": {
                    "id": "attach-new",
                    "title": "Uploaded.png",
                    "subtitle": null,
                    "url": "https://example.com/uploaded.png",
                    "metadata": null,
                    "createdAt": "2024-01-01T00:00:00.000Z",
                    "updatedAt": "2024-01-01T00:00:00.000Z",
                    "creator": null
                }
            }
        }"#;
        let response: AttachmentCreateResponse = serde_json::from_str(json).unwrap();
        assert!(response.attachment_create.success);
        assert!(response.attachment_create.attachment.is_some());
        assert_eq!(
            response.attachment_create.attachment.unwrap().title,
            "Uploaded.png"
        );
    }

    #[test]
    fn test_file_upload_response_deserialization() {
        let json = r#"{
            "fileUpload": {
                "uploadFile": {
                    "uploadUrl": "https://upload.example.com/presigned",
                    "assetUrl": "https://assets.example.com/file.png",
                    "headers": [
                        {
                            "key": "Content-Type",
                            "value": "image/png"
                        }
                    ]
                }
            }
        }"#;
        let response: FileUploadResponse = serde_json::from_str(json).unwrap();
        assert_eq!(
            response.file_upload.upload_file.upload_url,
            "https://upload.example.com/presigned"
        );
        assert_eq!(
            response.file_upload.upload_file.asset_url,
            "https://assets.example.com/file.png"
        );
        assert_eq!(response.file_upload.upload_file.headers.len(), 1);
        assert_eq!(
            response.file_upload.upload_file.headers[0].key,
            "Content-Type"
        );
    }
}
