# lin - A Linear CLI Tool

A command-line interface for [Linear](https://linear.app/), the issue tracking tool. Built with Rust for speed and reliability.

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/boyeln/lin/main/install.sh | bash
```

This downloads the latest binary to `~/.local/bin` (Linux/macOS).

### Updating

```bash
lin update              # Update to latest version
lin update --check      # Check if update is available
```

### Other options

- **Manual download**: Get binaries from the [releases page](https://github.com/boyeln/lin/releases)
- **From source**: `git clone && cargo build --release`

## Authentication

Get your API token from [Linear Settings → API](https://linear.app/settings/api).

### First Time Setup

```bash
lin auth work lin_api_...
```

This will:
- Validate your token
- Store it securely in `~/.config/lin/config.json`
- Automatically sync all teams and workflow states

### Multi-Organization Support

You can authenticate with multiple Linear organizations:

```bash
# Add your work organization
lin auth work lin_api_xxx

# Add your personal organization
lin auth personal lin_api_yyy

# Switch between them
lin auth switch personal

# List all organizations (* = active)
lin auth list

# Show current status
lin auth
```

### CI/CD and Automation

For CI/CD pipelines and scripts, use the `LINEAR_API_TOKEN` environment variable:

```bash
export LINEAR_API_TOKEN="lin_api_..."
lin issue list  # No authentication required
```

This bypasses the auth system and works immediately.

## Organization Management

The `lin auth` command manages your Linear organizations and credentials.

### Authentication Commands

```bash
# Authenticate with an organization (and sync data)
lin auth <name> <token>

# Show current organization status
lin auth

# List all organizations
lin auth list

# Switch active organization
lin auth switch <name>

# Remove an organization
lin auth remove <name>

# Manually refresh teams and workflow states
lin auth sync
```

### Intelligent Caching

When you authenticate, lin automatically caches your teams and workflow states:

```bash
lin auth work lin_api_xxx
# ✓ Authenticated as 'work'
# ✓ Synced 3 teams: ENG, DESIGN, PRODUCT
# ✓ Cached 15 workflow states
```

This enables:
- **Fast team resolution**: Use `--team ENG` instead of UUIDs
- **Ergonomic state names**: Use `--state "todo"` instead of state UUIDs
- **Zero API calls**: Team and state lookups use local cache
- **Auto-refresh**: Cache updates automatically on errors

The cache is stored in `~/.config/lin/config.json` alongside your tokens.

## Usage

Run `lin --help` for full documentation. Each subcommand also supports `--help` for detailed options.

### Commands

| Command | Description |
|---------|-------------|
| `lin auth` | Authenticate and manage organizations |
| `lin issue` | List, create, update, delete, archive issues |
| `lin team` | List and get team details |
| `lin user` | Show current user or list all users |
| `lin project` | List and get project details |
| `lin cycle` | List and get cycle/sprint details |
| `lin label` | List and get labels |
| `lin workflow` | List workflow states for a team |
| `lin document` | List, get, and create documents |
| `lin attachment` | List, get, and upload attachments |
| `lin search` | Full-text search for issues |
| `lin completions` | Generate shell completions |

### Quick Examples

```bash
# List issues using team key and state name
lin issue list --team ENG --state "in progress" --assignee me

# Get issue details
lin issue get ENG-123

# Create issue with ergonomic names
lin issue create --team ENG --title "Fix bug" --state "todo" --priority high

# Update issue state by name (case-insensitive)
lin issue update ENG-123 --state "done"

# Search for issues
lin search "authentication"
```

## Shell Completions

lin supports generating shell completion scripts for bash, zsh, fish, PowerShell, and elvish.

### Bash

```bash
# Generate completions and save to bash-completion directory
lin completions bash > ~/.local/share/bash-completion/completions/lin

# Or source directly in your .bashrc
echo 'eval "$(lin completions bash)"' >> ~/.bashrc
```

### Zsh

```bash
# Create the completions directory if needed
mkdir -p ~/.zfunc

# Generate completions
lin completions zsh > ~/.zfunc/_lin

# Add to your .zshrc (before compinit):
# fpath+=~/.zfunc
# autoload -Uz compinit && compinit
```

### Fish

```bash
# Generate completions to fish completions directory
lin completions fish > ~/.config/fish/completions/lin.fish
```

### PowerShell

```powershell
# Generate completions
lin completions powershell > _lin.ps1

# Add to your PowerShell profile
. _lin.ps1
```

### Elvish

```bash
# Generate completions
lin completions elvish > ~/.elvish/lib/lin.elv

# Add to your rc.elv:
# use lin
```

## Ergonomic Team and State Names

lin caches your organization's teams and workflow states for fast, user-friendly commands.

### Team Keys

Use short team keys instead of UUIDs:

```bash
# Before: Required UUID
lin issue create --team a1b2c3d4-... --title "Task"

# After: Use team key
lin issue create --team ENG --title "Task"
```

### State Names

Use natural state names (case-insensitive):

```bash
# All of these work
lin issue create --team ENG --state "todo" --title "Task"
lin issue update ENG-123 --state "in progress"
lin issue update ENG-123 --state "IN PROGRESS"
lin issue update ENG-123 --state "Done"
```

### How It Works

When you authenticate, lin automatically:
1. Fetches all teams and their workflow states
2. Caches them in `~/.config/lin/config.json`
3. Uses the cache for instant resolution (zero API calls)
4. Auto-refreshes on errors or with `lin auth sync`

### Caching

lin also caches API responses to speed up repeated queries. The cache is stored in `~/.cache/lin/` (or the platform-appropriate cache directory).

**Cache TTLs** by data type:
- Teams, Users, Workflow states: 1 hour
- Labels: 30 minutes
- Projects, Cycles: 15 minutes
- Documents: 10 minutes
- Issues, Comments: 5 minutes
- Search results: 2 minutes

**Cache Commands:**
```bash
lin cache status  # View cache statistics
lin cache clear   # Clear all cached entries
```

**Bypass cache:**
```bash
lin --no-cache issue list --team ENG
```

## Priority Values

When creating or updating issues, use these priority values:

| Value | Priority |
|-------|----------|
| 0 | No priority |
| 1 | Urgent |
| 2 | High |
| 3 | Normal/Medium |
| 4 | Low |

## Combined Filters

All filters in `lin issue list` can be combined together using AND logic. When multiple filters are specified, only issues matching ALL criteria are returned.

### Available Filters

| Filter | Description |
|--------|-------------|
| `--team` | Filter by team identifier (e.g., "ENG") |
| `--assignee` | Filter by assignee (user ID or "me") |
| `--state` | Filter by state name (e.g., "In Progress") |
| `--project` | Filter by project ID |
| `--cycle` | Filter by cycle ID |
| `--label` | Filter by label ID |
| `--priority` | Filter by priority (0-4 or: none, urgent, high, normal, low) |
| `--created-after` | Filter issues created after date (YYYY-MM-DD) |
| `--created-before` | Filter issues created before date (YYYY-MM-DD) |
| `--updated-after` | Filter issues updated after date (YYYY-MM-DD) |
| `--updated-before` | Filter issues updated before date (YYYY-MM-DD) |

### Example Combinations

```bash
# High priority issues in a specific team assigned to me
lin issue list --team ENG --assignee me --priority high

# Urgent issues in the Engineering team
lin issue list --team ENG --priority urgent

# Issues in progress created this month
lin issue list --state "In Progress" --created-after 2024-01-01 --created-before 2024-01-31

# My urgent issues sorted by creation date
lin issue list --assignee me --priority urgent --sort created --order desc

# Team issues in a specific project with high priority
lin issue list --team ENG --project <project-id> --priority high

# All filters combined
lin issue list --team ENG --assignee me --state "In Progress" --priority high --created-after 2024-01-01 --sort priority --order asc
```

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
