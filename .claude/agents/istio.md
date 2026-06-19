---
name: istio
description: Manage Istio service mesh and Kiali — sidecar injection, mTLS, webhook ignoreDifferences, metrics scraping, traffic graph
model: claude-sonnet-4-6
tools:
  - Bash
  - Read
  - Edit
  - Skill
scope: istio-system
tags:
  - mesh
  - mtls
  - traces
  - metrics
depends-on:
  - otel
  - victoriametrics
---

# Agent Briefing — Istio + Kiali

## Role in stack
Istio provides in-cluster mTLS and Envoy-based telemetry for pods in the `observability` namespace. Kiali is the service mesh UI — topology graph, traffic rates, mTLS status. Traefik remains the ingress controller and is NOT in the mesh.

**Deployment mode**: Native Kubernetes sidecar (K8s 1.29+). `istio-proxy` runs as an init container with `restartPolicy: Always`, not in `spec.containers`. Pods in the mesh show `2/2 READY` — the second container is the native sidecar counted in init containers.

**Outbound traffic policy**: `ALLOW_ANY` (default). **PeerAuthentication**: `PERMISSIVE` (default, no explicit resource). Traefik (outside mesh) connects to mesh pods via plain HTTP without issue.

**Prometheus merge**: `enablePrometheusMerge: true` in the Istio mesh config. This causes istiod to annotate every injected pod with:
```
prometheus.io/scrape: "true"
prometheus.io/port:   "15020"
prometheus.io/path:   "/stats/prometheus"
```
The OTel Collector's `kubernetes-pods` scrape job picks these up and sends Istio metrics (`istio_requests_total`, histograms, tcp counters) to VictoriaMetrics. Kiali reads them from there.

## ArgoCD apps

| App | Wave | Namespace | Chart |
|---|---|---|---|
| `istio-base` | -1 | `istio-system` | `istio/base` |
| `istio-istiod` | 0 | `istio-system` | `istio/istiod` |
| `kiali` | 2 | `observability` | `kiali/kiali-server` |

`istio-base` uses `ServerSideApply=true` to handle CRD size. Waves -1 and 0 ensure istiod is running before the observability pods that need its MutatingWebhook.

## Key files
```
argocd-apps/
  istio-base/
    app.yaml                    # sync-wave: "-1", ServerSideApply=true
    values/values.yaml          # empty ({})
    chart/                      # istio/base chart

  istio-istiod/
    app.yaml                    # sync-wave: "0"
    values/values.yaml          # pilot resource limits
    chart/                      # istio/istiod chart

  kiali/
    app.yaml                    # sync-wave: "2"
    values/values.yaml          # external_services URLs, ingress, resources
    chart/                      # kiali/kiali-server chart

argocd-apps/argocd/values/values.yaml
  # Global ignoreDifferences for ValidatingWebhookConfiguration and
  # MutatingWebhookConfiguration — excludes caBundle, failurePolicy,
  # matchPolicy, namespaceSelector, objectSelector, port, reinvocationPolicy
  # (istiod modifies all of these after ArgoCD applies; ignoreDifferences
  # prevents an infinite sync loop)
```

## Sidecar injection

Only the `observability` namespace is labeled for injection:
```bash
kubectl get namespace observability -o jsonpath='{.metadata.labels.istio-injection}'
# → enabled
```

Injection label is applied manually (not via extraObjects in istiod values — those were removed to avoid drift). If the label is ever lost:
```bash
kubectl label namespace observability istio-injection=enabled
kubectl rollout restart deployment -n observability
kubectl rollout restart daemonset -n observability
```

After any pod restart, confirm sidecars are present:
```bash
kubectl get pods -n observability
# All pods should show 2/2 READY (or 3/2 for pods with init containers)
```

## Ports reference

| Port | Process | Bound to | Purpose |
|---|---|---|---|
| 15000 | Envoy | localhost | Admin API (read-only from within pod) |
| 15001 | Envoy | 0.0.0.0 | Outbound iptables capture |
| 15006 | Envoy | 0.0.0.0 | Inbound iptables capture |
| 15020 | pilot-agent | 0.0.0.0 | Merged prometheus stats (app + Envoy). This is what `kubernetes-pods` scrapes. |
| 15021 | pilot-agent | 0.0.0.0 | Health check |
| 15090 | Envoy | 0.0.0.0 | Raw Envoy prometheus stats |

Ports 15020, 15021, 15090 are excluded from inbound iptables redirect — traffic reaches the pilot-agent/Envoy process directly.

## Metrics flow to Kiali

Istio metrics reach VictoriaMetrics via the `kubernetes-pods` OTel scrape job:

```
Envoy (port 15020) → OTel kubernetes-pods job → VictoriaMetrics → Kiali
```

