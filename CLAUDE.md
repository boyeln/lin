# lin

Rust CLI for Linear issue tracking. Uses clap, reqwest (blocking), serde, thiserror.

## Commands

```bash
cargo test                        # Run unit tests
cargo clippy && cargo fmt --check # Lint before commit
```

## Project Layout

- `src/commands/` — CLI command handlers
- `src/api/` — GraphQL client and queries
- `src/models/` — Domain types (Issue, Team, User)
- `tests/` — Integration tests (`#[ignore]`, CI only)

## Key Conventions

- All commands return `Result<(), LinError>`
- JSON-only output via `output::output_success`
- Token resolution: CLI flag > env var > config file
- Config location: `~/.config/lin/config.json`

## Adding Commands

1. Add query to `src/api/queries.rs`
2. Add types to `src/models/types.rs`
3. Add command in `src/commands/`
4. Wire up in `src/main.rs`
5. Add tests with mockito

## Linear API

- Endpoint: `https://api.linear.app/graphql`
- Auth header: `Authorization: <token>` (no Bearer prefix)
- Rate limit: 1500 req/hour

## Testing

- Unit tests use mockito for HTTP mocking
- Integration tests: `cargo test -- --ignored --test-threads=1`
- Test issues are prefixed with `[lin-test]`

## Git

- Use [Conventional Commits](https://www.conventionalcommits.org/): `feat:`, `fix:`, `docs:`, `test:`, `refactor:`
- **All changes must go through pull requests** - no direct commits to main (except for releases)
- PR titles should follow conventional commit format
- **Before every commit**, always run:
  ```bash
  cargo test && cargo fmt --check && cargo clippy
  ```
- Do not commit if any of these checks fail
- Do not mention Claude or AI assistance in commit messages
