//! Git integration commands.
//!
//! Commands for linking git branches and pull requests to Linear issues.
//! Linear tracks these links through its attachment system, recognizing
//! specific URL patterns for branches and PRs.

use crate::api::{queries, GraphQLClient};
use crate::commands::issue::is_uuid;
use crate::error::LinError;
use crate::models::{
    Attachment, AttachmentCreateResponse, IssueAttachmentsResponse, IssuesResponse,
};
use crate::output::{output, HumanDisplay, OutputFormat};
use crate::Result;
use serde::Serialize;

/// A git link (branch or PR) attached to an issue.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitLink {
    /// Unique identifier for the attachment.
    pub id: String,
    /// The link title (branch name or PR title).
    pub title: String,
    /// Optional subtitle/description.
    pub subtitle: Option<String>,
    /// The URL (branch URL or PR URL).
    pub url: String,
    /// Type of git link (branch or pull_request).
    pub link_type: GitLinkType,
    /// ISO 8601 timestamp of when the link was created.
    pub created_at: String,
}

/// Type of git link.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GitLinkType {
    /// A git branch.
    Branch,
    /// A pull request or merge request.
    PullRequest,
    /// Unknown link type (generic URL).
    Unknown,
}

impl GitLinkType {
    /// Detect the link type from a URL.
    pub fn from_url(url: &str) -> Self {
        let url_lower = url.to_lowercase();

        // Check for pull request patterns
        if url_lower.contains("/pull/")           // GitHub PR
            || url_lower.contains("/pulls/")      // GitHub PRs list
            || url_lower.contains("/merge_requests/")  // GitLab MR
            || url_lower.contains("/-/merge_requests/")
        // GitLab MR alternate
        {
            return GitLinkType::PullRequest;
        }

        // Check for branch patterns
        if url_lower.contains("/tree/")           // GitHub/GitLab branch
            || url_lower.contains("/-/tree/")     // GitLab branch
            || url_lower.contains("/src/branch/") // Gitea branch
            || url_lower.contains("/branch/")
        // Generic branch URL
        {
            return GitLinkType::Branch;
        }

        GitLinkType::Unknown
    }
}

impl HumanDisplay for GitLink {
    fn human_fmt(&self) -> String {
        use colored::Colorize;

        let type_label = match self.link_type {
            GitLinkType::Branch => "[Branch]".green(),
            GitLinkType::PullRequest => "[PR]".magenta(),
            GitLinkType::Unknown => "[Link]".dimmed(),
        };

        let mut parts = vec![format!("{} {}", type_label, self.title.bold())];
        parts.push(format!("  {}: {}", "URL".dimmed(), self.url.cyan()));

        if let Some(subtitle) = &self.subtitle {
            parts.push(format!("  {}: {}", "Description".dimmed(), subtitle));
        }

        // Format date (show date portion only)
        let date = if self.created_at.len() >= 10 {
            &self.created_at[..10]
        } else {
            &self.created_at
        };
        parts.push(format!("  {}: {}", "Created".dimmed(), date));

        parts.join("\n")
    }
}

impl From<Attachment> for GitLink {
    fn from(attachment: Attachment) -> Self {
        let link_type = GitLinkType::from_url(&attachment.url);
        GitLink {
            id: attachment.id,
            title: attachment.title,
            subtitle: attachment.subtitle,
            url: attachment.url,
            link_type,
            created_at: attachment.created_at,
        }
    }
}

/// Resolve an issue identifier or UUID to an issue ID.
///
/// If the identifier is a UUID, it's returned as-is.
/// If it's an identifier like "ENG-123", we look it up via the API.
fn resolve_issue_id(client: &GraphQLClient, identifier: &str) -> Result<String> {
    if is_uuid(identifier) {
        return Ok(identifier.to_string());
    }

    // Look up by identifier (e.g., "ENG-123")
    let variables = serde_json::json!({
        "filter": {
            "identifier": { "eq": identifier }
        }
    });

    let response: IssuesResponse = client.query(queries::ISSUE_BY_IDENTIFIER_QUERY, variables)?;

    response
        .issues
        .nodes
        .first()
        .map(|issue| issue.id.clone())
        .ok_or_else(|| LinError::api(format!("Issue '{}' not found", identifier)))
}

