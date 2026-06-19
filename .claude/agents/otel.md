---
name: otel
description: Manage OpenTelemetry Collector — pipelines, scrape targets, reconfigure, troubleshoot
model: claude-sonnet-4-6
tools:
  - Bash
  - Read
  - Edit
  - Skill
scope: observability
tags:
  - metrics
  - traces
  - logs
  - infra
depends-on:
  - victoriametrics
  - jaeger
  - loki
  - exporters
---

# Agent Briefing — OpenTelemetry Collector

## Role in stack
Single ingestion layer for all three pillars. It:
- **Scrapes** infrastructure metrics (node-exporter, cAdvisor/kubelet) via `prometheus` receiver
- **Watches** Kubernetes object state via `k8s_cluster` receiver (replaces kube-state-metrics)
- **Watches** Kubernetes API for cluster events via `k8s_events` receiver (all namespaces)
- **Receives** OTLP from applications on `:4317` (gRPC) and `:4318` (HTTP)
- **Exports** metrics → VictoriaMetrics (`prometheusremotewrite`), traces → Jaeger (OTLP), logs → Loki (`otlphttp/loki`)

In-cluster endpoint for apps: `http://opentelemetry-collector.observability.svc.cluster.local:4317`

No OTel Operator is used — this is a plain `opentelemetry-collector-contrib` Deployment.

### Pipelines
| Name | Receivers | Exporters |
|---|---|---|
| `metrics` | `otlp`, `prometheus`, `k8s_cluster` | `prometheusremotewrite`, `debug` |
| `traces` | `otlp` | `otlp/jaeger`, `debug` |
| `logs` | `otlp` | `otlphttp/loki`, `debug` |
| `logs/k8sevents` | `k8s_events` | `otlphttp/loki` |

K8s events arrive in Loki with stream label `service_name=k8sevents`. Query them: `{service_name="k8sevents"}`.

RBAC: the OTel ClusterRole must include `events` under resources (already set in `values/values.yaml`).

## Key files
```
argocd-apps/opentelemetry-collector/
  app.yaml
  values/values.yaml    # Contains the full collector config (receivers, processors, exporters, pipelines)
  chart/                # Vendored open-telemetry/opentelemetry-collector chart
```

## Skills

| When | Skill | Why |
|---|---|---|
| After editing `values/values.yaml` | `validate` | Dry-run all manifests before committing |
| After ArgoCD syncs | `stack-status` | Confirm collector pod is Running |
| After ArgoCD syncs | `verify-otel` | Confirm all three pipelines (metrics → VictoriaMetrics, traces → Jaeger, logs → Loki) are flowing |

## Status and logs
```bash
kubectl get application opentelemetry-collector -n argocd -o wide
kubectl get pods -n observability -l app.kubernetes.io/name=opentelemetry-collector
kubectl logs -n observability deployment/opentelemetry-collector --tail=100
```

Look for:
- `Everything is ready` — collector started successfully
- `receiver started` lines for each configured receiver
- No `ERROR` lines in exporters (especially remote_write and Loki exporter)

## Reconfigure
The full collector config lives in `values/values.yaml` under the `config:` key (inline YAML). Edit that file, commit, push — ArgoCD auto-syncs.

Common changes:
- Add a new scrape target: add a job under `receivers.prometheus.config.scrape_configs`
- Adjust batch processor: `processors.batch.timeout` / `send_batch_size`
- Change remote_write endpoint: `exporters.prometheusremotewrite.endpoint`
- Add a new OTLP pipeline (e.g., add another exporter)

## Force sync
```bash
kubectl annotate application opentelemetry-collector -n argocd argocd.argoproj.io/refresh=normal --overwrite
```

## Verify all three pipelines
```bash
# Check receiver stats (metrics accepted)
kubectl port-forward -n observability svc/victoria-metrics 8428:8428 &
PF=$!; sleep 3
curl -s "http://localhost:8428/api/v1/query?query=otelcol_receiver_accepted_metric_points_total" | \
  python3 -c "import sys,json; d=json.load(sys.stdin); print('metrics pipeline:', 'PASS' if d['data']['result'] else 'FAIL — no data received')"
kill $PF 2>/dev/null
```
