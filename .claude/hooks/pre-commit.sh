#!/usr/bin/env bash
# Lint staged YAML files before git commit (Claude-executed commits only).
# Also enforced by the pre-commit framework for direct terminal commits.
cmd=$(python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('tool_input',{}).get('command',''))" 2>/dev/null)
echo "$cmd" | grep -qE '^git commit' || exit 0

staged=$(git diff --cached --name-only --diff-filter=ACM 2>/dev/null | grep -E '\.ya?ml$' || true)
[ -n "$staged" ] || exit 0

echo "--- pre-commit: yamllint on staged files ---"
which yamllint >/dev/null 2>&1 || { echo "yamllint not found — skipping lint"; exit 0; }
echo "$staged" | xargs yamllint -d relaxed
