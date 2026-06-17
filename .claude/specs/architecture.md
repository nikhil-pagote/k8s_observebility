# Architecture Spec — k8s Observability Stack

## Overview

A local Kubernetes observability POC using **OpenTelemetry** as the unified collection layer, deployed via GitOps (ArgoCD), and exposed through a single Traefik ingress on path-based routing.

## Three Pillars

| Pillar | Tool | Role |
|---|---|---|
| Metrics | Prometheus + Grafana | Scrape, store, and visualize metrics |
| Traces | Jaeger | Distributed trace storage and UI |
| Logs | Loki | Log aggregation and querying |

## Data Flow

```
Applications (instrumented)
        │
        ▼ OTLP gRPC :4317
OTel Collector — Deployment
        │
   ┌────┴──────┐
   ▼           ▼           ▼
Prometheus  Jaeger       Loki
   │           │           │
   └─────┬─────┘           │
         ▼                 │
      Grafana ◄────────────┘
```

## Component Map

| Component | Namespace | Helm Chart | Purpose |
|---|---|---|---|
| Traefik | traefik | traefik/traefik | Ingress, NodePort 30080/30443 |
| ArgoCD | argocd | argo/argo-cd | GitOps reconciler |
| Prometheus | observability | prometheus-community/prometheus | Metrics store |
| Grafana | observability | grafana/grafana | Unified visualization |
| Jaeger | observability | jaegertracing/jaeger | Trace store (in-memory for POC) |
| Loki | observability | grafana/loki | Log store (single-binary, filesystem) |
| OTel Collector | observability | open-telemetry/opentelemetry-collector | OTLP receiver, fan-out to all backends |

## Ingress Routing

All UIs via Traefik at `http://localhost:30080`:

| Path | Service | Port | Notes |
|---|---|---|---|
| `/grafana` | grafana | 80 | sub-path routing via `serve_from_sub_path=true` |
| `/prometheus` | prometheus-server | 80 | |
| `/jaeger` | jaeger | 16686 | |
| `/traefik` | Traefik Dashboard | — | redirects to `/dashboard/` via middleware |
| `/argocd` | argocd-server (argocd ns) | 80 | cross-namespace IngressRoute in traefik ns |

## Kind Cluster

Name: `observability-cluster`
Kubernetes: v1.33.1
Topology: 1 control-plane + 3 workers
Port mappings: `30080 → :80`, `30443 → :443`
Config: `kind-config.yaml`

## Container Runtime

**Podman** — Docker is not used. Set via `.envrc` (sourced per shell session):

```bash
KIND_EXPERIMENTAL_PROVIDER=podman
DOCKER_HOST=unix:///run/user/1000/podman/podman.sock
```

Also injected automatically by Claude Code via the `env` block in `.claude/settings.json`.

## OTel Operator

Not used. The stack runs a plain `opentelemetry-collector-contrib` Deployment (no OTel Operator, no cert-manager). The guide `otel operator for k8s.md` documents an alternative operator-based approach — it is reference only and does not reflect the current deployment.

## ArgoCD Sync Order

Sync-wave annotation controls order:
1. Wave 0: Traefik (must be first — other apps depend on ingress)
2. Wave 1+: Prometheus, Grafana, Jaeger, Loki, OTel Collector

## Grafana Data Sources

Auto-provisioned via Grafana Helm values:
- Prometheus: `http://prometheus-server.observability.svc.cluster.local:80`
- Loki: `http://loki.observability.svc.cluster.local:3100`

Jaeger traces are queried via Grafana's Jaeger data source (manual or provisioned separately):
- Jaeger: `http://jaeger.observability.svc.cluster.local:16686`
