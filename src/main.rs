//! lin - A command-line interface for Linear
//!
//! Entry point for the CLI application.

use clap::{Parser, Subcommand, ValueEnum};
use lin::error::LinError;
use lin::output::{output_error, output_success};
use serde::Serialize;

/// lin - A command-line interface for Linear
#[derive(Parser, Debug)]
#[command(name = "lin")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Linear API token (can also be set via LINEAR_API_TOKEN env var)
    #[arg(long, global = true, env = "LINEAR_API_TOKEN")]
    api_token: Option<String>,

    /// Output format
    #[arg(long, global = true, default_value = "json")]
    output: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

/// Output format for CLI responses.
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum OutputFormat {
    /// JSON output (default, for scriptability)
    #[default]
    Json,
    /// Pretty-printed output with colors
    Pretty,
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
    /// Manage organization settings
    Org {
        #[command(subcommand)]
        command: OrgCommands,
    },
}

/// Issue-related subcommands.
#[derive(Subcommand, Debug)]
enum IssueCommands {
    /// List issues
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
    Get {
        /// Issue identifier (e.g., "ENG-123")
        identifier: String,
    },
    /// Create a new issue
    Create {
        /// Issue title
        #[arg(long)]
        title: String,
        /// Team identifier
        #[arg(long)]
        team: String,
        /// Issue description (optional)
        #[arg(long)]
        description: Option<String>,
        /// Priority (0-4, where 0 is no priority)
        #[arg(long)]
        priority: Option<u8>,
    },
    /// Update an existing issue
    Update {
        /// Issue identifier (e.g., "ENG-123")
        identifier: String,
        /// New title
        #[arg(long)]
        title: Option<String>,
        /// New description
        #[arg(long)]
        description: Option<String>,
        /// New state name
        #[arg(long)]
        state: Option<String>,
        /// New priority (0-4)
        #[arg(long)]
        priority: Option<u8>,
    },
}

/// Team-related subcommands.
#[derive(Subcommand, Debug)]
enum TeamCommands {
    /// List all teams
    List,
    /// Get details of a specific team
    Get {
        /// Team identifier or key
        identifier: String,
    },
}

/// User-related subcommands.
#[derive(Subcommand, Debug)]
enum UserCommands {
    /// Get information about the authenticated user
    Me,
    /// List users in the organization
    List,
}

/// Organization-related subcommands.
#[derive(Subcommand, Debug)]
enum OrgCommands {
    /// Get information about the current organization
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

    if let Err(err) = run(cli) {
        output_error(&err);
    }
}

fn run(cli: Cli) -> lin::Result<()> {
    // Validate that we have an API token for commands that need it
    let _token = cli.api_token.as_ref().ok_or_else(|| {
        LinError::config(
            "API token is required. Set LINEAR_API_TOKEN environment variable or use --api-token flag."
        )
    })?;

    match cli.command {
        Commands::Issue { command } => handle_issue_command(command),
        Commands::Team { command } => handle_team_command(command),
        Commands::User { command } => handle_user_command(command),
        Commands::Org { command } => handle_org_command(command),
    }
}

fn handle_issue_command(command: IssueCommands) -> lin::Result<()> {
    let response = match command {
        IssueCommands::List { .. } => PlaceholderResponse {
            message: "Command not yet implemented",
            command: "issue list".into(),
        },
        IssueCommands::Get { identifier } => PlaceholderResponse {
            message: "Command not yet implemented",
            command: format!("issue get {}", identifier),
        },
        IssueCommands::Create { .. } => PlaceholderResponse {
            message: "Command not yet implemented",
            command: "issue create".into(),
        },
        IssueCommands::Update { identifier, .. } => PlaceholderResponse {
            message: "Command not yet implemented",
            command: format!("issue update {}", identifier),
        },
    };
    output_success(&response);
    Ok(())
}

fn handle_team_command(command: TeamCommands) -> lin::Result<()> {
    let response = match command {
        TeamCommands::List => PlaceholderResponse {
            message: "Command not yet implemented",
            command: "team list".into(),
        },
        TeamCommands::Get { identifier } => PlaceholderResponse {
            message: "Command not yet implemented",
            command: format!("team get {}", identifier),
        },
    };
    output_success(&response);
    Ok(())
}

fn handle_user_command(command: UserCommands) -> lin::Result<()> {
    let response = match command {
        UserCommands::Me => PlaceholderResponse {
            message: "Command not yet implemented",
            command: "user me".into(),
        },
        UserCommands::List => PlaceholderResponse {
            message: "Command not yet implemented",
            command: "user list".into(),
        },
    };
    output_success(&response);
    Ok(())
}

fn handle_org_command(command: OrgCommands) -> lin::Result<()> {
    let response = match command {
        OrgCommands::Info => PlaceholderResponse {
            message: "Command not yet implemented",
            command: "org info".into(),
        },
    };
    output_success(&response);
    Ok(())
}
