# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Container runtime

**Podman** is the container runtime — Docker is not used. kind talks to Podman via two env vars set in `.envrc` and injected by Claude Code via `.claude/settings.json`:

```bash
KIND_EXPERIMENTAL_PROVIDER=podman
DOCKER_HOST=unix:///run/user/1000/podman/podman.sock
```

Load `.envrc` once per shell session (or use [direnv](https://direnv.net/) to auto-load):
```bash
source .envrc
```

Verify the Podman socket is running before any `kind` command:
```bash
systemctl --user status podman.socket   # start if inactive
```

## Cluster operations

```bash
# Apply the full ArgoCD app stack
kubectl apply -k argocd-apps/

# Check everything
kubectl get pods --all-namespaces
kubectl get applications -n argocd

# Watch a namespace come up
kubectl get pods -n observability -w

# Restart a deployment
kubectl rollout restart deployment/<name> -n observability

# Tail logs
kubectl logs -n observability deployment/opentelemetry-collector -f
```

## Git hooks

| Hook | File | Fires on | Does |
|---|---|---|---|
| `pre-commit` | `.claude/hooks/pre-commit.md` | `git commit` | yamllint on staged YAML (Claude); `pre-commit` framework for direct git use |
| `pre-push` | `.claude/hooks/pre-push.md` | `git push` | `kubectl --dry-run=client` on all manifests |

Hooks are `.md` files in `.claude/hooks/` — logic is wired into `settings.json` PreToolUse matchers for Claude-executed git commands.

For direct `git commit` from the terminal, the `pre-commit` framework applies the rules in `.pre-commit-config.yaml`. Install it once per clone:
```bash
uv tool install pre-commit   # if not already installed
pre-commit install
```

## Architecture

**GitOps layer** (`argocd-apps/`) — ArgoCD `Application` CRDs. `kustomization.yaml` is the single entry point; edit it to add/remove apps. ArgoCD reconciles the actual Helm charts from upstream.

**Cluster topology:**
- Kind cluster `observability-cluster`: 1 control-plane + 3 workers
- NodePort mappings: `30080 → :80`, `30443 → :443`
- All UIs via Traefik at `http://localhost:30080/{grafana,prometheus,jaeger,argocd}` and `/traefik` redirects to `/dashboard/`

**Namespace layout:**

| Namespace | Contents |
|---|---|
| `traefik` | Traefik ingress controller (sync-wave 0 — deployed first) |
| `argocd` | ArgoCD server + application controller |
| `observability` | Prometheus, Grafana, Jaeger, Loki, OTel Collector |

**Data flow:** Apps → OTel Collector (OTLP :4317) → Prometheus (metrics) + Jaeger (traces) + Loki (logs) → Grafana

## Key constraints

- ArgoCD sync-wave ordering: Traefik must be running before other apps create IngressRoutes.
- Grafana sub-path routing requires `serve_from_sub_path=true` and matching `root_url` in Helm values.
- **Log store is Loki** — the guide `otel operator for k8s.md` references Elasticsearch/ClickHouse, but this project uses Loki (single-binary, filesystem storage). Logs are received via OTLP (not filelog DaemonSet).
- OTel Collector in-cluster endpoint: `http://opentelemetry-collector.observability.svc.cluster.local:4317`

## Project skills

Skills in `.claude/skills/` (invoke via the Skill tool):

| Skill | Purpose |
|---|---|
| `validate` | `kubectl --dry-run=client` on all manifests |
| `stack-status` | Component health table across all namespaces |
| `verify-otel` | End-to-end pillar check (metrics → Prometheus, traces → Jaeger, logs → Loki) |
| `deploy` | Guided deployment with pre-flight checks |
| `helm` | Add repos, pull charts locally into `argocd-apps/<app>/chart/` |
| `kind-cluster` | Start, stop, restart, or check status of the Kind cluster |

Architecture and pipeline details: `.claude/specs/`

Project rules: `.claude/rules/`
