# lin - A Linear CLI Tool

A fast, ergonomic command-line interface for [Linear](https://linear.app/). Built with Rust.

## Quick Start

```bash
# Install
curl -fsSL https://raw.githubusercontent.com/boyeln/lin/main/install.sh | bash

# Authenticate (get your token from Linear Settings → API)
lin auth work lin_api_xxx

# Set default team (optional - saves typing --team on every command)
lin team switch ENG

# List your issues
lin issue list --assignee me

# Create an issue
lin issue create --title "Fix bug" --state "todo" --priority high

# Update an issue
lin issue update ENG-123 --state "in progress"
```

## Authentication

Get your API token from [Linear Settings → API](https://linear.app/settings/api).

You can manage multiple organizations:

```bash
# Add organizations
lin auth work lin_api_xxx
lin auth personal lin_api_yyy

# Switch between them
lin auth switch personal
lin auth list           # Show all (* = active)

# Remove an organization
lin auth remove work
```

For scripts and automation, use the `LINEAR_API_TOKEN` environment variable:
```bash
export LINEAR_API_TOKEN="lin_api_..."
lin issue list  # No auth command needed
```

## Commands

| Command | Description |
|---------|-------------|
| `lin issue` | List, create, update, delete, archive issues |
| `lin auth` | Manage authentication and organizations |
| `lin team` | List teams and get details |
| `lin user` | Show current user or list all users |
| `lin project` | List and get project details |
| `lin cycle` | List and get cycle/sprint details |
| `lin label` | List and get labels |
| `lin workflow` | List workflow states for a team |
| `lin attachment` | List, get, and upload attachments |
| `lin search` | Full-text search for issues |

Run `lin --help` or `lin <command> --help` for detailed options.

## Common Usage

```bash
# List issues with filters (combine any filters)
lin issue list --team ENG --state "in progress" --assignee me --priority high

# Get issue details
lin issue get ENG-123

# Update issue
lin issue update ENG-123 --state "done" --priority low --assignee @user

# Archive/restore
lin issue archive ENG-123
lin issue unarchive ENG-123

# Search
lin search "authentication bug"

# Teams and users
lin team list
lin user me
```

### Ergonomic Names

lin caches team keys and workflow states, so you can use human-friendly names instead of UUIDs:

```bash
# Use team keys instead of UUIDs
lin issue create --team ENG --title "Task"

# Use state names (case-insensitive)
lin issue update ENG-123 --state "in progress"
lin issue update ENG-123 --state "Done"

# Refresh cache if needed
lin auth sync
```

## Filters

All filters in `lin issue list` can be combined using AND logic:

```bash
--team ENG                       # Team key
--assignee me                    # User ID or "me"
--state "in progress"            # State name
--priority high                  # Priority (0-4 or name)
--project <id>                   # Project ID
--cycle <id>                     # Cycle ID
--label <id>                     # Label ID
--created-after 2024-01-01       # Date filters (YYYY-MM-DD)
--created-before 2024-01-31
--updated-after 2024-01-01
--updated-before 2024-01-31
--sort created                   # Sort field
--order desc                     # Sort order
```

Example:
```bash
# High priority issues assigned to me, created this month
lin issue list --team ENG --assignee me --priority high \
  --created-after 2024-01-01 --created-before 2024-01-31
```

## Output Formats

Default output is human-friendly with colors. Use `--json` for machine-readable output:

```bash
# Human-friendly (colored)
lin issue list

# JSON for scripting
lin issue list --json | jq '.data[].identifier'
lin issue get ENG-123 --json | jq '.data.title'
```

Colors auto-disable when piped or when `NO_COLOR` is set.

## Shell Completions

Generate completions for your shell:

```bash
# Bash
lin completions bash > ~/.local/share/bash-completion/completions/lin

# Zsh
mkdir -p ~/.zfunc
lin completions zsh > ~/.zfunc/_lin
# Add to .zshrc: fpath+=~/.zfunc

# Fish
lin completions fish > ~/.config/fish/completions/lin.fish

# PowerShell, Elvish
lin completions --help
```

## Updating

```bash
lin update              # Update to latest version
lin update --check      # Check if update available
```

Or re-run the install script, or download from [releases](https://github.com/boyeln/lin/releases).

## License

MIT
