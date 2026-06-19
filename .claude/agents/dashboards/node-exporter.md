---
name: dashboard-node-exporter
description: Manage node-exporter.json dashboard — update, replace, handle DS_PROMETHEUS variable quirk
model: claude-sonnet-4-6
tools:
  - Bash
  - Read
  - Write
scope: observability
tags:
  - dashboard
  - metrics
  - infra
depends-on:
  - grafana
  - exporters
  - victoriametrics
---

# Agent Briefing — Dashboard: Node Exporter Full

## Identity
- **File:** `argocd-apps/grafana/chart/dashboards/node-exporter.json`
- **Title:** Node Exporter Full
- **uid:** `rYdddlPWk`
- **Grafana values.yaml key:** `node-exporter`
- **Datasource:** `VictoriaMetrics` (prometheus-compatible)
- **Tags:** `linux`

## What it shows (31 panels)
Comprehensive host-level metrics per node:
- **Quick CPU / Mem / Disk:** stat panels — CPU busy %, sys load 5m/15m, RAM used, SWAP used, root FS used, CPU cores, uptime, totals
- **Basic CPU / Mem / Net / Disk:** time series for CPU, memory, network traffic, disk space
- **CPU / Memory / Net / Disk (detailed):**
  - Memory Meminfo, Vmstat
  - System Timesync, Processes, Misc
  - Hardware Misc, Systemd
  - Storage Disk, Storage Filesystem
  - Network Traffic, Sockstat, Netstat
- **Node Exporter:** scrape health panel

## Template variables
| Variable | Type | Purpose |
|---|---|---|
| `DS_PROMETHEUS` | datasource (prometheus) | Selects VictoriaMetrics — **note: named `DS_PROMETHEUS`, not `datasource`** |
| `job` | query | Prometheus job name for node-exporter scrape |
| `node` | query | Filter by node (instance label) |
| `diskdevices` | custom | Disk device regex filter (e.g. `sda|sdb`) |

## Metrics required
Sourced exclusively from **node-exporter** (scraped by OTel Collector):
- `node_cpu_seconds_total`
- `node_memory_*` (MemTotal, MemAvailable, etc.)
- `node_filesystem_*`
- `node_disk_*`
- `node_network_*`
- `node_load1`, `node_load5`, `node_load15`
- `node_time_seconds`, `node_boot_time_seconds`

## Update this dashboard
```bash
# Download newer version from grafana.com (official Node Exporter Full dashboard)
curl -sL "https://grafana.com/api/dashboards/1860/revisions/latest/download" -o /tmp/node-exporter-new.json

# Strip __requires / __elements
python3 -c "
import json
with open('/tmp/node-exporter-new.json') as f: d = json.load(f)
d.pop('__requires', None); d.pop('__elements', None)
with open('argocd-apps/grafana/chart/dashboards/node-exporter.json', 'w') as f:
    json.dump(d, f, indent=2)
print('uid:', d.get('uid'), 'title:', d.get('title'))
"

# IMPORTANT: verify the datasource variable name — this dashboard uses DS_PROMETHEUS, not datasource
# Grafana Helm chart auto-replaces DS_PROMETHEUS with the default datasource UID
python3 -c "
import json
with open('argocd-apps/grafana/chart/dashboards/node-exporter.json') as f: d = json.load(f)
for t in d.get('templating',{}).get('list',[]):
    if t.get('type') == 'datasource':
        print('datasource var:', t.get('name'), '— query:', t.get('query'))
"
```

After editing: commit, push — ArgoCD auto-syncs Grafana.

## Known quirks
- The datasource template variable is named `DS_PROMETHEUS` (not `datasource`). The Grafana Helm chart handles `DS_PROMETHEUS` → default datasource substitution automatically via `sidecar.dashboards.multicluster` or the provisioner's `inputs` mechanism. Do not rename this variable.
- `diskdevices` variable defaults to a regex — adjust in Grafana UI or patch the JSON if your nodes use `nvme` devices instead of `sd*`.
- Node dropdown populates from the `instance` label on `node_*` metrics — will be empty if node-exporter pods are not running.
