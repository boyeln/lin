# CLAUDE.md - Development Guide

## Project Overview

`lin` is a command-line interface for Linear, the issue tracking tool. Built with Rust using:
- **clap** (4.x) for CLI argument parsing (derive macros)
- **reqwest** (0.12) for HTTP requests (blocking)
- **serde** for JSON serialization
- **thiserror** for error handling
- **mockito** for testing

## Common Commands

```bash
# Build the project
cargo build

# Run in development
cargo run -- <args>

# Example: list teams
cargo run -- team list

# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_name

# Run tests for a specific module
cargo test commands::issue

# Check code without building
cargo check

# Format code
cargo fmt

# Run clippy lints
cargo clippy

# Build release binary
cargo build --release
```

## Project Structure

```
lin/
├── Cargo.toml              # Dependencies and project metadata
├── Cargo.lock              # Locked dependency versions
├── README.md               # User-facing documentation
├── CLAUDE.md               # This file - development guide
└── src/
    ├── main.rs             # Entry point and CLI definition (clap)
    ├── lib.rs              # Library root, exports all modules
    ├── error.rs            # LinError enum with thiserror
    ├── output.rs           # JSON output utilities
    ├── config.rs           # Configuration file management
    ├── auth.rs             # API token resolution
    ├── api/
    │   ├── mod.rs          # API module exports
    │   ├── client.rs       # GraphQL client (reqwest-based)
    │   └── queries.rs      # GraphQL query/mutation strings
    ├── commands/
    │   ├── mod.rs          # Command module exports
    │   ├── issue.rs        # Issue commands (list, get, create, update)
    │   ├── team.rs         # Team commands (list, get)
    │   ├── user.rs         # User commands (me, list)
    │   └── org.rs          # Organization config commands
    └── models/
        ├── mod.rs          # Model exports
        └── types.rs        # Domain types (Issue, Team, User, etc.)
```

## Architecture

### Layer Overview
1. **CLI Layer** (`main.rs`, `commands/`): Parses args, calls API, formats output
2. **API Layer** (`api/`): GraphQL client and query definitions
3. **Models** (`models/`): Domain types matching Linear's GraphQL schema
4. **Config** (`config.rs`, `auth.rs`): Token and organization management

### Key Patterns
- All commands return `Result<(), LinError>`
- JSON-only output for scriptability (via `output::output_success`)
- Token resolution: CLI flag > env var > config file
- Strong typing for all API responses via serde
- Tests use mockito for HTTP mocking

### Error Handling
```rust
// LinError variants:
LinError::Config(String)  // Config file issues
LinError::Api(String)     // API/GraphQL errors
LinError::Io(io::Error)   // File I/O errors
LinError::Parse(String)   // Parsing errors
```

## Testing

Tests use mockito for HTTP mocking. Each module has its own tests.

```bash
# Run all tests (137 tests)
cargo test

# Run tests for specific modules
cargo test api::client
cargo test commands::issue
cargo test config

# Run with verbose output
cargo test -- --nocapture
```

## Linear API

- GraphQL endpoint: `https://api.linear.app/graphql`
- Auth: `Authorization: <token>` header (no Bearer prefix)
- Rate limits: 1500 requests per hour
- Docs: https://developers.linear.app/docs/graphql/working-with-the-graphql-api

## Environment Variables

- `LINEAR_API_TOKEN`: API token for authentication (priority 2)
- Token can also be passed via `--api-token` flag (priority 1) or config file (priority 3)

## Config File

Location: `~/.config/lin/config.json`

```json
{
  "organizations": {
    "my-org": "lin_api_xxx..."
  },
  "default_org": "my-org"
}
```

## Adding New Commands

1. Add GraphQL query/mutation to `src/api/queries.rs`
2. Add response types to `src/models/types.rs`
3. Create command function in `src/commands/`
4. Wire up in `src/main.rs`
5. Add tests with mockito
