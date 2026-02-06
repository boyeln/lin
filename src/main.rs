//! lin - A command-line interface for Linear
//!
//! Entry point for the CLI application.

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use lin::api::GraphQLClient;
use lin::auth;
use lin::commands::{
    attachment, comment, completions, cycle, git, issue, label, project, relation, resolvers,
    search, self_update, team, user, workflow,
};
use lin::config::Config;
use lin::error::LinError;
use lin::output::{OutputFormat, init_colors, output_error_with_format};
use std::env;

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
        command: Box<IssueCommands>,
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
    /// Authenticate and manage organizations
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
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
    /// Search for issues
    #[command(after_help = "EXAMPLES:\n  \
    lin search \"authentication bug\"\n  \
    lin search \"fix login\" --team ENG --limit 10\n  \
    lin search \"urgent\" --assignee me --state \"In Progress\"")]
    Search {
        /// The search query string
        query: String,
        /// Filter by team key or UUID (e.g., "ENG")
        #[arg(long)]
        team: Option<String>,
        /// Filter by assignee (user ID or "me" for current user)
        #[arg(long)]
        assignee: Option<String>,
        /// Filter by state name or UUID (e.g., "In Progress", "Done")
        #[arg(long)]
        state: Option<String>,
        /// Maximum number of results to return
        #[arg(long, default_value = "50")]
        limit: u32,
    },
    /// Generate shell completion scripts
    #[command(after_help = "EXAMPLES:\n  \
    lin completions bash > ~/.local/share/bash-completion/completions/lin\n  \
    lin completions zsh > ~/.zfunc/_lin\n  \
    lin completions fish > ~/.config/fish/completions/lin.fish")]
    Completions {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Update lin to the latest version
    #[command(after_help = "EXAMPLES:\n  \
    lin update\n  \
    lin update --check")]
    Update {
        /// Only check if an update is available, don't install
        #[arg(long)]
        check: bool,
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
    lin issue list --priority urgent\n  \
    lin issue list --priority 2\n  \
    lin issue list --created-after 2024-01-01\n  \
    lin issue list --updated-before 2024-12-31\n  \
    lin issue list --sort priority --order asc\n  \
    lin issue list --sort updated --order desc\n  \
    lin issue list --team ENG --assignee me --priority high")]
    List {
        /// Filter by team key or UUID (e.g., "ENG")
        #[arg(long)]
        team: Option<String>,
        /// Filter by assignee
        #[arg(long)]
        assignee: Option<String>,
        /// Filter by state name or UUID (e.g., "In Progress", "Done")
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
        /// Filter by priority (0-4 or: none, urgent, high, normal, low)
        #[arg(long)]
        priority: Option<String>,
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
    lin issue create --team <team-id> --title \"New feature\" --labels <label-id1> --labels <label-id2>\n  \
    lin issue create --team <team-id> --title \"Project task\" --project <project-id> --estimate M")]
    Create {
        /// Issue title
        #[arg(long)]
        title: String,
        /// Team key or UUID (e.g., "ENG"). Uses current team if not specified.
        #[arg(long)]
        team: Option<String>,
        /// Issue description (optional)
        #[arg(long)]
        description: Option<String>,
        /// Assignee user ID (optional)
        #[arg(long)]
        assignee: Option<String>,
        /// State name or UUID (e.g., "Todo")
        #[arg(long)]
        state: Option<String>,
        /// Priority (0-4: 0=none, 1=urgent, 2=high, 3=normal, 4=low)
        #[arg(long)]
        priority: Option<u8>,
        /// Estimate (numeric value or team-configured name like "XS", "S", "M", "L", "XL")
        #[arg(long)]
        estimate: Option<String>,
        /// Label IDs to add to the issue (can be specified multiple times)
        #[arg(long)]
        labels: Option<Vec<String>>,
        /// Project ID to assign the issue to (optional)
        #[arg(long)]
        project: Option<String>,
    },
    /// Update an existing issue
    #[command(after_help = "EXAMPLES:\n  \
    lin issue update ENG-123 --title \"New title\"\n  \
    lin issue update ENG-123 --state <state-id> --priority 1\n  \
    lin issue update ENG-123 --labels <label-id1> --labels <label-id2>\n  \
    lin issue update ENG-123 --project <project-id> --estimate L")]
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
        /// State name or UUID (e.g., "In Progress")
        #[arg(long)]
        state: Option<String>,
        /// New priority (0-4: 0=none, 1=urgent, 2=high, 3=normal, 4=low)
        #[arg(long)]
        priority: Option<u8>,
        /// Estimate (numeric value or team-configured name like "XS", "S", "M", "L", "XL")
        #[arg(long)]
        estimate: Option<String>,
        /// Label IDs to set on the issue (replaces existing labels, can be specified multiple times)
        #[arg(long)]
        labels: Option<Vec<String>>,
        /// Project ID to assign the issue to (optional)
        #[arg(long)]
        project: Option<String>,
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
    /// Switch current team or show current team
    #[command(after_help = "EXAMPLES:\n  \
    lin team switch ENG\n  \
    lin team switch")]
    Switch {
        /// Team key to switch to (if not provided, shows current team)
        team: Option<String>,
    },
}

