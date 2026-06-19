---
name: dashboard-kube-state
description: Manage kubernetes-views-global.json — update, replace, fix datasource variable
model: claude-sonnet-4-6
tools:
  - Bash
  - Read
  - Write
scope: observability
tags:
  - dashboard
  - metrics
  - kubernetes
depends-on:
  - grafana
  - exporters
  - victoriametrics
---

# Agent Briefing — Dashboard: Kubernetes Views Global (kube-state)

## Identity
- **File:** `argocd-apps/grafana/chart/dashboards/kubernetes-views-global.json`
- **Title:** K8S Dashboard
- **uid:** _(none set)_
- **Grafana values.yaml key:** `kubernetes-views-global`
- **Datasource:** `VictoriaMetrics` (prometheus-compatible)
- **Tags:** `Prometheus`, `Kubernetes`

## What it shows (29 panels)
Cluster-wide Kubernetes resource overview across nodes, namespaces, pods, and containers:
- Node resource overview — CPU ratio, memory ratio, storage, core count
- Namespace resource statistics — CPU/memory usage per namespace
- Network overview — per namespace and node
- Pod resource detail — CPU, memory (WSS/RSS), network bandwidth
- PVC storage usage
- Container-level CPU/memory breakdown
- Microservices (container name) resource overview

## Template variables
| Variable | Type | Purpose |
|---|---|---|
| `datasource` | datasource (prometheus) | Selects VictoriaMetrics |
| `origin_prometheus` | query | Prometheus job filter |
| `Node` | query | Filter by node name |
| `NameSpace` | query | Filter by namespace |
| `Container` | query | Filter by container name |
| `Pod` | query | Filter by pod name |

## Metrics required
Sourced from **kube-state-metrics** and **node-exporter** (both scraped by OTel Collector):
- `kube_pod_info`, `kube_node_status_allocatable`, `kube_namespace_*`
- `node_cpu_seconds_total`, `node_memory_*`, `node_filesystem_*`, `node_network_*`
- `container_cpu_usage_seconds_total`, `container_memory_working_set_bytes` (from cAdvisor/kubelet)

## Update this dashboard
To replace with a newer version from grafana.com or a local edit:
```bash
# Download new version (if from grafana.com)
curl -sL "https://grafana.com/api/dashboards/<ID>/revisions/latest/download" -o /tmp/new.json

# Strip __requires / __elements (required for Grafana 12 file provisioner)
python3 -c "
import json
with open('/tmp/new.json') as f: d = json.load(f)
d.pop('__requires', None); d.pop('__elements', None)
with open('argocd-apps/grafana/chart/dashboards/kubernetes-views-global.json', 'w') as f:
    json.dump(d, f, indent=2)
print('uid:', d.get('uid'), 'title:', d.get('title'))
"

# Verify datasource variable resolves to 'VictoriaMetrics'
python3 -c "
import json
with open('argocd-apps/grafana/chart/dashboards/kubernetes-views-global.json') as f: d = json.load(f)
for t in d.get('templating',{}).get('list',[]):
    if t.get('type') == 'datasource':
        print('datasource var:', t.get('name'), '— query:', t.get('query'))
"
```

After editing: commit, push — ArgoCD auto-syncs Grafana.

## Known quirks
- No uid set in the JSON — Grafana assigns one on first load. Avoid setting a conflicting uid if re-vendoring.
- The dashboard title uses unusual Unicode characters (e.g., `：`) — do not alter the title unless replacing the whole dashboard.
- Namespace and node dropdowns populate from live metric labels; they will be empty if kube-state-metrics or node-exporter is down.
