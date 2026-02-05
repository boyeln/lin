# Releasing lin

This document describes how to release new versions of lin.

## Overview

Releases are distributed via GitHub Releases. When you create a release:
1. GitHub Actions builds binaries for all supported platforms
2. Binaries are automatically attached to the release
3. Users can install/update via `curl | bash` or `lin update`

## Creating a Release

### 1. Update version and changelog

```bash
# Update version in Cargo.toml
# Update CHANGELOG.md with new changes

git add Cargo.toml CHANGELOG.md
git commit -m "chore: release v0.2.0"
git push
```

### 2. Create and push a tag

```bash
git tag v0.2.0
git push origin v0.2.0
```

### 3. Create GitHub release

```bash
gh release create v0.2.0 --generate-notes
```

Or create the release manually in the GitHub UI.

### 4. Wait for builds

GitHub Actions will automatically:
- Build binaries for Linux (x86_64, aarch64), macOS (Intel, Apple Silicon), and Windows
- Upload binaries to the release

## How Users Get Updates

Users can update in two ways:

1. **Self-update command** (recommended):
   ```bash
   lin update
   ```

2. **Re-run install script**:
   ```bash
   curl -fsSL https://raw.githubusercontent.com/boyeln/lin/main/install.sh | bash
   ```

## Supported Platforms

| Platform | Architecture | Binary |
|----------|--------------|--------|
| Linux | x86_64 | `lin-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz` |
| Linux | ARM64 | `lin-vX.Y.Z-aarch64-unknown-linux-gnu.tar.gz` |
| macOS | Intel | `lin-vX.Y.Z-x86_64-apple-darwin.tar.gz` |
| macOS | Apple Silicon | `lin-vX.Y.Z-aarch64-apple-darwin.tar.gz` |
| Windows | x86_64 | `lin-vX.Y.Z-x86_64-pc-windows-msvc.zip` |

## Version Policy

Use [Semantic Versioning](https://semver.org/):

- **Major** (1.0.0): Breaking changes to CLI interface or output format
- **Minor** (0.1.0): New features, commands, or flags
- **Patch** (0.0.1): Bug fixes, documentation, performance improvements

## Commit Message Guidelines

Use [Conventional Commits](https://www.conventionalcommits.org/) for clear changelog generation:

```
feat: add new command for workflow management
fix: handle rate limiting correctly
docs: update installation instructions
refactor: split api module into separate files
```