/// Workflow state-related subcommands.
#[derive(Subcommand, Debug)]
enum WorkflowCommands {
    /// List workflow states for a team
    #[command(after_help = "EXAMPLES:\n  \
    lin workflow list --team <team-id>\n  \
    lin workflow list --team ENG\n  \
    lin workflow list")]
    List {
        /// Team key or UUID (e.g., "ENG"). Uses current team if not specified.
        #[arg(long)]
        team: Option<String>,
    },
}

/// Project-related subcommands.
#[derive(Subcommand, Debug)]
enum ProjectCommands {
    /// List all projects
    #[command(after_help = "EXAMPLES:\n  \
    lin project list\n  \
    lin project list --team ENG")]
    List {
        /// Filter by team key (optional), e.g. ENG
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
        /// Team key or UUID (e.g., "ENG")
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

/// Authentication-related subcommands.
#[derive(Subcommand, Debug)]
enum AuthCommands {
    /// Authenticate with a Linear organization
    #[command(after_help = "EXAMPLES:\n  \
    lin auth add my-org lin_api_xxxxx\n  \
    lin auth my-org lin_api_xxxxx")]
    Add {
        /// Organization name (e.g., "work", "personal")
        name: String,
        /// Linear API token
        token: String,
    },
    /// Switch to a different organization
    #[command(after_help = "EXAMPLES:\n  \
    lin auth switch work")]
    Switch {
        /// Organization name
        name: String,
    },
    /// List all authenticated organizations
    #[command(after_help = "EXAMPLES:\n  \
    lin auth list")]
    List,
    /// Remove an organization
    #[command(after_help = "EXAMPLES:\n  \
    lin auth remove work")]
    Remove {
        /// Organization name
        name: String,
    },
    /// Show status of current organization
    #[command(after_help = "EXAMPLES:\n  \
    lin auth status")]
    Status,
    /// Sync teams and workflow states
    #[command(after_help = "EXAMPLES:\n  \
    lin auth sync")]
    Sync,
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
        // Auth commands don't require being authenticated yet
        Commands::Auth { command } => handle_auth_command(command, format),
        // Completions command doesn't require an API token
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            completions::generate_completions(*shell, &mut cmd);
            Ok(())
        }
        // Update command doesn't require an API token
        Commands::Update { check } => {
            if *check {
                self_update::check_update(format)
            } else {
                self_update::update(format)
            }
        }
        // All other commands require an API token
        _ => {
            let (client, use_cache) = get_client_and_mode()?;

            match cli.command {
                Commands::Issue { command } => {
                    handle_issue_command(*command, client, use_cache, format)
                }
                Commands::Comment { command } => handle_comment_command(command, client, format),
                Commands::Attachment { command } => {
                    handle_attachment_command(command, client, format)
                }
                Commands::Team { command } => handle_team_command(command, client, format),
                Commands::User { command } => handle_user_command(command, client, format),
                Commands::Workflow { command } => {
                    handle_workflow_command(command, client, use_cache, format)
                }
                Commands::Project { command } => handle_project_command(command, client, format),
                Commands::Cycle { command } => handle_cycle_command(command, client, format),
                Commands::Label { command } => handle_label_command(command, client, format),
                Commands::Search {
                    query,
                    team,
                    assignee,
                    state,
                    limit,
                } => handle_search_command(client, &query, team, assignee, state, limit, format),
                Commands::Auth { .. } | Commands::Completions { .. } | Commands::Update { .. } => {
                    unreachable!()
                }
            }
        }
    }
}

