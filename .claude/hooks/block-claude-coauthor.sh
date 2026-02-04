#!/bin/bash
# Block git commits that include Claude as a co-author

INPUT=$(cat)
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // empty')

# Only check git commit commands
if echo "$COMMAND" | grep -q '^git commit'; then
  # Check for Co-authored-by with Claude (case-insensitive)
  if echo "$COMMAND" | grep -iqE 'Co-authored-by:.*claude'; then
    echo "Commits must not attribute Claude as co-author" >&2
    exit 2
  fi
fi

exit 0