/// Check if a URL is a git-related link.
fn is_git_link(url: &str) -> bool {
    let url_lower = url.to_lowercase();

    // GitHub patterns
    url_lower.contains("github.com")
        // GitLab patterns
        || url_lower.contains("gitlab.com")
        || url_lower.contains("gitlab.")
        // Bitbucket patterns
        || url_lower.contains("bitbucket.org")
        || url_lower.contains("bitbucket.")
        // Generic git patterns
        || url_lower.contains("/tree/")
        || url_lower.contains("/pull/")
        || url_lower.contains("/merge_requests/")
        || url_lower.contains("/branch/")
}

/// Link a git branch to an issue.
///
/// Creates an attachment with a branch URL that Linear will recognize.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `issue_identifier` - The issue ID or identifier (e.g., "ENG-123")
/// * `branch_name` - The name of the branch to link
/// * `repo_url` - Optional repository URL (defaults to a generic branch reference)
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```ignore
/// use lin::api::GraphQLClient;
/// use lin::commands::git::link_branch;
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// link_branch(&client, "ENG-123", "feature/my-branch", None, OutputFormat::Human)?;
/// ```
pub fn link_branch(
    client: &GraphQLClient,
    issue_identifier: &str,
    branch_name: &str,
    repo_url: Option<&str>,
    format: OutputFormat,
) -> Result<()> {
    let issue_id = resolve_issue_id(client, issue_identifier)?;

    // Construct the branch URL
    // If repo_url is provided, append the branch path
    // Otherwise, create a minimal reference
    let url = match repo_url {
        Some(repo) => {
            let repo = repo.trim_end_matches('/');
            format!("{}/tree/{}", repo, branch_name)
        }
        None => {
            // Create a generic branch reference that Linear can display
            format!("branch://{}", branch_name)
        }
    };

    let variables = serde_json::json!({
        "input": {
            "issueId": issue_id,
            "title": branch_name,
            "url": url,
            "subtitle": "Git branch"
        }
    });

    let response: AttachmentCreateResponse =
        client.query(queries::ATTACHMENT_CREATE_MUTATION, variables)?;

    if let Some(attachment) = response.attachment_create.attachment {
        let git_link: GitLink = attachment.into();
        output(&git_link, format);
    }
    Ok(())
}

/// Link a pull request URL to an issue.
///
/// Creates an attachment with the PR URL that Linear will recognize.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `issue_identifier` - The issue ID or identifier (e.g., "ENG-123")
/// * `pr_url` - The full URL of the pull request
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```ignore
/// use lin::api::GraphQLClient;
/// use lin::commands::git::link_pr;
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// link_pr(&client, "ENG-123", "https://github.com/org/repo/pull/42", OutputFormat::Human)?;
/// ```
pub fn link_pr(
    client: &GraphQLClient,
    issue_identifier: &str,
    pr_url: &str,
    format: OutputFormat,
) -> Result<()> {
    let issue_id = resolve_issue_id(client, issue_identifier)?;

    // Extract PR title from URL if possible, otherwise use a generic title
    let pr_title = extract_pr_title(pr_url);

    let variables = serde_json::json!({
        "input": {
            "issueId": issue_id,
            "title": pr_title,
            "url": pr_url,
            "subtitle": "Pull Request"
        }
    });

    let response: AttachmentCreateResponse =
        client.query(queries::ATTACHMENT_CREATE_MUTATION, variables)?;

    if let Some(attachment) = response.attachment_create.attachment {
        let git_link: GitLink = attachment.into();
        output(&git_link, format);
    }
    Ok(())
}