/// Get a GraphQL client and determine whether to use cache.
///
/// Returns (client, use_cache) where:
/// - client: GraphQL client initialized with the API token
/// - use_cache: true if using config-based auth (cache available), false if using env var
///
/// # Errors
///
/// Returns an error if no authentication is available.
fn get_client_and_mode() -> lin::Result<(GraphQLClient, bool)> {
    // Check if LINEAR_API_TOKEN env var is set
    if let Ok(token) = env::var(auth::LINEAR_API_TOKEN_ENV) {
        if !token.is_empty() {
            return Ok((GraphQLClient::new(&token), false));
        }
    }

    // Try to load config and get active org token
    let config = Config::load()?;
    match config.get_token(None) {
        Ok(token) => Ok((GraphQLClient::new(&token), true)),
        Err(_) => Err(LinError::config(
            "Not authenticated. Run: lin auth add <name> <token>".to_string(),
        )),
    }
}

fn handle_auth_command(command: &AuthCommands, format: OutputFormat) -> lin::Result<()> {
    match command {
        AuthCommands::Add { name, token } => {
            lin::commands::auth::auth_add(name.clone(), token.clone(), format)
        }
        AuthCommands::Switch { name } => lin::commands::auth::auth_switch(name.clone(), format),
        AuthCommands::List => lin::commands::auth::auth_list(format),
        AuthCommands::Remove { name } => lin::commands::auth::auth_remove(name.clone(), format),
        AuthCommands::Status => lin::commands::auth::auth_status(format),
        AuthCommands::Sync => lin::commands::auth::auth_sync(format),
    }
}

