//! lin - A command-line interface for Linear
//!
//! Entry point for the CLI application.

use clap::{Parser, Subcommand};
use lin::api::GraphQLClient;
use lin::auth::require_api_token;
use lin::commands::{
    attachment, comment, cycle, document, git, issue, label, org, project, relation, search, team,
    user, workflow,
};
use lin::config::Config;
use lin::output::{init_colors, output_error_with_format, output_success, OutputFormat};
use serde::Serialize;

/// lin - A command-line interface for Linear
#[derive(Parser, Debug)]
#[command(name = "lin")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(after_help = "EXAMPLES:\n  \
lin issue list --team ENG --assignee me\n  \
lin issue get ENG-123\n  \
lin user me\n  \
lin --json issue list | jq '.data[].identifier'")]
struct Cli {
    /// Linear API token (can also be set via LINEAR_API_TOKEN env var)
    #[arg(long, global = true)]
    api_token: Option<String>,

    /// Organization to use (uses default if not specified)
    #[arg(long, short, global = true)]
    org: Option<String>,

    /// Output in JSON format (default: human-friendly output)
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

/// Top-level commands for the lin CLI.
#[derive(Subcommand, Debug)]
enum Commands {
    /// Manage issues
    Issue {
        #[command(subcommand)]
        command: IssueCommands,
    },
    /// Manage comments on issues
    Comment {
        #[command(subcommand)]
        command: CommentCommands,
    },
    /// Manage attachments on issues
    Attachment {
        #[command(subcommand)]
        command: AttachmentCommands,
    },
    /// Manage teams
    Team {
        #[command(subcommand)]
        command: TeamCommands,
    },
    /// Get information about the current user
    User {
        #[command(subcommand)]
        command: UserCommands,
    },
    /// Manage organization configuration
    Org {
        #[command(subcommand)]
        command: OrgCommands,
    },
    /// Manage workflow states
    Workflow {
        #[command(subcommand)]
        command: WorkflowCommands,
    },
    /// Manage projects
    Project {
        #[command(subcommand)]
        command: ProjectCommands,
    },
    /// Manage cycles (sprints)
    Cycle {
        #[command(subcommand)]
        command: CycleCommands,
    },
    /// Manage labels
    Label {
        #[command(subcommand)]
        command: LabelCommands,
    },
    /// Manage documents
    Document {
        #[command(subcommand)]
        command: DocumentCommands,
    },
    /// Search for issues
    #[command(after_help = "EXAMPLES:\n  \
    lin search \"authentication bug\"\n  \
    lin search \"fix login\" --team ENG --limit 10\n  \
    lin search \"urgent\" --assignee me --state \"In Progress\"")]
    Search {
        /// The search query string
        query: String,
        /// Filter by team identifier (e.g., "ENG")
        #[arg(long)]
        team: Option<String>,
        /// Filter by assignee (user ID or "me" for current user)
        #[arg(long)]
        assignee: Option<String>,
        /// Filter by state name (e.g., "In Progress", "Done")
        #[arg(long)]
        state: Option<String>,
        /// Maximum number of results to return
        #[arg(long, default_value = "50")]
        limit: u32,
    },
}