/// Extract a title from a PR URL.
fn extract_pr_title(pr_url: &str) -> String {
    // Try to extract the PR number from common URL patterns
    // GitHub: https://github.com/owner/repo/pull/123
    // GitLab: https://gitlab.com/owner/repo/-/merge_requests/123

    if let Some(pr_num) = extract_pr_number(pr_url) {
        return format!("PR #{}", pr_num);
    }

    "Pull Request".to_string()
}

/// Extract PR/MR number from URL.
fn extract_pr_number(url: &str) -> Option<&str> {
    // GitHub pattern: /pull/<number>
    if let Some(idx) = url.find("/pull/") {
        let after = &url[idx + 6..];
        let end = after
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(after.len());
        if end > 0 {
            return Some(&after[..end]);
        }
    }

    // GitLab pattern: /merge_requests/<number>
    if let Some(idx) = url.find("/merge_requests/") {
        let after = &url[idx + 16..];
        let end = after
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(after.len());
        if end > 0 {
            return Some(&after[..end]);
        }
    }

    None
}

/// List all git-related links (branches and PRs) for an issue.
///
/// Fetches all attachments for the issue and filters to show only
/// git-related links based on URL patterns.
///
/// # Arguments
///
/// * `client` - The GraphQL client to use for the API request
/// * `issue_identifier` - The issue ID or identifier (e.g., "ENG-123")
/// * `format` - The output format (Human or Json)
///
/// # Example
///
/// ```ignore
/// use lin::api::GraphQLClient;
/// use lin::commands::git::list_links;
/// use lin::output::OutputFormat;
///
/// let client = GraphQLClient::new("lin_api_xxxxx");
/// list_links(&client, "ENG-123", OutputFormat::Human)?;
/// ```
pub fn list_links(
    client: &GraphQLClient,
    issue_identifier: &str,
    format: OutputFormat,
) -> Result<()> {
    let issue_id = resolve_issue_id(client, issue_identifier)?;

    let variables = serde_json::json!({
        "id": issue_id
    });

    let response: IssueAttachmentsResponse =
        client.query(queries::ISSUE_GIT_LINKS_QUERY, variables)?;

    // Filter attachments to only include git-related links
    let git_links: Vec<GitLink> = response
        .issue
        .attachments
        .nodes
        .into_iter()
        .filter(|a| {
            is_git_link(&a.url)
                || a.subtitle.as_deref() == Some("Git branch")
                || a.subtitle.as_deref() == Some("Pull Request")
        })
        .map(GitLink::from)
        .collect();

    output(&git_links, format);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::GraphQLClient;

    #[test]
    fn test_git_link_type_from_url_github_pr() {
        assert_eq!(
            GitLinkType::from_url("https://github.com/owner/repo/pull/123"),
            GitLinkType::PullRequest
        );
    }

    #[test]
    fn test_git_link_type_from_url_gitlab_mr() {
        assert_eq!(
            GitLinkType::from_url("https://gitlab.com/owner/repo/-/merge_requests/456"),
            GitLinkType::PullRequest
        );
    }

    #[test]
    fn test_git_link_type_from_url_github_branch() {
        assert_eq!(
            GitLinkType::from_url("https://github.com/owner/repo/tree/feature-branch"),
            GitLinkType::Branch
        );
    }

    #[test]
    fn test_git_link_type_from_url_gitlab_branch() {
        assert_eq!(
            GitLinkType::from_url("https://gitlab.com/owner/repo/-/tree/main"),
            GitLinkType::Branch
        );
    }

    #[test]
    fn test_git_link_type_from_url_unknown() {
        assert_eq!(
            GitLinkType::from_url("https://example.com/some/page"),
            GitLinkType::Unknown
        );
    }

    #[test]
    fn test_is_git_link_github() {
        assert!(is_git_link("https://github.com/owner/repo/pull/123"));
        assert!(is_git_link("https://github.com/owner/repo/tree/main"));
    }

    #[test]
    fn test_is_git_link_gitlab() {
        assert!(is_git_link(
            "https://gitlab.com/owner/repo/-/merge_requests/456"
        ));
        assert!(is_git_link("https://gitlab.com/owner/repo/-/tree/develop"));
    }

    #[test]
    fn test_is_git_link_bitbucket() {
        assert!(is_git_link(
            "https://bitbucket.org/owner/repo/pull-requests/789"
        ));
    }

    #[test]
    fn test_is_git_link_non_git() {
        assert!(!is_git_link("https://example.com/document.pdf"));
        assert!(!is_git_link("https://linear.app/workspace/issue/ENG-123"));
    }

    #[test]
    fn test_extract_pr_number_github() {
        assert_eq!(
            extract_pr_number("https://github.com/owner/repo/pull/123"),
            Some("123")
        );
        assert_eq!(
            extract_pr_number("https://github.com/owner/repo/pull/42/files"),
            Some("42")
        );
    }

    #[test]
    fn test_extract_pr_number_gitlab() {
        assert_eq!(
            extract_pr_number("https://gitlab.com/owner/repo/-/merge_requests/456"),
            Some("456")
        );
    }

    #[test]
    fn test_extract_pr_number_no_match() {
        assert_eq!(
            extract_pr_number("https://github.com/owner/repo/tree/main"),
            None
        );
    }

    #[test]
    fn test_extract_pr_title() {
        assert_eq!(
            extract_pr_title("https://github.com/owner/repo/pull/123"),
            "PR #123"
        );
        assert_eq!(
            extract_pr_title("https://example.com/something"),
            "Pull Request"
        );
    }

    #[test]
    fn test_link_branch_success() {
        let mut server = mockito::Server::new();

        // Mock for issue identifier resolution
        let issue_mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .match_header("content-type", "application/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issues": {
                            "nodes": [
                                {
                                    "id": "issue-123",
                                    "identifier": "ENG-456",
                                    "title": "Test Issue",
                                    "description": null,
                                    "priority": 0,
                                    "state": null,
                                    "team": null,
                                    "assignee": null,
                                    "createdAt": "2024-01-01T00:00:00.000Z",
                                    "updatedAt": "2024-01-01T00:00:00.000Z"
                                }
                            ]
                        }
                    }
                }"#,
            )
            .create();

        // Mock for attachment creation
        let attachment_mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "attachmentCreate": {
                            "success": true,
                            "attachment": {
                                "id": "attach-1",
                                "title": "feature/my-branch",
                                "subtitle": "Git branch",
                                "url": "https://github.com/org/repo/tree/feature/my-branch",
                                "metadata": null,
                                "createdAt": "2024-01-01T00:00:00.000Z",
                                "updatedAt": "2024-01-01T00:00:00.000Z",
                                "creator": null
                            }
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());

        let result = link_branch(
            &client,
            "ENG-456",
            "feature/my-branch",
            Some("https://github.com/org/repo"),
            OutputFormat::Human,
        );

        assert!(result.is_ok());
        issue_mock.assert();
        attachment_mock.assert();
    }

    #[test]
    fn test_link_pr_success() {
        let mut server = mockito::Server::new();

        // Mock for issue identifier resolution
        let issue_mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issues": {
                            "nodes": [
                                {
                                    "id": "issue-123",
                                    "identifier": "ENG-456",
                                    "title": "Test Issue",
                                    "description": null,
                                    "priority": 0,
                                    "state": null,
                                    "team": null,
                                    "assignee": null,
                                    "createdAt": "2024-01-01T00:00:00.000Z",
                                    "updatedAt": "2024-01-01T00:00:00.000Z"
                                }
                            ]
                        }
                    }
                }"#,
            )
            .create();

        // Mock for attachment creation
        let attachment_mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "attachmentCreate": {
                            "success": true,
                            "attachment": {
                                "id": "attach-2",
                                "title": "PR #42",
                                "subtitle": "Pull Request",
                                "url": "https://github.com/org/repo/pull/42",
                                "metadata": null,
                                "createdAt": "2024-01-01T00:00:00.000Z",
                                "updatedAt": "2024-01-01T00:00:00.000Z",
                                "creator": null
                            }
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());

        let result = link_pr(
            &client,
            "ENG-456",
            "https://github.com/org/repo/pull/42",
            OutputFormat::Human,
        );

        assert!(result.is_ok());
        issue_mock.assert();
        attachment_mock.assert();
    }

    #[test]
    fn test_list_links_success() {
        let mut server = mockito::Server::new();

        // Mock for issue identifier resolution
        let issue_mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issues": {
                            "nodes": [
                                {
                                    "id": "issue-123",
                                    "identifier": "ENG-456",
                                    "title": "Test Issue",
                                    "description": null,
                                    "priority": 0,
                                    "state": null,
                                    "team": null,
                                    "assignee": null,
                                    "createdAt": "2024-01-01T00:00:00.000Z",
                                    "updatedAt": "2024-01-01T00:00:00.000Z"
                                }
                            ]
                        }
                    }
                }"#,
            )
            .create();

        // Mock for attachments query
        let attachments_mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issue": {
                            "id": "issue-123",
                            "identifier": "ENG-456",
                            "attachments": {
                                "nodes": [
                                    {
                                        "id": "attach-1",
                                        "title": "feature/my-branch",
                                        "subtitle": "Git branch",
                                        "url": "https://github.com/org/repo/tree/feature/my-branch",
                                        "metadata": null,
                                        "createdAt": "2024-01-01T00:00:00.000Z",
                                        "updatedAt": "2024-01-01T00:00:00.000Z",
                                        "creator": null
                                    },
                                    {
                                        "id": "attach-2",
                                        "title": "PR #42",
                                        "subtitle": "Pull Request",
                                        "url": "https://github.com/org/repo/pull/42",
                                        "metadata": null,
                                        "createdAt": "2024-01-02T00:00:00.000Z",
                                        "updatedAt": "2024-01-02T00:00:00.000Z",
                                        "creator": null
                                    },
                                    {
                                        "id": "attach-3",
                                        "title": "screenshot.png",
                                        "subtitle": null,
                                        "url": "https://example.com/screenshot.png",
                                        "metadata": null,
                                        "createdAt": "2024-01-03T00:00:00.000Z",
                                        "updatedAt": "2024-01-03T00:00:00.000Z",
                                        "creator": null
                                    }
                                ]
                            }
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());

        let result = list_links(&client, "ENG-456", OutputFormat::Human);

        assert!(result.is_ok());
        issue_mock.assert();
        attachments_mock.assert();
    }

    #[test]
    fn test_list_links_with_uuid() {
        let mut server = mockito::Server::new();

        // Mock for attachments query (no identifier resolution needed for UUID)
        let mock = server
            .mock("POST", "/")
            .match_header("authorization", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issue": {
                            "id": "550e8400-e29b-41d4-a716-446655440000",
                            "identifier": "ENG-789",
                            "attachments": {
                                "nodes": []
                            }
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());

        let result = list_links(
            &client,
            "550e8400-e29b-41d4-a716-446655440000",
            OutputFormat::Human,
        );

        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_link_branch_issue_not_found() {
        let mut server = mockito::Server::new();

        // Mock for issue identifier resolution - returns empty
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "issues": {
                            "nodes": []
                        }
                    }
                }"#,
            )
            .create();

        let client = GraphQLClient::with_url("test-token", &server.url());

        let result = link_branch(
            &client,
            "NONEXISTENT-999",
            "feature/branch",
            None,
            OutputFormat::Human,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not found"));

        mock.assert();
    }

    #[test]
    fn test_git_link_human_display() {
        let link = GitLink {
            id: "attach-123".to_string(),
            title: "feature/my-branch".to_string(),
            subtitle: Some("Git branch".to_string()),
            url: "https://github.com/org/repo/tree/feature/my-branch".to_string(),
            link_type: GitLinkType::Branch,
            created_at: "2024-01-15T00:00:00.000Z".to_string(),
        };

        let output = link.human_fmt();
        assert!(output.contains("feature/my-branch"));
        assert!(output.contains("https://github.com/org/repo/tree/feature/my-branch"));
        assert!(output.contains("Git branch"));
        assert!(output.contains("2024-01-15"));
    }
}
