---
name: jaeger
description: Manage Jaeger — query traces, check services, reconfigure storage
model: claude-sonnet-4-6
tools:
  - Bash
  - Read
  - Edit
  - Skill
scope: observability
tags:
  - traces
  - storage
depends-on:
  - otel
---

# Agent Briefing — Jaeger

## Role in stack
Distributed trace storage and UI. Receives traces from the OTel Collector via OTLP exporter.

Deployed with **in-memory storage** (POC — traces are lost on pod restart). UI at `http://localhost:30080/jaeger`.

## Key files
```
argocd-apps/jaeger/
  app.yaml
  values/values.yaml
  chart/                # Vendored jaegertracing/jaeger chart
```

## Skills

| When | Skill | Why |
|---|---|---|
| After editing `values/values.yaml` | `validate` | Dry-run all manifests before committing |
| After ArgoCD syncs | `stack-status` | Confirm Jaeger pod is Running and ArgoCD shows Synced |

## Status and logs
```bash
kubectl get application jaeger -n argocd -o wide
kubectl get pods -n observability -l app.kubernetes.io/name=jaeger
kubectl logs -n observability deployment/jaeger --tail=50
```

## Query traces
```bash
kubectl port-forward -n observability svc/jaeger 16686:16686 &
PF=$!; sleep 3

# List services that have sent traces
curl -s "http://localhost:16686/jaeger/api/services" | \
  python3 -c "import sys,json; d=json.load(sys.stdin); print('services:', d.get('data',[]))"

kill $PF 2>/dev/null
```

## Reconfigure
Edit `argocd-apps/jaeger/values/values.yaml`, commit, push — ArgoCD auto-syncs.

Common changes:
- Switch from in-memory to Badger (persistent): set storage type in values
- `resources` — CPU/memory limits
- Sampling strategies

## Force sync
```bash
kubectl annotate application jaeger -n argocd argocd.argoproj.io/refresh=normal --overwrite
```
