---
name: exporters
description: Manage node-exporter — status, reconfigure, sync, verify metrics
model: claude-sonnet-4-6
tools:
  - Bash
  - Read
  - Edit
  - Skill
scope: observability
tags:
  - metrics
  - infra
  - prometheus
depends-on:
  - otel
  - victoriametrics
---

# Agent Briefing — Prometheus Exporters

## Role in stack
One exporter feeds host-level infrastructure metrics into the OTel Collector's prometheus scrape receiver, which pushes to VictoriaMetrics via remote_write.

| Exporter | Workload | What it exports |
|---|---|---|
| **node-exporter** | DaemonSet (one pod per node) | Host CPU, memory, disk I/O, network, kernel stats |

**Not present in this stack:**
- `kube-state-metrics` — replaced by OTel `k8s_cluster` receiver (emits `k8s_*` metrics)
- `pushgateway` — removed; batch jobs should push via OTLP to the collector on `:4317`

## Key files
```
argocd-apps/node-exporter/
  app.yaml
  values/values.yaml
  chart/
```

Deploys into the `observability` namespace.

## Skills

| When | Skill | Why |
|---|---|---|
| After editing `values/values.yaml` | `validate` | Dry-run all manifests before committing |
| After ArgoCD syncs | `stack-status` | Confirm node-exporter DaemonSet pods are Running on every node |

## Status
```bash
kubectl get applications -n argocd node-exporter \
  -o custom-columns='NAME:.metadata.name,SYNC:.status.sync.status,HEALTH:.status.health.status'

# Must be Running on every node (1 control-plane + 3 workers = 4 pods)
kubectl get pods -n observability -l app.kubernetes.io/name=node-exporter -o wide
kubectl get ds -n observability
```

## Logs
```bash
kubectl logs -n observability $(kubectl get pods -n observability -l app.kubernetes.io/name=node-exporter -o name | head -1) --tail=50
```

## Reconfigure
Edit `argocd-apps/node-exporter/values/values.yaml`, commit and push. ArgoCD auto-syncs.

Common changes:
- `resources` — CPU/memory limits
- `extraArgs` — enable/disable specific collectors (e.g., `--collector.diskstats`)

## Force sync
```bash
kubectl annotate application node-exporter -n argocd argocd.argoproj.io/refresh=normal --overwrite
```

## Verify metrics in VictoriaMetrics
```bash
kubectl port-forward -n observability svc/victoria-metrics 8428:8428 &
PF=$!; sleep 3

curl -s "http://localhost:8428/api/v1/query?query=node_cpu_seconds_total" | \
  python3 -c "import sys,json; d=json.load(sys.stdin); print('node-exporter:', 'PASS' if d['data']['result'] else 'FAIL')"

# Verify k8s_cluster receiver (replaces kube-state-metrics)
curl -s "http://localhost:8428/api/v1/query?query=k8s_pod_phase" | \
  python3 -c "import sys,json; d=json.load(sys.stdin); print('k8s_cluster:', 'PASS' if d['data']['result'] else 'FAIL')"

kill $PF 2>/dev/null
```
