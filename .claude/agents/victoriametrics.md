---
name: victoriametrics
description: Manage VictoriaMetrics — query metrics, check scrape targets, reconfigure retention and resources
model: claude-sonnet-4-6
tools:
  - Bash
  - Read
  - Edit
  - Skill
scope: observability
tags:
  - metrics
  - storage
depends-on:
  - otel
---

# Agent Briefing — VictoriaMetrics

## Role in stack
Metrics storage backend. Receives all metrics from the OTel Collector via `prometheusremotewrite` and exposes a Prometheus-compatible query API used by Grafana.

Grafana datasource name: `VictoriaMetrics` (type: prometheus, `http://victoria-metrics.observability.svc.cluster.local:8428`)

UI available at `http://localhost:30080/vmui` (via Traefik, no prefix strip — VictoriaMetrics serves `/vmui` natively).

## Key files
```
argocd-apps/victoria-metrics/
  app.yaml
  values/values.yaml
  chart/                # Vendored victoriametrics/victoria-metrics-single chart
```

## Skills

| When | Skill | Why |
|---|---|---|
| After editing `values/values.yaml` | `validate` | Dry-run all manifests before committing |
| After ArgoCD syncs | `stack-status` | Confirm VictoriaMetrics pod is Running and ArgoCD shows Synced |

## Status and logs
```bash
kubectl get application victoria-metrics -n argocd -o wide
kubectl get pods -n observability -l app.kubernetes.io/name=victoria-metrics-single
kubectl logs -n observability deployment/victoria-metrics --tail=50
```

## Query metrics
```bash
kubectl port-forward -n observability svc/victoria-metrics 8428:8428 &
PF=$!; sleep 3

# List all metric names
curl -s "http://localhost:8428/api/v1/label/__name__/values" | python3 -c "
import sys, json; d = json.load(sys.stdin)
print(f'{len(d[\"data\"])} metrics:', d['data'][:20])
"

# Check scrape targets are up
curl -s "http://localhost:8428/api/v1/query?query=up" | python3 -c "
import sys, json
for r in json.load(sys.stdin)['data']['result']:
    print(r['metric'].get('job','?'), ':', 'UP' if r['value'][1]=='1' else 'DOWN')
"

kill $PF 2>/dev/null
```

## Reconfigure
Edit `argocd-apps/victoria-metrics/values/values.yaml`, commit, push — ArgoCD auto-syncs.

Common changes:
- `retentionPeriod` — how long to keep metrics (default: `1` month)
- `resources` — CPU/memory limits
- `persistentVolume.size` — storage size

## Force sync
```bash
kubectl annotate application victoria-metrics -n argocd argocd.argoproj.io/refresh=normal --overwrite
```
