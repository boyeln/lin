//! lin - A command-line interface for Linear
//!
//! Entry point for the CLI application.

use clap::{Parser, Subcommand};
use lin::api::GraphQLClient;
use lin::auth::require_api_token;
use lin::commands::{issue, org, team, user, workflow};
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
}

/// Issue-related subcommands.
#[derive(Subcommand, Debug)]
enum IssueCommands {
    /// List issues
    #[command(after_help = "EXAMPLES:\n  \
    lin issue list --team ENG --assignee me\n  \
    lin issue list --state \"In Progress\" --limit 10")]
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
        /// Maximum number of issues to return
        #[arg(long, default_value = "50")]
        limit: u32,
    },
    /// Get details of a specific issue
    #[command(after_help = "EXAMPLES:\n  \
    lin issue get ENG-123")]
    Get {
        /// Issue identifier (e.g., "ENG-123")
        identifier: String,
    },
    /// Create a new issue
    #[command(after_help = "EXAMPLES:\n  \
    lin issue create --team <team-id> --title \"Fix bug\" --priority 2")]
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
    },
    /// Update an existing issue
    #[command(after_help = "EXAMPLES:\n  \
    lin issue update ENG-123 --title \"New title\"\n  \
    lin issue update ENG-123 --state <state-id> --priority 1")]
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
                Commands::Team { command } => handle_team_command(command, &token, format),
                Commands::User { command } => handle_user_command(command, &token, format),
                Commands::Workflow { command } => handle_workflow_command(command, &token, format),
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
            limit,
        } => {
            // If assignee is "me", we need to fetch the viewer ID first
            let viewer_id = if assignee.as_deref() == Some("me") {
                let response: lin::models::ViewerResponse =
                    client.query(lin::api::queries::VIEWER_QUERY, serde_json::json!({}))?;
                Some(response.viewer.id)
            } else {
                None
            };

            let options = issue::IssueListOptions {
                team,
                assignee,
                state,
                limit: Some(limit as i32),
            };
            issue::list_issues(&client, viewer_id.as_deref(), options, format)
        }
        IssueCommands::Get { identifier } => issue::get_issue(&client, &identifier, format),
        IssueCommands::Create {
            title,
            team,
            description,
            assignee,
            state,
            priority,
        } => {
            let options = issue::IssueCreateOptions {
                title,
                team_id: team,
                description,
                assignee_id: assignee,
                state_id: state,
                priority: priority.map(|p| p as i32),
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
        } => {
            let options = issue::IssueUpdateOptions {
                title,
                description,
                assignee_id: assignee,
                state_id: state,
                priority: priority.map(|p| p as i32),
            };
            issue::update_issue(&client, &identifier, options, format)
        }
        IssueCommands::Delete { identifier } => issue::delete_issue(&client, &identifier, format),
        IssueCommands::Archive { identifier } => issue::archive_issue(&client, &identifier, format),
        IssueCommands::Unarchive { identifier } => {
            issue::unarchive_issue(&client, &identifier, format)
        }
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
