# Kubernetes Observability Stack

A Kubernetes observability POC using **OpenTelemetry** as the unified collection layer, deployed via GitOps (ArgoCD), and exposed through a single Traefik ingress on path-based routing.

## Architecture

### Three pillars

| Pillar | Tool | Role |
|---|---|---|
| **Metrics** | Prometheus + Grafana | Collect, store, and visualize metrics |
| **Traces** | Jaeger | Distributed trace storage and query UI |
| **Logs** | Loki | Log aggregation and querying |

### Data flow

```
App (OTLP :4317/4318)
        │
        ▼
OTel Collector (Deployment)
        │
   ┌────┴──────┐
   ▼           ▼           ▼
Prometheus  Jaeger       Loki
   │           │           │
   └─────┬─────┘           │
         ▼                 │
      Grafana ◄────────────┘
```

### Components

| Component | Namespace | Purpose |
|---|---|---|
| **Traefik** | traefik | Path-based ingress on NodePort 30080 |
| **OTel Collector** | observability | Receives OTLP; fans out metrics → Prometheus, traces → Jaeger, logs → Loki |
| **Prometheus** | observability | Scrapes and stores metrics |
| **Loki** | observability | Stores log data (single-binary, filesystem) |
| **Jaeger** | observability | Stores distributed traces; trace search and dependency graphs |
| **Grafana** | observability | Unified dashboards — correlates traces ↔ logs ↔ metrics |
| **ArgoCD** | argocd | GitOps reconciler for all stack components |

### Ingress URL map

| Path | Service | Notes |
|---|---|---|
| `/grafana` | Grafana | admin / admin123 |
| `/prometheus` | Prometheus | — |
| `/jaeger` | Jaeger Query UI | — |
| `/traefik` | Traefik Dashboard | redirects to `/dashboard/` |
| `/argocd` | ArgoCD Server | admin / see below |

---

## Quick Start

### Prerequisites

- **Podman** (rootless) with the user socket running — Docker is not used
- `kubectl`, `helm` v3+, `kind`

```bash
# Verify Podman socket is active
systemctl --user enable --now podman.socket

# Load env vars (or use direnv to auto-load)
source .envrc
```

### 1. Create the Kind cluster

```bash
/kind-cluster start
# Creates: 1 control-plane + 3 workers
```

### 2. Deploy the stack

```bash
/deploy --step 1   # Install ArgoCD via Helm
/deploy --step 3   # Apply ArgoCD Application manifests
```

ArgoCD reads each app's local `chart/` and `values/values.yaml` and deploys them.

### 3. Access the UIs

| UI | URL | Credentials |
|---|---|---|
| Grafana | http://localhost:30080/grafana | admin / admin123 |
| Prometheus | http://localhost:30080/prometheus | — |
| Jaeger | http://localhost:30080/jaeger | — |
| Traefik | http://localhost:30080/traefik | — |
| ArgoCD | http://localhost:30080/argocd | admin / `kubectl get secret argocd-initial-admin-secret -n argocd -o jsonpath='{.data.password}' \| base64 -d` |

---

## Instrumentation

Send telemetry to the OTel Collector via OTLP:

```yaml
env:
  - name: OTEL_EXPORTER_OTLP_ENDPOINT
    value: "http://opentelemetry-collector.observability.svc.cluster.local:4317"
  - name: OTEL_SERVICE_NAME
    value: "my-service"
```

---

## Grafana Data Sources

Auto-provisioned via Helm values:
- **Prometheus**: `http://prometheus-server.observability.svc.cluster.local:80`
- **Loki**: `http://loki.observability.svc.cluster.local:3100`

### Import dashboards

| Dashboard | ID | Data source |
|---|---|---|
| OTel Collector | 15983 | Prometheus |
| Kubernetes Cluster | 7249 | Prometheus |
| Node Exporter | 1860 | Prometheus |

---

## Troubleshooting

| Symptom | Likely Cause | Fix |
|---|---|---|
| `/grafana` returns 404 | IngressRoute not picked up | `kubectl describe ingressroute -n observability`; check Traefik logs |
| Grafana CSS/JS broken | `serve_from_sub_path` not set | Helm upgrade with `serve_from_sub_path=true` and correct `root_url` |
| Jaeger UI: Unknown path | `base-path` not set | Check `jaeger.args: [--query.base-path=/jaeger]` in values |
| Collector `CrashLoopBackOff` | Bad config YAML or wrong image | `kubectl logs deployment/opentelemetry-collector -n observability` |
| No logs in Loki | Collector can't reach Loki | Check collector logs; verify Loki pod running and OTLP endpoint |
| No traces in Jaeger | Collector can't reach Jaeger | Verify `jaeger-collector` service; check `insecure: true` on exporter |

### Useful commands

```bash
kubectl get pods --all-namespaces
kubectl get applications -n argocd
kubectl logs -n observability deployment/opentelemetry-collector -f
kubectl get ingressroute -A
```

---

## Resource Requirements

### Minimum

- CPU: 4 cores
- Memory: 8 GB RAM
- Storage: 20 GB

### Per component

| Component | Memory | CPU |
|---|---|---|
| Grafana | 512 MB | 500m |
| Prometheus | 2 GB | 1 core |
| Jaeger | 512 MB | 500m |
| Loki | 512 MB | 500m |
| OTel Collector | 512 MB | 500m |

---

## Production Considerations

This stack is a **development/POC** setup. For production:

- **Jaeger storage**: Switch from in-memory to a persistent backend (e.g., Elasticsearch, Cassandra)
- **Loki**: Switch from filesystem to object storage (S3, GCS); deploy in distributed mode
- **Prometheus**: Add persistent storage (PVC with `storageSpec`)
- **Security**: Enable TLS, RBAC, and network policies throughout
- **High availability**: Multiple replicas for all components
