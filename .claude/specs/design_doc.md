# Design Document — k8s Observability Stack

Technical implementation details, constraints, and known quirks. Companion to `architecture.md` and `otel-pipeline.md`.

---

## ArgoCD Application Management

All ArgoCD `Application` CRDs live in `argocd-apps/<app>/app.yaml`. They are applied manually via:
```bash
kubectl apply -k argocd-apps/          # apply all apps
kubectl apply -f argocd-apps/<app>/app.yaml  # apply one app
```

**These files are NOT self-managed by ArgoCD.** A git push alone does not update the in-cluster Application CRD — `kubectl apply` must be run after any change to an `app.yaml`.

### Sync options

All apps use `automated` sync with `prune: true` and `selfHeal: true`. App-specific overrides:

| App | Extra syncOptions | Reason |
|---|---|---|
| `grafana` | `ServerSideApply=true` | Total dashboard ConfigMap size exceeds the 262KB `kubectl.kubernetes.io/last-applied-configuration` annotation limit |

### Sync-wave order

| Wave | Apps |
|---|---|
| 0 | traefik |
| 1 | victoria-metrics, grafana, jaeger, loki, opentelemetry-collector, node-exporter |

---

## Grafana

**Version:** 12.4.3

**Persistence:** `/var/lib/grafana` on a 10Gi PVC (`persistence.enabled: true`). Pod restarts retain all data.

**Sub-path routing:** `serve_from_sub_path=true` and `root_url=http://localhost:30080/grafana` set in `grafana.ini` Helm values. Both must match or the UI breaks.

### Datasources

| Name | Type | URL | Default |
|---|---|---|---|
| VictoriaMetrics | prometheus | `http://victoria-metrics.observability.svc.cluster.local:8428` | yes |
| Loki | loki | `http://loki.observability.svc.cluster.local:3100` | no |

### Dashboard provisioning

Dashboards are file-provisioned from `argocd-apps/grafana/chart/dashboards/` via the `default` provider (path: `/var/lib/grafana/dashboards/default`, `updateIntervalSeconds: 10`).

Wired into Grafana via `dashboards.default` in `values/values.yaml`.

#### Vendored dashboards

| values.yaml key | File | Source | uid | Datasource var |
|---|---|---|---|---|
| `kubernetes-views-global` | `kubernetes-views-global.json` | Vendored | _(none)_ | `datasource` → VictoriaMetrics |
| `traefik` | `traefik.json` | Vendored | `k8s-traefik-ingress-nextgen` | `datasource` → VictoriaMetrics |
| `node-exporter` | `node-exporter.json` | Vendored | `rYdddlPWk` | `DS_PROMETHEUS` → VictoriaMetrics |
| `loki-k8s-events` | `loki-k8s-events.json` | grafana.com ID 17882 | `kubernetes-event-exporter` | `datasource` → Loki |

#### Runtime-downloaded dashboard

| values.yaml key | gnetId | Revision | Datasource |
|---|---|---|---|
| `victoria-metrics` | 10229 | 35 | VictoriaMetrics |

### Traefik dashboard — scrape_interval fix

The vendored `traefik.json` uses a `scrape_interval` constant variable. The original value was `${VAR_SCRAPE_INTERVAL}` — a Grafana Cloud placeholder that does not exist in self-hosted Grafana. All panels that set `interval=$scrape_interval` showed "Invalid interval string".

Fixed by hardcoding the value to `15s` (matching the OTel Collector scrape interval) in the variable definition.

### Loki K8s Events dashboard — query format

The dashboard (grafana.com ID 17882) was designed for `kubernetes-event-exporter` which outputs JSON to stdout. This stack uses the OTel `k8s_events` receiver instead. All LogQL queries were rewritten:

| Old (kubernetes-event-exporter) | New (OTel k8s_events) |
|---|---|
| `{container="event-exporter"}` | `{service_name="k8sevents", k8s_namespace_name=~"$namespace"}` |
| `\| json \| __error__=\`\`` | _(removed — no JSON body)_ |
| `\| metadata_namespace=~"$namespace"` | _(moved to stream selector)_ |
| `\| reason="X"` | `\| k8s_event_reason="X"` |
| `\| type="Warning"` | `\| detected_level="warn"` |
| `sum by (reason)` | `sum by (k8s_event_reason)` |
| `sum by (metadata_namespace)` | `sum by (k8s_namespace_name)` |
| `sum by (source_component)` | `sum by (k8s_source_component)` |
| `sum by (involvedObject_kind)` | `sum by (k8s_object_kind)` |

### Grafana 12 file provisioner constraint