fn handle_issue_command(
    command: IssueCommands,
    client: GraphQLClient,
    use_cache: bool,
    format: OutputFormat,
) -> lin::Result<()> {
    match command {
        IssueCommands::List {
            team,
            assignee,
            state,
            project,
            cycle,
            label,
            priority,
            limit,
            created_after,
            created_before,
            updated_after,
            updated_before,
            sort,
            order,
        } => {
            // Resolve team if provided or use current team
            let resolved_team = if team.is_some() {
                team.clone()
            } else {
                // Try to get current team, but it's optional for list
                let config = Config::load()?;
                config.get_current_team()
            };
            // If assignee is "me", we need to fetch the viewer ID first
            let viewer_id = if assignee.as_deref() == Some("me") {
                let response: lin::models::ViewerResponse = client.query(
                    lin::api::queries::organization::VIEWER_QUERY,
                    serde_json::json!({}),
                )?;
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

            // Parse priority filter if provided
            let priority_filter = if let Some(priority_str) = &priority {
                let prio = issue::PriorityFilter::parse(priority_str).ok_or_else(|| {
                    lin::error::LinError::config(format!(
                        "Invalid priority '{}'. Valid values: 0-4 or none, urgent, high, normal, low",
                        priority_str
                    ))
                })?;
                Some(prio)
            } else {
                None
            };

            let options = issue::IssueListOptions {
                team: resolved_team,
                assignee,
                state,
                project,
                cycle,
                label,
                priority: priority_filter,
                limit: Some(limit as i32),
                created_after,
                created_before,
                updated_after,
                updated_before,
                sort_by,
                sort_order,
            };
            issue::list::list_issues(&client, viewer_id.as_deref(), options, format)
        }
        IssueCommands::Get {
            identifier,
            with_comments,
        } => issue::get::get_issue_with_comments(&client, &identifier, with_comments, format),
        IssueCommands::Create {
            title,
            team,
            description,
            assignee,
            state,
            priority,
            estimate,
            labels,
            project,
        } => {
            // Resolve team key to team ID (using current team if not specified)
            let team_id = resolvers::resolve_team_or_current(&client, team.as_deref(), use_cache)?;

            // Get the actual team key for state/estimate resolution
            let team_key = if let Some(ref t) = team {
                t.clone()
            } else {
                // We need the team key for state/estimate resolution
                resolvers::get_team_key(&client, &team_id)?
            };

            // Resolve state name to state ID if provided
            let state_id = if let Some(state_name) = state {
                Some(resolvers::resolve_state_id(
                    &client,
                    &team_key,
                    &state_name,
                    use_cache,
                )?)
            } else {
                None
            };

            // Resolve estimate name to numeric value if provided
            let estimate_value = if let Some(est) = estimate {
                Some(resolvers::resolve_estimate_value(
                    &est,
                    Some(&team_key),
                    use_cache,
                )?)
            } else {
                None
            };

            let options = issue::IssueCreateOptions {
                title,
                team_id,
                description,
                assignee_id: assignee,
                state_id,
                priority: priority.map(|p| p as i32),
                estimate: estimate_value,
                label_ids: labels,
                project_id: project,
            };
            issue::create::create_issue(&client, options, format)
        }
        IssueCommands::Update {
            identifier,
            title,
            description,
            assignee,
            state,
            priority,
            estimate,
            labels,
            project,
        } => {
            // We may need team context for state or estimate resolution
            let team_key_opt = if state.is_some() || estimate.is_some() {
                // Resolve identifier to UUID if needed to get team context
                let issue_id = if issue::is_uuid(&identifier) {
                    identifier.clone()
                } else {
                    // Parse identifier and look up issue
                    let (team_key, number) = issue::parse_identifier(&identifier)?;
                    let lookup_variables = serde_json::json!({
                        "filter": {
                            "team": { "key": { "eq": team_key } },
                            "number": { "eq": number }
                        }
                    });
                    let lookup_response: lin::models::IssuesResponse = client.query(
                        lin::api::queries::issue::ISSUE_BY_IDENTIFIER_QUERY,
                        lookup_variables,
                    )?;
                    if lookup_response.issues.nodes.is_empty() {
                        return Err(LinError::api(format!("Issue '{}' not found", identifier)));
                    }
                    lookup_response.issues.nodes[0].id.clone()
                };

                // Get team context
                let team_id = resolvers::get_issue_team_id(&client, &issue_id)?;
                Some(resolvers::get_team_key(&client, &team_id)?)
            } else {
                None
            };

            // Resolve state name to state ID if provided and not already a UUID
            let state_id = if let Some(state_name) = state {
                if issue::is_uuid(&state_name) {
                    Some(state_name)
                } else {
                    let team_key = team_key_opt.as_ref().unwrap();
                    Some(resolvers::resolve_state_id(
                        &client,
                        team_key,
                        &state_name,
                        use_cache,
                    )?)
                }
            } else {
                None
            };

            // Resolve estimate name to numeric value if provided
            let estimate_value = if let Some(est) = estimate {
                let team_key = team_key_opt.as_deref();
                Some(resolvers::resolve_estimate_value(
                    &est, team_key, use_cache,
                )?)
            } else {
                None
            };

            let options = issue::IssueUpdateOptions {
                title,
                description,
                assignee_id: assignee,
                state_id,
                priority: priority.map(|p| p as i32),
                estimate: estimate_value,
                label_ids: labels,
                project_id: project,
            };
            issue::update::update_issue(&client, &identifier, options, format)
        }
        IssueCommands::Delete { identifier } => {
            issue::delete::delete_issue(&client, &identifier, format)
        }
        IssueCommands::Archive { identifier } => {
            issue::delete::archive_issue(&client, &identifier, format)
        }
        IssueCommands::Unarchive { identifier } => {
            issue::delete::unarchive_issue(&client, &identifier, format)
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
    client: GraphQLClient,
    format: OutputFormat,
) -> lin::Result<()> {
    match command {
        CommentCommands::List { issue } => comment::list_comments(&client, &issue, format),
        CommentCommands::Add { issue, body } => {
            comment::create_comment(&client, &issue, &body, format)
        }
    }
}

fn handle_attachment_command(
    command: AttachmentCommands,
    client: GraphQLClient,
    format: OutputFormat,
) -> lin::Result<()> {
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
    client: GraphQLClient,
    format: OutputFormat,
) -> lin::Result<()> {
    match command {
        TeamCommands::List => team::list_teams(&client, format),
        TeamCommands::Get { identifier } => team::get_team(&client, &identifier, format),
        TeamCommands::Switch { team } => team::switch_team(team, format),
    }
}

fn handle_user_command(
    command: UserCommands,
    client: GraphQLClient,
    format: OutputFormat,
) -> lin::Result<()> {
    match command {
        UserCommands::Me => user::me(&client, format),
        UserCommands::List => user::list_users(&client, format),
    }
}

fn handle_workflow_command(
    command: WorkflowCommands,
    client: GraphQLClient,
    use_cache: bool,
    format: OutputFormat,
) -> lin::Result<()> {
    match command {
        WorkflowCommands::List { team } => {
            // Resolve team key to team ID (using current team if not specified)
            let team_id = resolvers::resolve_team_or_current(&client, team.as_deref(), use_cache)?;
            workflow::list_workflow_states(&client, &team_id, format)
        }
    }
}

fn handle_project_command(
    command: ProjectCommands,
    client: GraphQLClient,
    format: OutputFormat,
) -> lin::Result<()> {
    match command {
        ProjectCommands::List { team } => {
            let options = project::ProjectListOptions { team_key: team };
            project::list_projects(&client, options, format)
        }
        ProjectCommands::Get { id } => project::get_project(&client, &id, format),
    }
}

fn handle_cycle_command(
    command: CycleCommands,
    client: GraphQLClient,
    format: OutputFormat,
) -> lin::Result<()> {
    match command {
        CycleCommands::List { team } => cycle::list_cycles(&client, &team, format),
        CycleCommands::Get { id } => cycle::get_cycle(&client, &id, format),
    }
}

fn handle_label_command(
    command: LabelCommands,
    client: GraphQLClient,
    format: OutputFormat,
) -> lin::Result<()> {
    match command {
        LabelCommands::List { team } => {
            let options = label::LabelListOptions { team_id: team };
            label::list_labels(&client, options, format)
        }
        LabelCommands::Get { id } => label::get_label(&client, &id, format),
    }
}

fn handle_search_command(
    client: GraphQLClient,
    query: &str,
    team: Option<String>,
    assignee: Option<String>,
    state: Option<String>,
    limit: u32,
    format: OutputFormat,
) -> lin::Result<()> {
    // If assignee is "me", we need to fetch the viewer ID first
    let viewer_id = if assignee.as_deref() == Some("me") {
        let response: lin::models::ViewerResponse = client.query(
            lin::api::queries::organization::VIEWER_QUERY,
            serde_json::json!({}),
        )?;
        Some(response.viewer.id)
    } else {
        None
    };

    // Resolve team if provided or use current team
    let resolved_team = if team.is_some() {
        team.clone()
    } else {
        // Try to get current team, but it's optional for search
        let config = Config::load()?;
        config.get_current_team()
    };

    let options = search::SearchOptions {
        team: resolved_team,
        assignee,
        state,
        limit: Some(limit as i32),
    };

    search::search_issues(&client, query, viewer_id.as_deref(), options, format)
}