/// Issue-related subcommands.
#[derive(Subcommand, Debug)]
enum IssueCommands {
    /// List issues
    #[command(after_help = "EXAMPLES:\n  \
    lin issue list --team ENG --assignee me\n  \
    lin issue list --state \"In Progress\" --limit 10\n  \
    lin issue list --project <project-id>\n  \
    lin issue list --cycle <cycle-id>\n  \
    lin issue list --label <label-id>\n  \
    lin issue list --created-after 2024-01-01\n  \
    lin issue list --updated-before 2024-12-31\n  \
    lin issue list --sort priority --order asc\n  \
    lin issue list --sort updated --order desc")]
    List {
        /// Filter by team identifier
        #[arg(long)]
        team: Option<String>,
        /// Filter by assignee
        #[arg(long)]
        assignee: Option<String>,
        /// Filter by state (e.g., "In Progress", "Done")
        #[arg(long)]
        state: Option<String>,
        /// Filter by project ID
        #[arg(long)]
        project: Option<String>,
        /// Filter by cycle ID
        #[arg(long)]
        cycle: Option<String>,
        /// Filter by label ID
        #[arg(long)]
        label: Option<String>,
        /// Maximum number of issues to return
        #[arg(long, default_value = "50")]
        limit: u32,
        /// Filter issues created after this date (YYYY-MM-DD)
        #[arg(long)]
        created_after: Option<String>,
        /// Filter issues created before this date (YYYY-MM-DD)
        #[arg(long)]
        created_before: Option<String>,
        /// Filter issues updated after this date (YYYY-MM-DD)
        #[arg(long)]
        updated_after: Option<String>,
        /// Filter issues updated before this date (YYYY-MM-DD)
        #[arg(long)]
        updated_before: Option<String>,
        /// Sort by field (priority, created, updated, title)
        #[arg(long)]
        sort: Option<String>,
        /// Sort direction (asc, desc). Uses field-appropriate default if not specified
        #[arg(long)]
        order: Option<String>,
    },
    /// Get details of a specific issue
    #[command(after_help = "EXAMPLES:\n  \
    lin issue get ENG-123\n  \
    lin issue get ENG-123 --with-comments")]
    Get {
        /// Issue identifier (e.g., "ENG-123")
        identifier: String,
        /// Include comments in the output
        #[arg(long)]
        with_comments: bool,
    },
    /// Create a new issue
    #[command(after_help = "EXAMPLES:\n  \
    lin issue create --team <team-id> --title \"Fix bug\" --priority 2\n  \
    lin issue create --team <team-id> --title \"New feature\" --labels <label-id1> --labels <label-id2>")]
    Create {
        /// Issue title
        #[arg(long)]
        title: String,
        /// Team ID (UUID of the team)
        #[arg(long)]
        team: String,
        /// Issue description (optional)
        #[arg(long)]
        description: Option<String>,
        /// Assignee user ID (optional)
        #[arg(long)]
        assignee: Option<String>,
        /// Initial state ID (optional)
        #[arg(long)]
        state: Option<String>,
        /// Priority (0-4: 0=none, 1=urgent, 2=high, 3=normal, 4=low)
        #[arg(long)]
        priority: Option<u8>,
        /// Label IDs to add to the issue (can be specified multiple times)
        #[arg(long)]
        labels: Option<Vec<String>>,
    },
    /// Update an existing issue
    #[command(after_help = "EXAMPLES:\n  \
    lin issue update ENG-123 --title \"New title\"\n  \
    lin issue update ENG-123 --state <state-id> --priority 1\n  \
    lin issue update ENG-123 --labels <label-id1> --labels <label-id2>")]
    Update {
        /// Issue identifier (e.g., "ENG-123") or UUID
        identifier: String,
        /// New title
        #[arg(long)]
        title: Option<String>,
        /// New description
        #[arg(long)]
        description: Option<String>,
        /// Assignee user ID
        #[arg(long)]
        assignee: Option<String>,
        /// New state ID
        #[arg(long)]
        state: Option<String>,
        /// New priority (0-4: 0=none, 1=urgent, 2=high, 3=normal, 4=low)
        #[arg(long)]
        priority: Option<u8>,
        /// Label IDs to set on the issue (replaces existing labels, can be specified multiple times)
        #[arg(long)]
        labels: Option<Vec<String>>,
    },
    /// Delete an issue
    #[command(after_help = "EXAMPLES:\n  \
    lin issue delete ENG-123")]
    Delete {
        /// Issue identifier (e.g., "ENG-123") or UUID
        identifier: String,
    },
    /// Archive an issue
    #[command(after_help = "EXAMPLES:\n  \
    lin issue archive ENG-123")]
    Archive {
        /// Issue identifier (e.g., "ENG-123") or UUID
        identifier: String,
    },
    /// Unarchive an issue
    #[command(after_help = "EXAMPLES:\n  \
    lin issue unarchive ENG-123")]
    Unarchive {
        /// Issue identifier (e.g., "ENG-123") or UUID
        identifier: String,
    },
    /// Link a git branch to an issue
    #[command(after_help = "EXAMPLES:\n  \
    lin issue link-branch ENG-123 feature/my-feature\n  \
    lin issue link-branch ENG-123 feature/my-feature --repo https://github.com/org/repo")]
    LinkBranch {
        /// Issue identifier (e.g., "ENG-123") or UUID
        identifier: String,
        /// Name of the branch to link
        branch: String,
        /// Repository URL (optional, for constructing the branch URL)
        #[arg(long)]
        repo: Option<String>,
    },
    /// Link a pull request URL to an issue
    #[command(after_help = "EXAMPLES:\n  \
    lin issue link-pr ENG-123 https://github.com/org/repo/pull/42")]
    LinkPr {
        /// Issue identifier (e.g., "ENG-123") or UUID
        identifier: String,
        /// Full URL of the pull request
        url: String,
    },
    /// List linked branches and pull requests for an issue
    #[command(after_help = "EXAMPLES:\n  \
    lin issue links ENG-123")]
    Links {
        /// Issue identifier (e.g., "ENG-123") or UUID
        identifier: String,
    },
    /// List all relations for an issue (parent, children, blocks, blocked by, etc.)
    #[command(after_help = "EXAMPLES:\n  \
    lin issue relations ENG-123")]
    Relations {
        /// Issue identifier (e.g., "ENG-123") or UUID
        identifier: String,
    },
    /// Add a relation between two issues
    #[command(after_help = "EXAMPLES:\n  \
    lin issue add-relation ENG-123 ENG-456 --type blocks\n  \
    lin issue add-relation ENG-123 ENG-456 --type parent\n  \
    lin issue add-relation ENG-123 ENG-456 --type related")]
    AddRelation {
        /// Source issue identifier (e.g., "ENG-123") or UUID
        issue: String,
        /// Target issue identifier (e.g., "ENG-456") or UUID
        related_issue: String,
        /// Relation type: parent, sub, blocks, blocked_by, related, duplicate
        #[arg(long = "type")]
        relation_type: String,
    },
    /// Remove a relation by its ID
    #[command(after_help = "EXAMPLES:\n  \
    lin issue remove-relation <relation-id>")]
    RemoveRelation {
        /// The relation ID to remove
        relation_id: String,
    },
}

