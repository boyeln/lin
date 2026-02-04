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

# Delete an issue
lin issue delete ENG-123

# Archive an issue
lin issue archive ENG-123

# Unarchive an issue
lin issue unarchive ENG-123

# List all teams
lin team list

# Get a specific team
lin team get <team-id>

# Show current user
lin user me

# List all users
lin user list

# List workflow states for a team
lin workflow list --team <team-id>
lin workflow list --team ENG

# List all projects
lin project list

# List projects for a specific team
lin project list --team <team-id>

# Get a specific project
lin project get <project-id>

# List issues filtered by project
lin issue list --project <project-id>

# List issues filtered by cycle
lin issue list --cycle <cycle-id>

# List cycles for a team
lin cycle list --team <team-id>
lin cycle list --team ENG

# Get details of a specific cycle (includes issues)
lin cycle get <cycle-id>

# List all labels in the workspace
lin label list

# List labels for a specific team
lin label list --team <team-id>

# Get details of a specific label
lin label get <label-id>

# List issues filtered by label
lin issue list --label <label-id>

# List issues with date range filters
lin issue list --created-after 2024-01-01
lin issue list --created-before 2024-12-31
lin issue list --updated-after 2024-06-01
lin issue list --updated-before 2024-06-30

# Combine date filters with other filters
lin issue list --team ENG --created-after 2024-01-01 --created-before 2024-06-30
lin issue list --assignee me --updated-after 2024-01-01

# Sort issues by field (priority, created, updated, title)
lin issue list --sort priority
lin issue list --sort created
lin issue list --sort updated
lin issue list --sort title

# Sort with explicit direction (asc or desc)
lin issue list --sort priority --order asc    # Urgent issues first
lin issue list --sort priority --order desc   # Low priority issues first
lin issue list --sort created --order desc    # Newest issues first (default)
lin issue list --sort updated --order asc     # Oldest updated first

# Combine sorting with filters
lin issue list --team ENG --sort priority --order asc
lin issue list --assignee me --sort updated --order desc

# Create an issue with labels
lin issue create --team <team-id> --title "New feature" --labels <label-id1> --labels <label-id2>

# Update an issue's labels
lin issue update ENG-123 --labels <label-id1> --labels <label-id2>

# List all documents
lin document list

# List documents for a specific project
lin document list --project <project-id>

# Get details of a specific document (includes content)
lin document get <document-id>

# Create a new document
lin document create --title "My Document" --content "# Hello\n\nWorld"

# Create a document associated with a project
lin document create --title "Project Doc" --content "Content" --project <project-id>

# List attachments on an issue
lin attachment list --issue ENG-123

# Upload a file as an attachment to an issue
lin attachment upload --issue ENG-123 /path/to/file.png

# Get details of a specific attachment (includes download URL)
lin attachment get <attachment-id>

# Link a git branch to an issue
lin issue link-branch ENG-123 feature/my-feature
lin issue link-branch ENG-123 feature/my-feature --repo https://github.com/org/repo

# Link a pull request to an issue
lin issue link-pr ENG-123 https://github.com/org/repo/pull/42

# List linked branches and PRs for an issue
lin issue links ENG-123

# Manage organizations
lin org list
lin org add <name>
lin org remove <name>
lin org set-default <name>

# Search for issues
lin search "authentication bug"
lin search "fix login" --team ENG --limit 10
lin search "urgent" --assignee me --state "In Progress"

# List issue relations (parent, children, blocks, blocked by, related)
lin issue relations ENG-123

# Add a relation between issues
lin issue add-relation ENG-123 ENG-456 --type blocks
lin issue add-relation ENG-123 ENG-456 --type parent
lin issue add-relation ENG-123 ENG-456 --type sub
lin issue add-relation ENG-123 ENG-456 --type blocked_by
lin issue add-relation ENG-123 ENG-456 --type related
lin issue add-relation ENG-123 ENG-456 --type duplicate

# Remove a relation
lin issue remove-relation <relation-id>
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
| Delete issues | ✅ | Delete an issue by ID or identifier |
| Archive/unarchive | ✅ | Archive or unarchive issues |
| Comments | ✅ | Add/list comments on issues |
| Projects | ✅ | List projects, get project details, filter issues by project |
| Cycles/Sprints | ✅ | List cycles, get cycle details with issues, filter issues by cycle |
| Labels | ✅ | List labels, get label details, filter issues by label, add labels to issues |
| Workflow states | ✅ | List workflow states for a team |
| Documents | ✅ | List, get, and create documents |
| File uploads | ✅ | Upload attachments to issues, list and get attachment details |
| Git integration | ✅ | Link branches and PRs to issues, list git links |
| Templates | ❌ | Bulk create issues from templates |
| Search | ✅ | Full-text search for issues |
| **Issue Relations** | | |
| Issue relations | ✅ | Parent/child, blocks/blocked by, relates to, duplicate |
| **Advanced Filtering** | | |
| Date range filters | ✅ | Filter by created/updated date ranges |
| Sort options | ✅ | Sort by priority, created, updated, or title with asc/desc order |
| Combined filters | ❌ | Complex filter combinations |
| **CLI Experience** | | |
| Shell completions | ❌ | Bash, zsh, fish completions |
| Interactive TUI | ❌ | Interactive terminal UI for browsing |
| Caching | ❌ | Cache responses for faster repeated queries |
| Config validation | ❌ | Validate configuration file |
| Examples in help | ✅ | Add usage examples to --help output |
| Human-friendly output | ✅ | Human-readable output (default) |
| JSON output | ✅ | Machine-readable JSON output (--json flag) |
| Colored output | ✅ | Syntax highlighting and colors for terminal |

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

### Colored Output

Human-friendly output includes colors to improve readability:
- Issue identifiers are shown in **bold cyan**
- Status is colored based on workflow state (green for completed, yellow for in progress, etc.)
- Priority levels have distinct colors (red for urgent, yellow for high, etc.)
- Errors are displayed in **bold red**

Colors are automatically disabled when:
- Output is piped to another command
- The `NO_COLOR` environment variable is set (see [no-color.org](https://no-color.org/))

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
