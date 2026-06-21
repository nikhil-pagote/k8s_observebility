#!/usr/bin/env bash
# Lint a YAML file immediately after an Edit tool call.
file=$(python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('tool_input',{}).get('file_path',''))" 2>/dev/null)
echo "$file" | grep -qE '\.ya?ml$' || exit 0
which yamllint >/dev/null 2>&1 || { echo "yamllint not available"; exit 0; }
yamllint -d relaxed "$file" && echo "$file: lint OK"
