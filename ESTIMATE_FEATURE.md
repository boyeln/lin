# Estimate Feature

This document explains the new estimate support added to the `lin` CLI tool.

## Overview

The estimate feature allows you to assign story points or other numeric estimates to Linear issues. You can use either:
1. **Direct numeric values** (e.g., `1`, `2`, `3`, `5`, `8.0`)
2. **Team-configured friendly names** (e.g., `XS`, `S`, `M`, `L`, `XL`)

The feature works similarly to how priority is handled, but with customizable mappings stored in your config file.

## Usage

### Creating Issues with Estimates

```bash
# Using a numeric value
lin issue create --team ENG --title "Fix bug" --estimate 3

# Using a friendly name (requires configuration)
lin issue create --team ENG --title "New feature" --estimate M
```

### Updating Issue Estimates

```bash
# Update with numeric value
lin issue update ENG-123 --estimate 5

# Update with friendly name
lin issue update ENG-123 --estimate L
```

### Viewing Issue Estimates

When you list or get issues, the estimate will be displayed in the output:

```bash
lin issue get ENG-123
```

Output:
```
ENG-123 Fix authentication bug
  Status: In Progress
  Priority: High
  Estimate: 5
  Team: Engineering
```

## Configuration

### Setting Up Team Estimates

Estimates are configured per-team in your `~/.config/lin/config.json` file. You can manually edit this file to add estimate mappings:

```json
{
  "active_org": "my-company",
  "orgs": {
    "my-company": {
      "token": "lin_api_xxxxx",
      "cache": {
        "teams": {
          "ENG": {
            "id": "team-uuid-here",
            "name": "Engineering",
            "states": { ... },
            "estimates": {
              "xs": 1.0,
              "s": 2.0,
              "m": 3.0,
              "l": 5.0,
              "xl": 8.0,
              "xxl": 13.0
            }
          }
        }
      }
    }
  }
}
```

### Estimate Scale Examples

#### Fibonacci Scale
```json
"estimates": {
  "1": 1.0,
  "2": 2.0,
  "3": 3.0,
  "5": 5.0,
  "8": 8.0,
  "13": 13.0,
  "21": 21.0
}
```

#### T-Shirt Sizes
```json
"estimates": {
  "xs": 1.0,
  "s": 2.0,
  "m": 3.0,
  "l": 5.0,
  "xl": 8.0,
  "xxl": 13.0
}
```

#### Linear Scale
```json
"estimates": {
  "small": 1.0,
  "medium": 2.0,
  "large": 3.0
}
```

## Technical Details

### Config Structure

The `estimates` field is a map of lowercase estimate names to numeric values:
- Keys are automatically lowercased for case-insensitive lookup
- Values are stored as `f64` (floating-point numbers)
- Estimates are scoped per-team (each team can have different scales)

### API Integration

The feature integrates with Linear's GraphQL API:
- Issues now include an `estimate` field (type: `Float`)
- The field is optional (can be `null`)
- Estimates are sent to Linear as numeric values

### Resolution Logic

When you provide an estimate:
1. First, try to parse it as a number (e.g., `"5"` → `5.0`)
2. If not a number and cache is enabled, look it up in the team's estimate config
3. If not found, show an error with available estimate names

### Error Handling

If you use a friendly name that's not configured:
```bash
lin issue create --team ENG --title "Test" --estimate XXL
```

You'll get an error like:
```
Estimate 'XXL' not found for team 'ENG'. Available estimates: xs, s, m, l, xl. You can also use a numeric value directly.
```

## Examples

### Full Workflow Example

1. Configure estimates for your team (edit `~/.config/lin/config.json`):
```json
"estimates": {
  "xs": 1.0,
  "s": 2.0,
  "m": 3.0,
  "l": 5.0,
  "xl": 8.0
}
```

2. Create an issue with an estimate:
```bash
lin issue create \
  --team ENG \
  --title "Implement user authentication" \
  --estimate L \
  --priority high
```

3. Update the estimate later:
```bash
lin issue update ENG-123 --estimate XL
```

4. View the issue:
```bash
lin issue get ENG-123
```

## Notes

- Estimate names are **case-insensitive** (both `M` and `m` work)
- You can always use numeric values directly, even without configuration
- Each team can have a different estimate scale
- The feature works in both cached mode (with config) and env var mode (numeric only)
- Estimates appear in both human-friendly and JSON output formats
