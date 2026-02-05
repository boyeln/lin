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

Run `lin --help` for full documentation. Each subcommand also supports `--help` for detailed options.

### Commands

| Command | Description |
|---------|-------------|
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
| `lin org` | Manage organizations and config |
| `lin cache` | View cache status or clear cache |
| `lin completions` | Generate shell completions |

### Quick Examples

```bash
lin issue list --team ENG --assignee me
lin issue get ENG-123
lin issue create --team ENG --title "Fix bug"
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

## Caching

lin caches API responses locally to speed up repeated queries. The cache is stored in `~/.cache/lin/` (or the platform-appropriate cache directory).

### Cache TTLs

Different data types have different cache lifetimes based on how frequently they change:

| Data Type | TTL | Rationale |
|-----------|-----|-----------|
| Teams | 1 hour | Teams rarely change |
| Users | 1 hour | User list is stable |
| Workflow states | 1 hour | Workflow configuration rarely changes |
| Labels | 30 minutes | Labels change occasionally |
| Projects | 15 minutes | Projects change moderately |
| Cycles | 15 minutes | Cycle data changes moderately |
| Documents | 10 minutes | Documents are edited more frequently |
| Issues | 5 minutes | Issues change frequently |
| Comments | 5 minutes | Comments are added frequently |
| Search results | 2 minutes | Search should return fresh results |

### Cache Commands

```bash
# View cache statistics (size, entries, expired entries)
lin cache status

# Clear all cached entries
lin cache clear
```

### Bypassing the Cache

Use the `--no-cache` flag to bypass the cache and fetch fresh data from the API:

```bash
# Fetch fresh data, ignoring any cached responses
lin --no-cache issue list --team ENG
lin --no-cache team list
```

### Cache Location

The cache is stored in:
- Linux: `~/.cache/lin/`
- macOS: `~/Library/Caches/lin/`
- Windows: `{FOLDERID_LocalAppData}/lin/`

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
