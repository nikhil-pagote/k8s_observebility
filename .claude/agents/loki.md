---
name: loki
description: Manage Loki — query logs, check ingestion, reconfigure retention and storage
model: claude-sonnet-4-6
tools:
  - Bash
  - Read
  - Edit
  - Skill
scope: observability
tags:
  - logs
  - storage
depends-on:
  - otel
---

# Agent Briefing — Loki

## Role in stack
Log aggregation backend. Receives logs from the OTel Collector via the Loki exporter (OTLP logs pipeline). Grafana queries it via the `Loki` datasource.

Deployed as **single-binary** with **filesystem storage** (not distributed mode, no object storage — this is a POC).

Grafana datasource name: `Loki` (type: loki, `http://loki.observability.svc.cluster.local:3100`)

**Not** using a filelog DaemonSet — logs arrive exclusively via OTel Collector OTLP push.

## Key files
```
argocd-apps/loki/
  app.yaml
  values/values.yaml
  chart/                # Vendored grafana/loki chart
```

## Skills

| When | Skill | Why |
|---|---|---|
| After editing `values/values.yaml` | `validate` | Dry-run all manifests before committing |
| After ArgoCD syncs | `stack-status` | Confirm Loki pod is Running and ArgoCD shows Synced |

## Status and logs
```bash
kubectl get application loki -n argocd -o wide
kubectl get pods -n observability -l app.kubernetes.io/name=loki
kubectl logs -n observability deployment/loki --tail=50
```

## Log streams

| Stream | Source | Labels |
|---|---|---|
| `{service_name="k8sevents"}` | OTel `k8s_events` receiver (all K8s events) | `k8s_namespace_name`, `detected_level` |
| `{job="..."}` | App OTLP push via OTel Collector | varies by app |

## Query logs
```bash
kubectl port-forward -n observability svc/loki 3100:3100 &
PF=$!; sleep 3

# List labels (proves logs are being ingested)
curl -s "http://localhost:3100/loki/api/v1/labels" | \
  python3 -c "import sys,json; d=json.load(sys.stdin); print('labels:', d.get('data',[]))"

# Verify K8s events are flowing
curl -sG "http://localhost:3100/loki/api/v1/series" \
  --data-urlencode 'match[]={service_name="k8sevents"}' | \
  python3 -c "import sys,json; d=json.load(sys.stdin); print('k8s event streams:', len(d.get('data',[])))"

# Query recent logs
curl -sG "http://localhost:3100/loki/api/v1/query_range" \
  --data-urlencode 'query={job=~".+"}' \
  --data-urlencode "start=$(date -d '5 minutes ago' +%s)000000000" \
  --data-urlencode "end=$(date +%s)000000000" \
  --data-urlencode "limit=10" | \
  python3 -c "
import sys, json
d = json.load(sys.stdin)
for stream in d.get('data',{}).get('result',[]):
    for ts, line in stream.get('values',[]):
        print(line[:120])
"

kill $PF 2>/dev/null
```

## Reconfigure
Edit `argocd-apps/loki/values/values.yaml`, commit, push — ArgoCD auto-syncs.

Common changes:
- Retention: `loki.limits_config.retention_period` (e.g., `168h` for 7 days)
- Storage size: `singleBinary.persistence.size`
- Ingestion rate limits: `loki.limits_config.ingestion_rate_mb` / `ingestion_burst_size_mb`

## Force sync
```bash
kubectl annotate application loki -n argocd argocd.argoproj.io/refresh=normal --overwrite
```