/// Comment-related subcommands.
#[derive(Subcommand, Debug)]
enum CommentCommands {
    /// List comments on an issue
    #[command(after_help = "EXAMPLES:\n  \
    lin comment list ENG-123")]
    List {
        /// Issue identifier (e.g., "ENG-123") or UUID
        issue: String,
    },
    /// Add a comment to an issue
    #[command(after_help = "EXAMPLES:\n  \
    lin comment add ENG-123 --body \"This is my comment\"")]
    Add {
        /// Issue identifier (e.g., "ENG-123") or UUID
        issue: String,
        /// The comment body/content
        #[arg(long)]
        body: String,
    },
}

/// Attachment-related subcommands.
#[derive(Subcommand, Debug)]
enum AttachmentCommands {
    /// List attachments on an issue
    #[command(after_help = "EXAMPLES:\n  \
    lin attachment list --issue ENG-123")]
    List {
        /// Issue identifier (e.g., "ENG-123") or UUID
        #[arg(long)]
        issue: String,
    },
    /// Upload a file as an attachment to an issue
    #[command(after_help = "EXAMPLES:\n  \
    lin attachment upload --issue ENG-123 /path/to/file.png")]
    Upload {
        /// Issue identifier (e.g., "ENG-123") or UUID
        #[arg(long)]
        issue: String,
        /// Path to the file to upload
        file_path: String,
    },
    /// Get details of a specific attachment (including download URL)
    #[command(after_help = "EXAMPLES:\n  \
    lin attachment get <attachment-id>")]
    Get {
        /// Attachment ID
        id: String,
    },
}