Grafana 12's file provisioner **silently skips** any JSON file containing `__requires` or `__elements` as top-level keys. These keys are added by grafana.com's import wizard and must be stripped before vendoring:

```python
d.pop('__requires', None)
d.pop('__elements', None)
```

No error is logged when a file is skipped — the dashboard simply does not appear.

### PVC residue

When a dashboard is removed from git and ArgoCD syncs, the JSON file is deleted from the ConfigMap but the copy on the PVC at `/var/lib/grafana/dashboards/default/<name>.json` persists. Grafana continues serving the stale dashboard from the PVC. Remove it manually:

```bash
kubectl exec -n observability deployment/grafana -- rm /var/lib/grafana/dashboards/default/<name>.json
```

Grafana auto-purges it from its database within seconds of file deletion.

---

## VictoriaMetrics

Single-node mode (`victoria-metrics-single` chart). Receives all metrics via `prometheusremotewrite` from the OTel Collector.

- Remote write endpoint: `http://victoria-metrics.observability.svc.cluster.local:8428/api/v1/write`
- Query API (Prometheus-compatible): `http://victoria-metrics.observability.svc.cluster.local:8428`
- UI path: `/vmui` (no prefix strip — VictoriaMetrics serves this path natively)
- All scraped metrics carry the label `origin_prometheus=otel-collector` (set via `external_labels` on the remote_write exporter)

---

## Loki

Single-binary deployment, filesystem storage. Not distributed mode.

- Ingestion: OTLP only via OTel Collector (`http://loki.observability.svc.cluster.local:3100/otlp`)
- No filelog DaemonSet — logs arrive exclusively via OTel push
- Query API: `http://loki.observability.svc.cluster.local:3100`

---

## Jaeger

In-memory storage (POC). Traces are lost on pod restart.

- Receives traces from OTel Collector via OTLP gRPC: `jaeger.observability.svc.cluster.local:4317`
- UI path: `/jaeger` via Traefik

---

## OTel Collector

`opentelemetry-collector-contrib` image required (not the core image) — contrib ships the `prometheusremotewrite` exporter, `otlphttp/loki` exporter, `prometheus` receiver, and `k8s_events` receiver.

Plain Deployment (no OTel Operator, no cert-manager).

In-cluster OTLP endpoint for applications: `http://opentelemetry-collector.observability.svc.cluster.local:4317`

### Kubernetes Events pipeline

The `k8s_events` receiver watches the Kubernetes API for cluster events across all namespaces and forwards them to Loki via a dedicated `logs/k8sevents` pipeline. No separate `kubernetes-event-exporter` deployment is needed.

Events arrive in Loki with:
- Stream label `service_name=k8sevents` (set via `resource/k8sevents` processor)
- Stream label `k8s_namespace_name=<namespace>` (from `k8s.namespace.name` resource attribute)
- Structured metadata: `k8s_event_reason`, `k8s_object_kind`, `k8s_object_name`, `k8s_source_component`
- Severity: Normal → `detected_level=info`, Warning → `detected_level=warn`

RBAC required: `events` resource must be in the OTel ClusterRole (under `clusterRole.rules` in `values/values.yaml`).

---

## Traefik

- Deployed in `traefik` namespace (all others in `observability`)
- sync-wave 0 — must be Running before any other app creates IngressRoutes
- IngressRoute definitions for observability UIs: `argocd-apps/observability-ingress.yaml`
- Metrics exposed on port 8082, scraped by OTel Collector via kubernetes_sd

---

## Prometheus Exporters

Only node-exporter is deployed as a Prometheus exporter; it deploys into the `observability` namespace and is scraped by the OTel Collector prometheus receiver.

| Exporter | Workload | Scrape port | Label `job` |
|---|---|---|---|
| node-exporter | DaemonSet | 9100 | `node-exporter` |

Kubernetes object state (formerly kube-state-metrics) is now provided by the OTel `k8s_cluster` receiver, emitting `k8s_*` metrics directly. Pushgateway was removed — batch jobs should send OTLP to the collector on `:4317`.

node-exporter runs as a DaemonSet — one pod per node. Verify coverage with `kubectl get ds -n observability`.

---

## Kind Cluster

Name: `observability-cluster`
Config: `kind-config.yaml`
Kubernetes: v1.33.1
Topology: 1 control-plane + 3 workers
Port mappings: `30080 → :80`, `30443 → :443`
Container runtime: Podman (not Docker)

Required env vars (set in `.envrc` and injected via `.claude/settings.json`):
```bash
KIND_EXPERIMENTAL_PROVIDER=podman
DOCKER_HOST=unix:///run/user/1000/podman/podman.sock
```
