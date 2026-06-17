# Architecture Spec — k8s Observability Stack

## Overview

A local Kubernetes observability POC using **OpenTelemetry** as the unified collection layer, deployed via GitOps (ArgoCD), and exposed through a single Traefik ingress on path-based routing.

## Three Pillars

| Pillar | Tool | Role |
|---|---|---|
| Metrics | Prometheus + Grafana | Scrape, store, and visualize metrics |
| Traces | Jaeger | Distributed trace storage and UI |
| Logs | ClickHouse | High-performance log storage and querying |

## Data Flow

```
Applications (instrumented)
        │
        ▼ OTLP gRPC :4317
OpenTelemetry Collector (Deployment)
        │
   ┌────┼────┐
   ▼    ▼    ▼
Prometheus Jaeger ClickHouse
   │         │         │
   └─────────┴─────────┘
                │
             Grafana (unified dashboards)
```

OTel Collector also runs as a **DaemonSet** (one pod per node) for log tailing:
```
/var/log/pods (node) → filelog receiver → k8sattributes → ClickHouse
```

## Component Map

| Component | Namespace | Helm Chart | Purpose |
|---|---|---|---|
| Traefik | traefik | traefik/traefik | Ingress, NodePort 30080 |
| ArgoCD | argocd | argo-helm | GitOps reconciler |
| Prometheus | observability | bitnami/prometheus | Metrics store |
| Grafana | observability | bitnami/grafana | Unified visualization |
| Jaeger | observability | jaegertracing/jaeger | Trace store (in-memory for POC) |
| ClickHouse | observability | bitnami/clickhouse | Log store |
| OTel Collector | observability | open-telemetry/opentelemetry-collector | OTLP receiver, fan-out |

## Ingress Routing

All UIs via Traefik at `http://localhost:30080`:

| Path | Service | Port |
|---|---|---|
| `/grafana` | Grafana | 80 |
| `/prometheus` | Prometheus | 9090 |
| `/jaeger` | Jaeger Query | 16686 |
| `/clickhouse` | ClickHouse HTTP | 8123 |
| `/traefik` | Traefik Dashboard | 9000 |
| `/argocd` | ArgoCD Server | 80 |

## Kind Cluster

Name: `observability-cluster`
Kubernetes: v1.33.1
Topology: 1 control-plane + 2 workers
Port mappings: `30080 → :80`, `30443 → :443`

## OTel Operator Target Architecture

The guide (`otel operator for k8s.md`) describes the next evolution: replacing the plain OTel Collector Deployment with the **OTel Operator** managing Collector CRDs. Key additions:

- cert-manager for webhook TLS
- `OpenTelemetryCollector` CR (Deployment mode for metrics+traces)
- `OpenTelemetryCollector` CR (DaemonSet mode for logs → ClickHouse)
- `Instrumentation` CR for auto-injection into app pods
- Jaeger Operator managing a `Jaeger` CR

> Note: The guide references Elasticsearch for logs. **This project uses ClickHouse** for log storage instead.

## ArgoCD Sync Order

Sync-wave annotation controls order:
1. Wave 0: Traefik (must be first — other apps depend on ingress)
2. Wave 1+: Prometheus, Grafana, Jaeger, ClickHouse, OTel Collector

## Grafana Data Sources

Auto-provisioned:
- Prometheus: `http://prometheus-server.observability.svc.cluster.local:80`

Manual (add via UI):
- Jaeger: `http://jaeger-query.observability.svc.cluster.local:16686`
- ClickHouse: `http://clickhouse.observability.svc.cluster.local:8123` (plugin required)