/// Team-related subcommands.
#[derive(Subcommand, Debug)]
enum TeamCommands {
    /// List all teams
    #[command(after_help = "EXAMPLES:\n  \
    lin team list")]
    List,
    /// Get details of a specific team
    #[command(after_help = "EXAMPLES:\n  \
    lin team get ENG")]
    Get {
        /// Team identifier or key
        identifier: String,
    },
}

/// Workflow state-related subcommands.
#[derive(Subcommand, Debug)]
enum WorkflowCommands {
    /// List workflow states for a team
    #[command(after_help = "EXAMPLES:\n  \
    lin workflow list --team <team-id>\n  \
    lin workflow list --team ENG")]
    List {
        /// Team ID (UUID) or team key (e.g., "ENG")
        #[arg(long)]
        team: String,
    },
}

/// Project-related subcommands.
#[derive(Subcommand, Debug)]
enum ProjectCommands {
    /// List all projects
    #[command(after_help = "EXAMPLES:\n  \
    lin project list\n  \
    lin project list --team <team-id>")]
    List {
        /// Filter by team ID (optional)
        #[arg(long)]
        team: Option<String>,
    },
    /// Get details of a specific project
    #[command(after_help = "EXAMPLES:\n  \
    lin project get <project-id>")]
    Get {
        /// Project ID
        id: String,
    },
}

/// Cycle (sprint) related subcommands.
#[derive(Subcommand, Debug)]
enum CycleCommands {
    /// List cycles for a team
    #[command(after_help = "EXAMPLES:\n  \
    lin cycle list --team <team-id>\n  \
    lin cycle list --team ENG")]
    List {
        /// Team ID (UUID) or team key (e.g., "ENG")
        #[arg(long)]
        team: String,
    },
    /// Get details of a specific cycle including its issues
    #[command(after_help = "EXAMPLES:\n  \
    lin cycle get <cycle-id>")]
    Get {
        /// Cycle ID
        id: String,
    },
}

/// Label-related subcommands.
#[derive(Subcommand, Debug)]
enum LabelCommands {
    /// List all labels in the workspace
    #[command(after_help = "EXAMPLES:\n  \
    lin label list\n  \
    lin label list --team <team-id>")]
    List {
        /// Filter by team ID to show only team-specific labels (optional)
        #[arg(long)]
        team: Option<String>,
    },
    /// Get details of a specific label
    #[command(after_help = "EXAMPLES:\n  \
    lin label get <label-id>")]
    Get {
        /// Label ID
        id: String,
    },
}

/// Document-related subcommands.
#[derive(Subcommand, Debug)]
enum DocumentCommands {
    /// List all documents
    #[command(after_help = "EXAMPLES:\n  \
    lin document list\n  \
    lin document list --project <project-id>")]
    List {
        /// Filter by project ID (optional)
        #[arg(long)]
        project: Option<String>,
    },
    /// Get details of a specific document (including content)
    #[command(after_help = "EXAMPLES:\n  \
    lin document get <document-id>")]
    Get {
        /// Document ID
        id: String,
    },
    /// Create a new document
    #[command(after_help = "EXAMPLES:\n  \
    lin document create --title \"My Doc\" --content \"# Hello\"\n  \
    lin document create --title \"Project Doc\" --content \"Content\" --project <project-id>")]
    Create {
        /// Document title
        #[arg(long)]
        title: String,
        /// Document content (markdown)
        #[arg(long)]
        content: String,
        /// Project ID to associate the document with (optional)
        #[arg(long)]
        project: Option<String>,
    },
}

/// User-related subcommands.
#[derive(Subcommand, Debug)]
enum UserCommands {
    /// Get information about the authenticated user
    #[command(after_help = "EXAMPLES:\n  \
    lin user me")]
    Me,
    /// List users in the organization
    #[command(after_help = "EXAMPLES:\n  \
    lin user list")]
    List,
}

