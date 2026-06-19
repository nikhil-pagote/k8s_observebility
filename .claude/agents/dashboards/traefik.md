---
name: dashboard-traefik
description: Manage traefik.json dashboard — update, replace, fix datasource variable
model: claude-sonnet-4-6
tools:
  - Bash
  - Read
  - Write
scope: observability
tags:
  - dashboard
  - metrics
  - ingress
depends-on:
  - grafana
  - traefik
  - victoriametrics
---

# Agent Briefing — Dashboard: Traefik

## Identity
- **File:** `argocd-apps/grafana/chart/dashboards/traefik.json`
- **Title:** Kubernetes Traefik Ingress NextGen on Prometheus
- **uid:** `k8s-traefik-ingress-nextgen`
- **Grafana values.yaml key:** `traefik`
- **Datasource:** `VictoriaMetrics` (prometheus-compatible)
- **Tags:** `ingress`, `networking`, `services`, `k8s`, `traefik`

## What it shows (16 panels)
HTTP traffic and ingress health for Traefik:
- **Overview row:** total requests (period), req/2m rate, % success rate, HTTP 1xx/2xx/3xx/4xx/5xx counts
- **Request Volume:** time series of request throughput
- **Total HTTP Requests:** cumulative count
- **Request Success Rate:** % of non-5xx responses over time
- **HTTP Status Codes:** breakdown by status code
- **Latency:** p50/p95/p99 response times
- **Connections:** active connection count
- **Certificates:** TLS cert expiry status
- **CPU Intensive / Optional Graphs:** extended CPU metrics

## Template variables
| Variable | Type | Purpose |
|---|---|---|
| `datasource` | datasource (prometheus) | Selects VictoriaMetrics |
| `host` | query | Filter by Traefik host/entrypoint |
| `router` | query | Filter by Traefik router name |
| `pod` | query | Filter by Traefik pod |
| `scrape_interval` | interval | Matches OTel scrape interval (15s) |

## Metrics required
Sourced from **Traefik's built-in Prometheus metrics** (scraped by OTel Collector):
- `traefik_router_requests_total`
- `traefik_router_request_duration_seconds_*`
- `traefik_entrypoint_requests_total`
- `traefik_entrypoint_open_connections`
- `traefik_tls_certs_not_after`

Traefik metrics exposure must be enabled in `argocd-apps/traefik/values/values.yaml`:
```yaml
metrics:
  prometheus:
    enabled: true
```

## Update this dashboard
```bash
# Download newer version (check grafana.com for the latest Traefik dashboard)
curl -sL "https://grafana.com/api/dashboards/<ID>/revisions/latest/download" -o /tmp/traefik-new.json

# Strip __requires / __elements
python3 -c "
import json
with open('/tmp/traefik-new.json') as f: d = json.load(f)
d.pop('__requires', None); d.pop('__elements', None)
with open('argocd-apps/grafana/chart/dashboards/traefik.json', 'w') as f:
    json.dump(d, f, indent=2)
print('uid:', d.get('uid'), 'title:', d.get('title'))
"

# Verify datasource variable resolves to 'VictoriaMetrics'
python3 -c "
import json
with open('argocd-apps/grafana/chart/dashboards/traefik.json') as f: d = json.load(f)
for t in d.get('templating',{}).get('list',[]):
    if t.get('type') == 'datasource':
        print('datasource var:', t.get('name'), '— query:', t.get('query'))
"
```

After editing: commit, push — ArgoCD auto-syncs Grafana.

## Known quirks
- **`scrape_interval` variable** is hardcoded to `15s` in this vendored copy. The original grafana.com download set it to `${VAR_SCRAPE_INTERVAL}` (a Grafana Cloud-only placeholder), which caused "Invalid interval string" on all panels using `interval=$scrape_interval`. If re-vendoring, always patch the variable back to `15s`:
  ```python
  for t in d['templating']['list']:
      if t.get('name') == 'scrape_interval':
          t['query'] = '15s'
          t['current'] = {'value': '15s', 'text': '15s', 'selected': False}
          t['options'] = [{'value': '15s', 'text': '15s', 'selected': False}]
  ```
- Router and pod dropdowns will be empty if Traefik metrics are disabled or OTel isn't scraping the Traefik pod.
