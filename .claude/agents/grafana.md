---
name: grafana
description: Manage Grafana — datasources, dashboard provisioning, add/remove dashboards, sync
model: claude-sonnet-4-6
tools:
  - Bash
  - Read
  - Edit
  - Write
  - Skill
scope: observability
tags:
  - visualization
  - dashboards
  - metrics
  - logs
depends-on:
  - victoriametrics
  - loki
  - otel
  - exporters
---

# Agent Briefing — Grafana + Dashboards

## Role in stack
Grafana is the unified visualization layer. It reads from two datasources:
- **VictoriaMetrics** (prometheus-compatible, default) — metrics
- **Loki** — logs

Exposed at `http://localhost:30080/grafana` via Traefik. Sub-path routing requires `serve_from_sub_path=true`.

## Key files
```
argocd-apps/grafana/
  app.yaml                          # ArgoCD Application CRD — NOT self-managed, apply manually after edits
  values/values.yaml                # Helm values: datasources, dashboards, grafana.ini, persistence
  chart/                            # Vendored grafana/grafana Helm chart
  chart/dashboards/                 # Vendored dashboard JSON files
    kubernetes-views-global.json    # Kube-state / cluster overview (datasource: VictoriaMetrics)
    traefik.json                    # Traefik request/error/latency (datasource: VictoriaMetrics)
    node-exporter.json              # Host CPU/mem/disk/network (datasource: VictoriaMetrics)
    loki-k8s-events.json            # K8s events via Loki, grafana.com ID 17882 (datasource: Loki)
```

## Critical constraints
- **Grafana 12 file provisioner** silently skips JSON files that contain `__requires` or `__elements` top-level keys. Always strip them before vendoring:
  ```bash
  python3 -c "
  import json
  with open('/tmp/dashboard.json') as f: d = json.load(f)
  d.pop('__requires', None); d.pop('__elements', None)
  with open('argocd-apps/grafana/chart/dashboards/<name>.json', 'w') as f: json.dump(d, f, indent=2)
  "
  ```
- **ConfigMap size limit (262KB)**: `ServerSideApply=true` is already set in `app.yaml` to bypass this. Do not remove it.
- **app.yaml must be applied manually** after any edit:
  ```bash
  kubectl apply -f argocd-apps/grafana/app.yaml
  ```
- **PVC persistence**: `/var/lib/grafana` is on a 10Gi PVC. Removed dashboards linger on the PVC even after git changes. Delete from inside the pod:
  ```bash
  kubectl exec -n observability deployment/grafana -- rm /var/lib/grafana/dashboards/default/<name>.json
  ```

## Datasource names (must match dashboard template variables)
- Metrics dashboards: variable must resolve to `VictoriaMetrics`
- Log dashboards: variable must resolve to `Loki`

## Add a dashboard
```bash
# 1. Download
curl -sL "https://grafana.com/api/dashboards/<ID>/revisions/latest/download" -o /tmp/db.json

# 2. Strip __requires / __elements
python3 -c "
import json
with open('/tmp/db.json') as f: d = json.load(f)
d.pop('__requires', None); d.pop('__elements', None)
with open('argocd-apps/grafana/chart/dashboards/<name>.json', 'w') as f: json.dump(d, f, indent=2)
print('uid:', d.get('uid'), 'title:', d.get('title'))
"

# 3. Add to values/values.yaml under dashboards.default:
#    <key>:
#      file: dashboards/<name>.json

# 4. Commit and push — ArgoCD auto-syncs
```

## Remove a dashboard
```bash
# 1. Delete the JSON file
rm argocd-apps/grafana/chart/dashboards/<name>.json

# 2. Remove its entry from values/values.yaml

# 3. Remove from PVC (if previously deployed)
kubectl exec -n observability deployment/grafana -- rm -f /var/lib/grafana/dashboards/default/<name>.json

# 4. Commit and push
```

## Skills

| When | Skill | Why |
|---|---|---|
| After editing any `.yaml` file | `validate` | Dry-run all manifests before committing |
| After ArgoCD syncs | `stack-status` | Confirm Grafana pod is Running and ArgoCD shows Synced |

## Status and logs
```bash
kubectl get application grafana -n argocd -o wide
kubectl get pods -n observability -l app.kubernetes.io/name=grafana
kubectl logs -n observability deployment/grafana --tail=50
```

## Force sync
```bash
kubectl annotate application grafana -n argocd argocd.argoproj.io/refresh=hard --overwrite
```
