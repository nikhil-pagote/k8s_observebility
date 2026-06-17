---
description: Install and manage Python CLI tools and packages for this project using uv. Never use pip, pipx, or virtualenv.
argument-hint: "[--install-all | --tool <name>]"
allowed-tools:
  - Bash
---

All Python tooling is managed with `uv`. Use `uv tool install` for CLI tools (pre-commit, ruff)
and `uv add` for project dependencies.

## Install all project tools

```bash
uv tool install pre-commit
uv tool install ruff
```

## Install a specific tool

```bash
uv tool install <name>
```

## Verify installed tools

```bash
uv tool list
```

## Linting and formatting with ruff

```bash
# Lint YAML-adjacent Python scripts (if any)
uv run ruff check .

# Format
uv run ruff format .

# Fix auto-fixable issues
uv run ruff check --fix .
```

## Rules

- Never use `pip install`, `pipx install`, `python -m pip`, or `virtualenv`.
- Never pass `--break-system-packages` to pip.
- `uv tool install` is the equivalent of `pipx install` — installs CLI tools in isolated environments.
- `uv run` executes scripts in the project's managed environment.