/// Organization-related subcommands.
#[derive(Subcommand, Debug)]
enum OrgCommands {
    /// Add an organization (reads API token from stdin)
    #[command(after_help = "EXAMPLES:\n  \
    echo \"lin_api_...\" | lin org add my-org")]
    Add {
        /// Name to identify this organization
        name: String,
    },
    /// Remove an organization
    #[command(after_help = "EXAMPLES:\n  \
    lin org remove my-org")]
    Remove {
        /// Name of the organization to remove
        name: String,
    },
    /// List all configured organizations
    #[command(after_help = "EXAMPLES:\n  \
    lin org list")]
    List,
    /// Set the default organization
    #[command(after_help = "EXAMPLES:\n  \
    lin org set-default my-org")]
    SetDefault {
        /// Name of the organization to set as default
        name: String,
    },
    /// Get information about the current Linear organization (requires API token)
    #[command(after_help = "EXAMPLES:\n  \
    lin org info")]
    Info,
}

/// Placeholder response for unimplemented commands.
#[derive(Serialize)]
struct PlaceholderResponse {
    message: &'static str,
    command: String,
}

fn main() {
    let cli = Cli::parse();
    let format = OutputFormat::from_json_flag(cli.json);

    // Initialize color support (respects NO_COLOR env and TTY detection)
    init_colors();

    if let Err(err) = run(cli, format) {
        output_error_with_format(&err, format);
    }
}

fn run(cli: Cli, format: OutputFormat) -> lin::Result<()> {
    match &cli.command {
        // Org commands don't all require an API token
        Commands::Org { command } => handle_org_command(command, &cli, format),
        // All other commands require an API token
        _ => {
            let config = Config::load()?;
            let token = require_api_token(cli.api_token.as_deref(), &config, cli.org.as_deref())?;

            match cli.command {
                Commands::Issue { command } => handle_issue_command(command, &token, format),
                Commands::Comment { command } => handle_comment_command(command, &token, format),
                Commands::Attachment { command } => {
                    handle_attachment_command(command, &token, format)
                }
                Commands::Team { command } => handle_team_command(command, &token, format),
                Commands::User { command } => handle_user_command(command, &token, format),
                Commands::Workflow { command } => handle_workflow_command(command, &token, format),
                Commands::Project { command } => handle_project_command(command, &token, format),
                Commands::Cycle { command } => handle_cycle_command(command, &token, format),
                Commands::Label { command } => handle_label_command(command, &token, format),
                Commands::Document { command } => handle_document_command(command, &token, format),
                Commands::Search {
                    query,
                    team,
                    assignee,
                    state,
                    limit,
                } => handle_search_command(&token, &query, team, assignee, state, limit, format),
                Commands::Org { .. } => unreachable!(),
            }
        }
    }
}

