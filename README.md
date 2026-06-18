# Kubernetes Observability Stack

A Kubernetes observability POC using **OpenTelemetry** as the unified collection layer, deployed via GitOps (ArgoCD), and exposed through a single Traefik ingress on path-based routing.

## Architecture

### Three pillars

| Pillar | Tool | Role |
|---|---|---|
| **Metrics** | VictoriaMetrics + Grafana | Store and visualize metrics (OTel pushes via remote_write) |
| **Traces** | Jaeger | Distributed trace storage and query UI |
| **Logs** | Loki | Log aggregation and querying |

### Data flow

```
node-exporter       :9100 ─┐
kube-state-metrics  :8080 ─┤
pushgateway         :9091 ─┤  OTel prometheus receiver (pull)
cAdvisor (kubelet)        ─┤
annotated pods/endpoints  ─┘
                           │
App (OTLP :4317/4318) ─────┤  OTel Collector (single ingestion layer)
                           │
             ┌─────────────┼──────────────┐
             ▼             ▼              ▼
     VictoriaMetrics     Jaeger         Loki
     (remote_write)   (OTLP traces)  (OTLP logs)
             │             │              │
             └─────────────┴──────────────┘
                           ▼
                        Grafana
```

### Components

| Component | Namespace | Purpose |
|---|---|---|
| **Traefik** | traefik | Path-based ingress on NodePort 30080 |
| **OTel Collector** | observability | Single ingestion layer — scrapes infra metrics, receives OTLP from apps |
| **VictoriaMetrics** | observability | Metrics storage backend, receives remote_write, serves PromQL |
| **node-exporter** | observability | Host metrics (CPU, memory, disk, network) — DaemonSet |
| **kube-state-metrics** | observability | Kubernetes object metrics (pod status, replicas, resource limits) |
| **Pushgateway** | observability | Accepts pushed metrics from batch jobs and short-lived processes |
| **Loki** | observability | Log storage (single-binary, filesystem) |
| **Jaeger** | observability | Distributed trace storage and query UI |
| **Grafana** | observability | Unified dashboards — correlates metrics, traces, and logs |
| **ArgoCD** | argocd | GitOps reconciler for all stack components |

### Ingress URL map

| Path | Service | Notes |
|---|---|---|
| `/grafana` | Grafana | admin / admin123 |
| `/vmui` | VictoriaMetrics UI | ad-hoc PromQL queries |
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
| VictoriaMetrics | http://localhost:30080/vmui | — |
| Jaeger | http://localhost:30080/jaeger | — |
| Traefik | http://localhost:30080/traefik | — |
| ArgoCD | http://localhost:30080/argocd | admin / `kubectl get secret argocd-initial-admin-secret -n argocd -o jsonpath='{.data.password}' \| base64 -d` |

---

## Instrumentation

### App telemetry (metrics, traces, logs)

Send all telemetry to the OTel Collector via OTLP:

```yaml
env:
  - name: OTEL_EXPORTER_OTLP_ENDPOINT
    value: "http://opentelemetry-collector.observability.svc.cluster.local:4317"
  - name: OTEL_SERVICE_NAME
    value: "my-service"
```

### Batch job metrics (Pushgateway)

For jobs that exit before they can be scraped:

```bash
# Push a metric from a shell script
echo "job_duration_seconds 42" | curl --data-binary @- \
  http://pushgateway.observability.svc.cluster.local:9091/metrics/job/my-batch-job
```

---

## Grafana Data Sources

Auto-provisioned via Helm values:
- **VictoriaMetrics** (Prometheus-compatible): `http://victoria-metrics.observability.svc.cluster.local:8428`
- **Loki**: `http://loki.observability.svc.cluster.local:3100`

### Dashboards (auto-provisioned)

| Dashboard | Grafana ID | Source |
|---|---|---|
| K8S Dashboard (global cluster view) | 15661 rev 2 | vendored JSON — `chart/dashboards/` |
| Kubernetes Traefik Ingress NextGen | 25330 rev 1 | downloaded via gnetId at startup |
| Node Exporter Full | 1860 rev 22 | downloaded via gnetId at startup |
| VictoriaMetrics single-node | 10229 rev 35 | downloaded via gnetId at startup |

The K8S Dashboard JSON is pre-patched and committed to the repo (`argocd-apps/grafana/chart/dashboards/kubernetes-views-global.json`) so it works with VictoriaMetrics as the datasource without any runtime network dependency.

---

## Troubleshooting

| Symptom | Likely Cause | Fix |
|---|---|---|
| `/grafana` returns 404 | IngressRoute not picked up | `kubectl describe ingressroute -n observability`; check Traefik logs |
| Grafana CSS/JS broken | `serve_from_sub_path` not set | Check `serve_from_sub_path=true` and `root_url` in Grafana values |
| Jaeger UI: Unknown path | `base-path` not set | Check `userconfig.extensions.jaeger_query.base_path: /jaeger` in values |
| Collector `CrashLoopBackOff` | Bad config YAML or wrong image | `kubectl logs deployment/opentelemetry-collector -n observability` |
| No metrics in VictoriaMetrics | remote_write failing | Check OTel logs for `prometheusremotewrite` errors; verify VM pod running |
| No logs in Loki | Collector can't reach Loki | Check collector logs; verify Loki pod running and OTLP endpoint |
| No traces in Jaeger | Collector can't reach Jaeger | Verify `jaeger` service in observability ns; check `insecure: true` on exporter |

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
| VictoriaMetrics | 1 GB | 1 core |
| Grafana | 512 MB | 500m |
| Jaeger | 512 MB | 500m |
| Loki | 512 MB | 500m |
| OTel Collector | 512 MB | 500m |
| node-exporter | 128 MB | 250m |
| kube-state-metrics | 256 MB | 250m |
| Pushgateway | 128 MB | 250m |

---

## Production Considerations

This stack is a **development/POC** setup. For production:

- **Jaeger storage**: Switch from in-memory to a persistent backend (e.g., Elasticsearch, Cassandra)
- **Loki**: Switch from filesystem to object storage (S3, GCS); deploy in distributed mode
- **VictoriaMetrics**: Deploy as a cluster for high availability and horizontal scaling
- **Alertmanager**: Add for alert routing to PagerDuty, Slack, email etc.
- **Security**: Enable TLS, RBAC, and network policies throughout
- **High availability**: Multiple replicas for all components
