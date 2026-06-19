---
name: dashboard-loki-k8s-events
description: Manage loki-k8s-events.json (grafana.com ID 17882) — update, re-vendor, strip __requires/__elements
model: claude-sonnet-4-6
tools:
  - Bash
  - Read
  - Write
scope: observability
tags:
  - dashboard
  - logs
  - kubernetes
depends-on:
  - grafana
  - loki
---

# Agent Briefing — Dashboard: Kubernetes Event Exporter (Loki)

## Identity
- **File:** `argocd-apps/grafana/chart/dashboards/loki-k8s-events.json`
- **Title:** Kubernetes Event Exporter
- **uid:** `kubernetes-event-exporter`
- **Grafana values.yaml key:** `loki-k8s-events`
- **Datasource:** `Loki`
- **Tags:** `loki`
- **Original source:** grafana.com dashboard ID 17882 (by devmem, last updated 2023-01-17)
- **GitHub:** https://github.com/resmoio/kubernetes-event-exporter

## What it shows (15 panels)
Kubernetes event monitoring via Loki log queries:

**Details section:**
- `Kubernetes Events - Details` — searchable event table with regex filter
- `Events Details` — raw event log stream

**Stats section:**
- `Overview` — total event counts
- `Warnings` — warning event count
- `Image Pull Failed` — stat panel
- `Liveness Probe Failed` — stat panel
- `Volume Mount Failed` — stat panel
- `Scheduling Failed` — stat panel
- `Container OOM Killed` — stat panel
- `Container Crashed` — stat panel
- `Pod Evicted` — stat panel
- `System OOM` — stat panel

**Distribution section:**
- `Kubernetes Events - Distribution` — pie chart by event type/reason
- `Kubernetes Events - Raw Logs` — unfiltered log panel

## Template variables
| Variable | Type | Purpose |
|---|---|---|
| `datasource` | datasource (loki) | Selects the `Loki` datasource |
| `namespace` | query | Filter events by source namespace |
| `contains` | textbox | Regex search filter across event messages |

## Log source

Events are collected by the OTel Collector `k8s_events` receiver (no separate deployment needed) and pushed to Loki via the `logs/k8sevents` pipeline. Events arrive with:
- Stream labels: `service_name=k8sevents`, `k8s_namespace_name=<namespace>`
- Structured metadata: `k8s_event_reason`, `k8s_object_kind`, `k8s_source_component`
- Severity: Normal → `detected_level=info`, Warning → `detected_level=warn`

Verify events are arriving in Loki:
```bash
kubectl port-forward -n observability svc/loki 3100:3100 &
PF=$!; sleep 3
curl -sG "http://localhost:3100/loki/api/v1/series" \
  --data-urlencode 'match[]={service_name="k8sevents"}' | \
  python3 -c "import sys,json; d=json.load(sys.stdin); print('streams:', len(d.get('data',[])), '— k8s events flowing' if d.get('data') else 'no data yet')"
kill $PF 2>/dev/null
```

## Update this dashboard
This file was vendored from grafana.com ID 17882 with `__requires` and `__elements` stripped (required for Grafana 12). To re-vendor:

```bash
curl -sL "https://grafana.com/api/dashboards/17882/revisions/latest/download" -o /tmp/loki-k8s-events-new.json

python3 -c "
import json
with open('/tmp/loki-k8s-events-new.json') as f: d = json.load(f)
d.pop('__requires', None); d.pop('__elements', None)
with open('argocd-apps/grafana/chart/dashboards/loki-k8s-events.json', 'w') as f:
    json.dump(d, f, indent=2)
print('uid:', d.get('uid'), 'title:', d.get('title'))
"

# Verify datasource variable resolves to 'Loki' (type: loki)
python3 -c "
import json
with open('argocd-apps/grafana/chart/dashboards/loki-k8s-events.json') as f: d = json.load(f)
for t in d.get('templating',{}).get('list',[]):
    if t.get('type') == 'datasource':
        print('datasource var:', t.get('name'), '— query:', t.get('query'))
"
```

After editing: commit, push — ArgoCD auto-syncs Grafana.

## Known quirks
- **`__requires` / `__elements` must be stripped.** Grafana 12's file provisioner silently skips any JSON containing these keys. This was the root cause of this dashboard not appearing on first deploy.
- The `datasource` variable query is `loki` (lowercase) — this matches the Loki datasource type, not the datasource name. Grafana resolves it to the first available Loki-type datasource (`Loki` in this stack).
- The `contains` variable is a free-text regex box — it defaults to empty (matches all events).
- Stat panels (OOM kills, crashes, etc.) will show 0 if no such events have occurred — this is correct, not a data issue.