fn handle_issue_command(
    command: IssueCommands,
    token: &str,
    format: OutputFormat,
) -> lin::Result<()> {
    let client = GraphQLClient::new(token);

    match command {
        IssueCommands::List {
            team,
            assignee,
            state,
            project,
            cycle,
            label,
            limit,
            created_after,
            created_before,
            updated_after,
            updated_before,
            sort,
            order,
        } => {
            // If assignee is "me", we need to fetch the viewer ID first
            let viewer_id = if assignee.as_deref() == Some("me") {
                let response: lin::models::ViewerResponse =
                    client.query(lin::api::queries::VIEWER_QUERY, serde_json::json!({}))?;
                Some(response.viewer.id)
            } else {
                None
            };

            // Parse sort field if provided
            let sort_by = if let Some(sort_str) = &sort {
                let field = issue::IssueSortField::parse(sort_str).ok_or_else(|| {
                    lin::error::LinError::config(format!(
                        "Invalid sort field '{}'. Valid fields: priority, created, updated, title",
                        sort_str
                    ))
                })?;
                Some(field)
            } else {
                None
            };

            // Parse sort order if provided
            let sort_order = if let Some(order_str) = &order {
                let ord = issue::SortOrder::parse(order_str).ok_or_else(|| {
                    lin::error::LinError::config(format!(
                        "Invalid sort order '{}'. Valid orders: asc, desc",
                        order_str
                    ))
                })?;
                Some(ord)
            } else {
                None
            };

            let options = issue::IssueListOptions {
                team,
                assignee,
                state,
                project,
                cycle,
                label,
                limit: Some(limit as i32),
                created_after,
                created_before,
                updated_after,
                updated_before,
                sort_by,
                sort_order,
            };
            issue::list_issues(&client, viewer_id.as_deref(), options, format)
        }
        IssueCommands::Get {
            identifier,
            with_comments,
        } => issue::get_issue_with_comments(&client, &identifier, with_comments, format),
        IssueCommands::Create {
            title,
            team,
            description,
            assignee,
            state,
            priority,
            labels,
        } => {
            let options = issue::IssueCreateOptions {
                title,
                team_id: team,
                description,
                assignee_id: assignee,
                state_id: state,
                priority: priority.map(|p| p as i32),
                label_ids: labels,
            };
            issue::create_issue(&client, options, format)
        }
        IssueCommands::Update {
            identifier,
            title,
            description,
            assignee,
            state,
            priority,
            labels,
        } => {
            let options = issue::IssueUpdateOptions {
                title,
                description,
                assignee_id: assignee,
                state_id: state,
                priority: priority.map(|p| p as i32),
                label_ids: labels,
            };
            issue::update_issue(&client, &identifier, options, format)
        }
        IssueCommands::Delete { identifier } => issue::delete_issue(&client, &identifier, format),
        IssueCommands::Archive { identifier } => issue::archive_issue(&client, &identifier, format),
        IssueCommands::Unarchive { identifier } => {
            issue::unarchive_issue(&client, &identifier, format)
        }
        IssueCommands::LinkBranch {
            identifier,
            branch,
            repo,
        } => git::link_branch(&client, &identifier, &branch, repo.as_deref(), format),
        IssueCommands::LinkPr { identifier, url } => {
            git::link_pr(&client, &identifier, &url, format)
        }
        IssueCommands::Links { identifier } => git::list_links(&client, &identifier, format),
        IssueCommands::Relations { identifier } => {
            relation::list_relations(&client, &identifier, format)
        }
        IssueCommands::AddRelation {
            issue,
            related_issue,
            relation_type,
        } => {
            let rel_type = relation::RelationType::parse(&relation_type).ok_or_else(|| {
                lin::error::LinError::config(format!(
                    "Invalid relation type '{}'. Valid types: parent, sub, blocks, blocked_by, related, duplicate",
                    relation_type
                ))
            })?;
            relation::add_relation(&client, &issue, &related_issue, rel_type, format)
        }
        IssueCommands::RemoveRelation { relation_id } => {
            relation::remove_relation(&client, &relation_id, format)
        }
    }
}

fn handle_comment_command(
    command: CommentCommands,
    token: &str,
    format: OutputFormat,
) -> lin::Result<()> {
    let client = GraphQLClient::new(token);

    match command {
        CommentCommands::List { issue } => comment::list_comments(&client, &issue, format),
        CommentCommands::Add { issue, body } => {
            comment::create_comment(&client, &issue, &body, format)
        }
    }
}

fn handle_attachment_command(
    command: AttachmentCommands,
    token: &str,
    format: OutputFormat,
) -> lin::Result<()> {
    let client = GraphQLClient::new(token);

    match command {
        AttachmentCommands::List { issue } => attachment::list_attachments(&client, &issue, format),
        AttachmentCommands::Upload { issue, file_path } => {
            attachment::upload_attachment(&client, &issue, &file_path, format)
        }
        AttachmentCommands::Get { id } => attachment::get_attachment(&client, &id, format),
    }
}

fn handle_team_command(
    command: TeamCommands,
    token: &str,
    format: OutputFormat,
) -> lin::Result<()> {
    let client = GraphQLClient::new(token);

    match command {
        TeamCommands::List => team::list_teams(&client, format),
        TeamCommands::Get { identifier } => team::get_team(&client, &identifier, format),
    }
}

