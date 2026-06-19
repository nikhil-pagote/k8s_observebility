---
description: Keep docs/ and .claude/specs/ in sync with code changes before committing
alwaysApply: true
---

# Rule: Update Docs and Specs Before Commit

Any change to stack configuration, architecture, routing, or operational behaviour must be reflected in the relevant documentation **before** the commit that introduces the change. Do not commit code changes and defer doc updates to a follow-up commit.

## Scope

| Directory | Update when… |
|---|---|
| `docs/troubleshooting_guide.md` | A new failure mode is discovered, a workaround changes, or a known issue is resolved |
| `docs/promql.md` | A new scrape job is added or removed, a metric namespace changes, or a new query pattern is established |
| `.claude/specs/architecture.md` | Ingress routes, namespace layout, sync-wave order, or data-flow changes |
| `.claude/specs/design_doc.md` | A component is reconfigured in a non-obvious way (e.g. label limits, path prefix, auth strategy) |
| `.claude/specs/otel-pipeline.md` | OTel receivers, processors, exporters, or pipeline routing changes |
| `CLAUDE.md` | A new doc, spec, skill, agent briefing, or project-wide constraint is added |

## What counts as a required update

- New Traefik route or middleware → update `architecture.md` ingress table and `traefik.md` agent briefing
- New or removed OTel scrape job → update `promql.md` and `otel-pipeline.md`
- New component or ArgoCD app → update `architecture.md`, `design_doc.md`, `CLAUDE.md` subagent table
- New `docs/` file → add a row to the `## Docs` section in `CLAUDE.md`
- New `.claude/specs/` file → add a reference in `CLAUDE.md`
- Operational gotcha discovered during debugging → add to `troubleshooting_guide.md`

## What does NOT require a doc update

- Bumping a chart version with no behavioural change
- Adjusting resource limits (CPU/memory)
- Fixing a typo or label in a Helm value that has no user-visible effect
- Changes to dashboard JSON that don't affect query patterns

## Checklist before `git commit`

1. Does this change affect how a component is accessed, configured, or debugged? → update the relevant spec.
2. Does this change introduce a new query target or metric prefix? → update `docs/promql.md`.
3. Does this change resolve or introduce a known failure mode? → update `docs/troubleshooting_guide.md`.
4. Does this change add a new file in `docs/` or `.claude/specs/`? → add it to `CLAUDE.md`.
