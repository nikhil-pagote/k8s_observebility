# Architecture Spec — k8s Observability Stack

## Overview

A local Kubernetes observability POC using **OpenTelemetry** as the unified collection layer, deployed via GitOps (ArgoCD), and exposed through a single Traefik ingress on path-based routing.

## Three Pillars

| Pillar | Tool | Role |
|---|---|---|
| Metrics | VictoriaMetrics + Grafana | Store and visualize metrics (OTel pushes via remote_write) |
| Traces | Jaeger | Distributed trace storage and UI |
| Logs | Loki | Log aggregation and querying |

## Data Flow

```
node-exporter  ──────────────────────────────┐
kube-state-metrics ──────────────────────────┤ OTel prometheus receiver (pull)
cAdvisor (kubelet) ──────────────────────────┤
annotated pods/endpoints ────────────────────┘
                                             │
App (OTLP :4317/4318) ───────────────────────┤ OTel Collector
                                             │
                              ┌──────────────┼──────────────┐
                              ▼              ▼              ▼
                      VictoriaMetrics     Jaeger          Loki
                      (remote_write)    (OTLP traces)  (OTLP logs)
                              │              │              │
                              └──────────────┴──────────────┘
                                             ▼
                                          Grafana
```

## Component Map

| Component | Namespace | Helm Chart | Purpose |
|---|---|---|---|
| Traefik | traefik | traefik/traefik | Ingress, NodePort 30080/30443 |
| ArgoCD | argocd | argo/argo-cd | GitOps reconciler |
| VictoriaMetrics | observability | victoriametrics/victoria-metrics-single | Metrics storage backend (receives remote_write) |
| Grafana | observability | grafana/grafana | Unified visualization |
| Jaeger | observability | jaegertracing/jaeger | Trace store (in-memory for POC) |
| Loki | observability | grafana/loki | Log store (single-binary, filesystem) |
| OTel Collector | observability | open-telemetry/opentelemetry-collector | Single ingestion layer — scrapes infra, receives OTLP from apps |
| node-exporter | observability | prometheus-community/prometheus-node-exporter | Host metrics (CPU, mem, disk, network) — DaemonSet |
| kube-state-metrics | observability | prometheus-community/kube-state-metrics | Kubernetes object metrics |
| Pushgateway | observability | prometheus-community/prometheus-pushgateway | Accepts pushed metrics from batch/short-lived jobs |

## Ingress Routing

All UIs via Traefik at `http://localhost:30080`:

| Path | Service | Port | Notes |
|---|---|---|---|
| `/grafana` | grafana | 80 | sub-path routing via `serve_from_sub_path=true` |
| `/vmui` | victoria-metrics-server | 8428 | no prefix strip — VM serves vmui natively at /vmui |
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
2. Wave 1+: VictoriaMetrics, Grafana, Jaeger, Loki, OTel Collector, node-exporter, kube-state-metrics

## Grafana Data Sources

Auto-provisioned via Grafana Helm values:
- VictoriaMetrics (as Prometheus type): `http://victoria-metrics-server.observability.svc.cluster.local:8428`
- Loki: `http://loki.observability.svc.cluster.local:3100`

Jaeger traces are queried via Grafana's Jaeger data source (add manually):
- Jaeger: `http://jaeger.observability.svc.cluster.local:16686`