fn handle_user_command(
    command: UserCommands,
    token: &str,
    format: OutputFormat,
) -> lin::Result<()> {
    let client = GraphQLClient::new(token);

    match command {
        UserCommands::Me => user::me(&client, format),
        UserCommands::List => user::list_users(&client, format),
    }
}

fn handle_workflow_command(
    command: WorkflowCommands,
    token: &str,
    format: OutputFormat,
) -> lin::Result<()> {
    let client = GraphQLClient::new(token);

    match command {
        WorkflowCommands::List { team } => workflow::list_workflow_states(&client, &team, format),
    }
}

fn handle_project_command(
    command: ProjectCommands,
    token: &str,
    format: OutputFormat,
) -> lin::Result<()> {
    let client = GraphQLClient::new(token);

    match command {
        ProjectCommands::List { team } => {
            let options = project::ProjectListOptions { team_id: team };
            project::list_projects(&client, options, format)
        }
        ProjectCommands::Get { id } => project::get_project(&client, &id, format),
    }
}

fn handle_cycle_command(
    command: CycleCommands,
    token: &str,
    format: OutputFormat,
) -> lin::Result<()> {
    let client = GraphQLClient::new(token);

    match command {
        CycleCommands::List { team } => cycle::list_cycles(&client, &team, format),
        CycleCommands::Get { id } => cycle::get_cycle(&client, &id, format),
    }
}

fn handle_label_command(
    command: LabelCommands,
    token: &str,
    format: OutputFormat,
) -> lin::Result<()> {
    let client = GraphQLClient::new(token);

    match command {
        LabelCommands::List { team } => {
            let options = label::LabelListOptions { team_id: team };
            label::list_labels(&client, options, format)
        }
        LabelCommands::Get { id } => label::get_label(&client, &id, format),
    }
}

fn handle_document_command(
    command: DocumentCommands,
    token: &str,
    format: OutputFormat,
) -> lin::Result<()> {
    let client = GraphQLClient::new(token);

    match command {
        DocumentCommands::List { project } => {
            let options = document::DocumentListOptions {
                project_id: project,
            };
            document::list_documents(&client, options, format)
        }
        DocumentCommands::Get { id } => document::get_document(&client, &id, format),
        DocumentCommands::Create {
            title,
            content,
            project,
        } => {
            let options = document::DocumentCreateOptions {
                title,
                content,
                project_id: project,
            };
            document::create_document(&client, options, format)
        }
    }
}

fn handle_org_command(command: &OrgCommands, cli: &Cli, format: OutputFormat) -> lin::Result<()> {
    match command {
        OrgCommands::Add { name } => org::add_org(name, format),
        OrgCommands::Remove { name } => org::remove_org(name, format),
        OrgCommands::List => org::list_orgs(format),
        OrgCommands::SetDefault { name } => org::set_default_org(name, format),
        OrgCommands::Info => {
            // Info requires API token
            let config = Config::load()?;
            let _token = require_api_token(cli.api_token.as_deref(), &config, cli.org.as_deref())?;

            let response = PlaceholderResponse {
                message: "Command not yet implemented",
                command: "org info".into(),
            };
            output_success(&response);
            Ok(())
        }
    }
}

fn handle_search_command(
    token: &str,
    query: &str,
    team: Option<String>,
    assignee: Option<String>,
    state: Option<String>,
    limit: u32,
    format: OutputFormat,
) -> lin::Result<()> {
    let client = GraphQLClient::new(token);

    // If assignee is "me", we need to fetch the viewer ID first
    let viewer_id = if assignee.as_deref() == Some("me") {
        let response: lin::models::ViewerResponse =
            client.query(lin::api::queries::VIEWER_QUERY, serde_json::json!({}))?;
        Some(response.viewer.id)
    } else {
        None
    };

    let options = search::SearchOptions {
        team,
        assignee,
        state,
        limit: Some(limit as i32),
    };

    search::search_issues(&client, query, viewer_id.as_deref(), options, format)
}
