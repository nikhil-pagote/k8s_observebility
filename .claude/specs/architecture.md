# Architecture Spec — k8s Observability Stack

## Overview

A local Kubernetes observability POC using **OpenTelemetry** as the unified collection layer, deployed via GitOps (ArgoCD), and exposed through a single Traefik ingress on path-based routing. **Istio** runs as a service mesh (Envoy sidecar mode) in the `observability` namespace, providing in-cluster mTLS and traffic observability. Traefik remains the ingress controller — Istio is mesh-only.

## Three Pillars

| Pillar | Tool | Role |
|---|---|---|
| Metrics | VictoriaMetrics + Grafana | Store and visualize metrics (OTel pushes via remote_write) |
| Traces | Jaeger | Distributed trace storage and UI |
| Logs | Loki | Log aggregation and querying |

## Data Flow

```
node-exporter  ──────────────────────────────┐
cAdvisor (kubelet /metrics/cadvisor) ────────┤ OTel prometheus receiver (pull)
kubelet  (kubelet /metrics) ─────────────────┤   (volume stats for PVCs)
annotated pods/endpoints ────────────────────┘
                                             │
Kubernetes API (state) ───────────────────── ┤ k8s_cluster receiver (watch)
                                             │
App (OTLP :4317/4318) ───────────────────────┤ OTel Collector
                                             │
Kubernetes API (events) ─────────────────────┤ k8s_events receiver (watch)
                                             │
                              ┌──────────────┼──────────────┐
                              ▼              ▼              ▼
                      VictoriaMetrics     Jaeger          Loki
                      (remote_write)    (OTLP traces)  (OTLP logs)
                              │              │              │
                              └──────────────┴──────────────┘
                                             ▼
                                          Grafana

In-cluster pod-to-pod traffic (observability ns):
  Envoy sidecar ──mTLS──▶ Envoy sidecar   (controlled by istiod)
                                ▼
                             Kiali  (reads from VictoriaMetrics + Jaeger)
```

## Component Map

| Component | Namespace | Helm Chart | Purpose |
|---|---|---|---|
| Traefik | traefik | traefik/traefik | Ingress, NodePort 30080/30443 |
| ArgoCD | argocd | argo/argo-cd | GitOps reconciler (managed outside App of Apps — Helm upgrade only) |
| Gateway API CRDs | kube-system | vendored YAML v1.2.1 | Required by istiod at startup |
| istiod | istio-system | istio/istiod | Istio control plane — Envoy injection, mTLS, traffic policy |
| istio-base | istio-system | istio/base | Istio CRDs and cluster-level RBAC |
| VictoriaMetrics | observability | victoriametrics/victoria-metrics-single | Metrics storage backend (receives remote_write) |
| Grafana | observability | grafana/grafana | Unified visualization |
| Jaeger | observability | jaegertracing/jaeger | Trace store (in-memory for POC) |
| Loki | observability | grafana/loki | Log store (single-binary, filesystem) |
| OTel Collector | observability | open-telemetry/opentelemetry-collector | Single ingestion layer — scrapes infra, receives OTLP from apps |
| node-exporter | observability | prometheus-community/prometheus-node-exporter | Host metrics (CPU, mem, disk, network) — DaemonSet |
| Kiali | observability | kiali/kiali-server | Istio mesh visualization — topology, traffic, health |

## Ingress Routing

All UIs via Traefik at `http://localhost:30080`:

| Path | Service | Port | Notes |
|---|---|---|---|
| `/grafana` | grafana | 80 | sub-path routing via `serve_from_sub_path=true` |
| `/vmui` | victoria-metrics | 8428 | no prefix strip — VM serves vmui natively at /vmui |
| `/jaeger` | jaeger | 16686 | |
| `/kiali` | kiali | 20001 | managed by Kiali's own Helm chart ingress |
| `/traefik` | Traefik Dashboard | — | redirects to `/dashboard/` via middleware |
| `/argocd` | argocd-server (argocd ns) | 80 | cross-namespace IngressRoute in traefik ns |

