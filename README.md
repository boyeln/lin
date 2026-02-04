# lin - A Linear CLI Tool

A command-line interface for [Linear](https://linear.app/), the issue tracking tool. Built with Rust for speed and reliability.

## Installation

```bash
cargo install lin
```

Or build from source:

```bash
git clone https://github.com/your-repo/lin.git
cd lin
cargo build --release
```

## Authentication

lin supports multiple authentication methods (in order of priority):

1. `--api-token <token>` CLI flag
2. `LINEAR_API_TOKEN` environment variable
3. `~/.config/lin/config.json` configuration file

To set up authentication:

```bash
# Using environment variable
export LINEAR_API_TOKEN="lin_api_..."

# Or add an organization via CLI (reads token from stdin)
echo "lin_api_..." | lin org add my-org

# Set a default organization
lin org set-default my-org
```

## Usage

```bash
# List all issues
lin issue list

# List issues with filters
lin issue list --team ENG --assignee me --state "In Progress"

# Get a specific issue by identifier
lin issue get ENG-123

# Get a specific issue by UUID
lin issue get 550e8400-e29b-41d4-a716-446655440000

# Create a new issue
lin issue create --team <team-id> --title "Fix bug" --description "Details here" --priority 2

# Update an issue
lin issue update ENG-123 --title "Updated title" --state <state-id>

# List all teams
lin team list

# Get a specific team
lin team get <team-id>

# Show current user
lin user me

# List all users
lin user list

# Manage organizations
lin org list
lin org add <name>
lin org remove <name>
lin org set-default <name>
```

## Features

| Feature | Status | Description |
|---------|--------|-------------|
| **Issue Management** | | |
| List issues | ✅ | List issues with filters (team, assignee, state, limit) |
| Get issue | ✅ | Get issue by UUID or identifier (ABC-123) |
| Create issue | ✅ | Create issues with title, description, priority, assignee, state |
| Update issue | ✅ | Update issue fields (title, description, priority, assignee, state) |
| **Teams** | | |
| List teams | ✅ | List all teams in workspace |
| Get team | ✅ | Get team details by ID |
| **Users** | | |
| Current user | ✅ | Show authenticated user info |
| List users | ✅ | List all users in workspace |
| **Configuration** | | |
| Multiple orgs | ✅ | Support for multiple organizations |
| Default org | ✅ | Set a default organization |
| Token from CLI | ✅ | Pass token via --api-token flag |
| Token from env | ✅ | Read token from LINEAR_API_TOKEN |
| Token from file | ✅ | Read token from config file |
| Comments | ❌ | Add/list comments on issues |
| Projects | ❌ | List and filter by projects |
| Cycles/Sprints | ❌ | View and manage cycles |
| Labels | ❌ | Manage issue labels |
| Workflow states | ❌ | List workflow states for a team |
| Documents | ❌ | Create/manage documents |
| File uploads | ❌ | Upload attachments |
| Templates | ❌ | Bulk create issues from templates |
| Git integration | ❌ | Link issues to branches |
| Search | ❌ | Full-text search for issues |
| **Core Issue Operations** | | |
| Delete issues | ❌ | Delete an issue by ID or identifier |
| Archive/unarchive | ❌ | Archive or unarchive issues |
| Issue relations | ❌ | Parent/child, blocks/blocked by, relates to |
| **Advanced Filtering** | | |
| Date range filters | ❌ | Filter by created/updated date ranges |
| Sort options | ❌ | Sort by priority, updated date, created date |
| Combined filters | ❌ | Complex filter combinations |
| **CLI Experience** | | |
| Shell completions | ❌ | Bash, zsh, fish completions |
| Interactive TUI | ❌ | Interactive terminal UI for browsing |
| Caching | ❌ | Cache responses for faster repeated queries |
| Config validation | ❌ | Validate configuration file |
| Examples in help | ✅ | Add usage examples to --help output |
| Human-friendly output | ✅ | Human-readable output (default) |
| JSON output | ✅ | Machine-readable JSON output (--json flag) |
| Colored output | ❌ | Syntax highlighting and colors for terminal |

## Priority Values

When creating or updating issues, use these priority values:

| Value | Priority |
|-------|----------|
| 0 | No priority |
| 1 | Urgent |
| 2 | High |
| 3 | Normal/Medium |
| 4 | Low |

## Output Format

By default, lin outputs human-friendly text:

```bash
$ lin issue list
ENG-123 Fix the authentication bug
  Status: In Progress
  Priority: High
  Assignee: John Doe

ENG-124 Update documentation
  Status: Todo
  Priority: Normal

$ lin user me
John Doe (JD)
  john@example.com

$ lin team list
[ENG] Engineering
  The engineering team

[DES] Design
```

### JSON Output

Use the `--json` flag for machine-readable JSON output, useful for scripting:

```bash
# Get JSON output
lin issue list --json

# Pipe to jq for processing
lin issue list --json | jq '.data[].identifier'

# Get just the issue title
lin issue get ENG-123 --json | jq '.data.title'

# Count issues
lin issue list --team ENG --json | jq '.data | length'
```

## License

MIT