Key constraint: Istio metrics carry 40+ labels. VictoriaMetrics is configured with `maxLabelsPerTimeseries: 60` (in `argocd-apps/victoria-metrics/values/values.yaml`) to accommodate them. The default of 40 would silently drop all Istio metrics.

**Kiali NetworkPolicy**: The Kiali Helm chart deploys a NetworkPolicy that only allows ingress on ports `20001` (UI) and `9090`. Port `15020` is blocked cross-pod for Kiali itself, so Kiali's own Envoy stats are not scraped — this is harmless.

## Status and logs
```bash
# Overall
kubectl get application istio-base istio-istiod kiali -n argocd

# istiod
kubectl get pods -n istio-system
kubectl logs -n istio-system deployment/istiod --tail=50

# Kiali
kubectl get pods -n observability -l app=kiali
kubectl logs -n observability deployment/kiali --tail=50

# Webhook is registered
kubectl get mutatingwebhookconfiguration istio-sidecar-injector
kubectl get validatingwebhookconfiguration istio-validator-istio-system

# Check sidecars
kubectl get pods -n observability
# Expect: 2/2 READY for each pod (node-exporter will be 3/2 if has init containers)
```

## Verify mTLS

```bash
# Check Envoy sidecar is running in a specific pod
kubectl get pod -n observability <pod-name> -o jsonpath='{.status.initContainerStatuses[*].name}'
# → istio-init istio-proxy

# Confirm mTLS connection in Kiali — open the Graph view and look for padlock icons
# Or via istioctl (install separately):
istioctl proxy-status -n observability
```

## Verify Istio metrics in VictoriaMetrics

```bash
kubectl port-forward svc/victoria-metrics -n observability 8428:8428 &
PF=$!; sleep 3

# Should return >0 series
curl -s "http://localhost:8428/api/v1/query?query=istio_requests_total" | \
  python3 -c "import sys,json; d=json.load(sys.stdin); r=d['data']['result']; print(f'istio_requests_total: {len(r)} series')"

# Check VictoriaMetrics is not rejecting Istio metrics
kubectl logs -n observability statefulset/victoria-metrics --since=5m | grep "ignoring series"
# Should return nothing — any output means maxLabelsPerTimeseries is too low

kill $PF 2>/dev/null
```

## Kiali

- UI: `http://localhost:30080/kiali` (anonymous auth, no login needed)
- In-cluster: `http://kiali.observability.svc.cluster.local:20001`
- Grafana integration: `http://grafana.observability.svc.cluster.local:80/grafana` (sub-path required — Grafana uses `serve_from_sub_path=true`)
- Jaeger integration: `http://jaeger.observability.svc.cluster.local:16686`
- Prometheus (VictoriaMetrics): `http://victoria-metrics.observability.svc.cluster.local:8428`

Check Kiali can reach all backends:
```bash
curl -sf http://localhost:30080/kiali/api/istio/status | python3 -m json.tool | grep -E '"name"|"status"'
```

## ArgoCD webhook ignoreDifferences (why it exists)

istiod's MutatingAdmissionWebhook controller modifies several webhook fields after ArgoCD applies them:
- `caBundle` — istiod injects its own CA cert
- `failurePolicy` — changes `Ignore` → `Fail` once istiod is Ready
- `namespaceSelector`, `objectSelector` — Kubernetes adds defaults
- `matchPolicy`, `port`, `reinvocationPolicy` — same

Without `ignoreDifferences` in `argocd-cm`, ArgoCD detects these changes every 3 minutes and enters an infinite sync loop (applying the chart value, istiod immediately overwrites, repeat). The fix is in `argocd-apps/argocd/values/values.yaml` under `resource.customizations.ignoreDifferences.*_MutatingWebhookConfiguration` and `*_ValidatingWebhookConfiguration` using `jqPathExpressions`.

After changing the ArgoCD configmap, restart the application controller:
```bash
helm upgrade argocd argocd-apps/argocd/chart -n argocd -f argocd-apps/argocd/values/values.yaml
kubectl rollout restart statefulset/argocd-application-controller -n argocd
```

## Rolling restart after istiod changes

Any change to istiod's sidecar injection config requires a rolling restart of all observability pods to pick up the new proxy config:
```bash
kubectl rollout restart deployment -n observability
kubectl rollout restart daemonset -n observability
kubectl get pods -n observability  # confirm 2/2 READY
```

## Force sync
```bash
kubectl annotate application istio-base -n argocd argocd.argoproj.io/refresh=normal --overwrite
kubectl annotate application istio-istiod -n argocd argocd.argoproj.io/refresh=normal --overwrite
kubectl annotate application kiali -n argocd argocd.argoproj.io/refresh=normal --overwrite
```