`/grafana`, `/vmui`, `/jaeger` are defined in `argocd-apps/observability-ingress.yaml`.
`/kiali` is defined via `deployment.ingress.override_yaml` in `argocd-apps/kiali/values/values.yaml`.

## Service Mesh (Istio — Envoy sidecar mode)

- **Scope:** `observability` namespace only. Labeled `istio-injection: enabled` by `istio-mesh-config` app.
- **Traefik and ArgoCD are NOT in the mesh** — no sidecar injection in `traefik` or `argocd` namespaces.
- **PeerAuthentication:** PERMISSIVE (default) — allows plain HTTP from Traefik into mesh pods.
- **Sidecar injection:** istiod's MutatingWebhook injects `istio-proxy` (Envoy) + `istio-init` into every new pod in labeled namespaces. Existing pods need `kubectl rollout restart` after namespace labeling.
- **Kiali:** reads metrics from VictoriaMetrics (Prometheus-compatible), traces from Jaeger, links to Grafana. Auth: anonymous.

## GitOps (App of Apps)

`root-app.yaml` (repo root) is the **bootstrap** — applied once manually:
```bash
kubectl apply -f root-app.yaml
```

After that, ArgoCD manages everything in `argocd-apps/` automatically. Adding a new app = create `argocd-apps/<app>/app.yaml` + add to `kustomization.yaml`, push — ArgoCD picks it up.

ArgoCD itself is **not** in `kustomization.yaml`. It is managed directly via:
```bash
helm upgrade argocd argocd-apps/argocd/chart -n argocd -f argocd-apps/argocd/values/values.yaml
```

## ArgoCD Sync Order (sync-wave)

| Wave | Apps |
|---|---|
| -2 | gateway-api-crds |
| -1 | istio-base |
| 0 | istio-istiod, traefik (traefik must be up before IngressRoutes) |
| 1 | istio-mesh-config, VictoriaMetrics, Grafana, Jaeger, Loki, OTel Collector, node-exporter |
| 2 | Kiali |

## ArgoCD Global Config (argocd-cm)

Two global customizations in `argocd-apps/argocd/values/values.yaml`:

- **Ingress health:** always returns Healthy — Traefik in NodePort mode never populates `status.loadBalancer` ADDRESS, which ArgoCD's default health check requires.
- **Webhook ignoreDifferences:** istiod modifies `ValidatingWebhookConfiguration` and `MutatingWebhookConfiguration` at runtime (caBundle, failurePolicy, matchPolicy, etc.). Global `jqPathExpressions` in argocd-cm prevent these fields from triggering OutOfSync.

## Kind Cluster

Name: `observability-cluster`
Kubernetes: v1.33.1
Topology: 1 control-plane + 3 workers
Port mappings: `30080 → :80`, `30443 → :443`
Config: `kind-config.yaml`

## Container Runtime

**Podman** — Docker is not used. Set via `.envrc` (sourced per shell session):

```bash
KIND_EXPERIMENTAL_PROVIDER=podman
DOCKER_HOST=unix:///run/user/1000/podman/podman.sock
```

Also injected automatically by Claude Code via the `env` block in `.claude/settings.json`.

## OTel Operator

Not used. The stack runs a plain `opentelemetry-collector-contrib` Deployment (no OTel Operator, no cert-manager). The guide `otel operator for k8s.md` documents an alternative operator-based approach — it is reference only and does not reflect the current deployment.

## Grafana Data Sources

Auto-provisioned via Grafana Helm values:
- VictoriaMetrics (as Prometheus type): `http://victoria-metrics.observability.svc.cluster.local:8428`
- Loki: `http://loki.observability.svc.cluster.local:3100`

Jaeger traces can be explored directly at `http://localhost:30080/jaeger`. A Grafana Jaeger datasource can be added manually for trace correlation:
- Jaeger: `http://jaeger.observability.svc.cluster.local:16686`
