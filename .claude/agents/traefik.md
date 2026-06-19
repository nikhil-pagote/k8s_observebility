---
name: traefik
description: Manage Traefik — IngressRoutes, reconfigure entrypoints, check routing and sync
model: claude-sonnet-4-6
tools:
  - Bash
  - Read
  - Edit
  - Skill
scope: traefik
tags:
  - ingress
  - networking
depends-on: []
---

# Agent Briefing — Traefik

## Role in stack
Ingress controller and reverse proxy. Deployed first (sync-wave 0) because all other apps create IngressRoutes that depend on it.

NodePort: `30080 → :80`, `30443 → :443`. All UIs route through Traefik:

| Path | Backend | Notes |
|---|---|---|
| `/grafana` | grafana.observability:80 | sub-path routing |
| `/vmui` | victoria-metrics.observability:8428 | no prefix strip |
| `/jaeger` | jaeger.observability:16686 | |
| `/traefik` | Traefik dashboard | redirects to `/dashboard/` |
| `/argocd` | argocd-server.argocd:80 | cross-namespace IngressRoute |

## Key files
```
argocd-apps/traefik/
  app.yaml                       # sync-wave: "0"
  values/values.yaml
  chart/                         # Vendored traefik/traefik chart

argocd-apps/observability-ingress.yaml   # IngressRoute definitions for all observability UIs
```

## Skills

| When | Skill | Why |
|---|---|---|
| After editing `values/values.yaml` or `observability-ingress.yaml` | `validate` | Dry-run all manifests before committing |
| After ArgoCD syncs | `stack-status` | Confirm Traefik pod is Running and all IngressRoutes are accepted |

## Status and logs
```bash
kubectl get application traefik -n argocd -o wide
kubectl get pods -n traefik
kubectl get svc -n traefik traefik
kubectl logs -n traefik deployment/traefik --tail=50

# Check IngressRoutes
kubectl get ingressroute -n observability
kubectl get ingressroute -n traefik
```

## Add a new route
Edit `argocd-apps/observability-ingress.yaml` — add an `IngressRoute` resource pointing to the new service. Commit and push.

## Reconfigure
Edit `argocd-apps/traefik/values/values.yaml`, commit, push — ArgoCD auto-syncs.

Common changes:
- `ports` — adjust NodePort numbers
- `logs.access.enabled: true` — enable access logs
- `metrics.prometheus.enabled: true` — expose Traefik metrics (already scraped by OTel Collector)

## Force sync
```bash
kubectl annotate application traefik -n argocd argocd.argoproj.io/refresh=normal --overwrite
```

## Constraint
Traefik **must** be Running before any other app that creates IngressRoutes. If other apps fail to sync with IngressRoute errors, check Traefik pod status first.
