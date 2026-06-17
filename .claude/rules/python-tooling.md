---
description: All Python tooling must use uv. Never use pip, pipx, or virtualenv.
applies-to: python
---

# Rule: Python Tooling

All Python tooling is managed with `uv`. Never use pip, pipx, or virtualenv.

## Package management

- Use `uv tool install` for CLI tools (pre-commit, ruff) — equivalent of `pipx install`.
- Use `uv add` for project dependencies.
- Use `uv run` to execute scripts in the project's managed environment.
- Never use `pip install`, `python -m pip`, `pipx install`, or `virtualenv`.
- Never pass `--break-system-packages` to pip.

## Linting and formatting

- Use `ruff check` for linting and `ruff format` for formatting.
- Run `ruff check --fix && ruff format` before considering any Python change done.
