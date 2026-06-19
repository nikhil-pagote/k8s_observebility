---
description: Deploy the observability stack — ArgoCD via Helm, then bootstrap the App of Apps so ArgoCD manages everything else
argument-hint: "[--step <1-4>]"
allowed-tools:
  - Bash
  - Read
---

Deploy flow:
1. Pre-flight checks
2. Helm installs ArgoCD onto the Kind cluster
3. Bootstrap the App of Apps (`root-app.yaml`) — ArgoCD then reconciles all apps from `argocd-apps/`
4. Monitor deployment and activate Istio sidecars

Pass `--step N` to run only a specific step.

## Pre-flight

```bash
source .envrc

# Tooling
podman version 2>/dev/null && echo "podman: OK" || echo "podman: NOT FOUND"
kind version 2>/dev/null && echo "kind: OK" || echo "kind: NOT FOUND"
kubectl version --client 2>/dev/null && echo "kubectl: OK" || echo "kubectl: NOT FOUND"
helm version 2>/dev/null && echo "helm: OK" || echo "helm: NOT FOUND"

# Kind cluster must be running
kind get clusters | grep -q observability-cluster \
  && echo "cluster: OK" \
  || echo "cluster: NOT RUNNING — run /kind-cluster start first"

# Local charts must be present
missing=()
for app in argocd traefik grafana victoria-metrics node-exporter jaeger loki opentelemetry-collector istio-base istio-istiod kiali; do
  [ -f "argocd-apps/$app/chart/Chart.yaml" ] || missing+=("$app")
done
[ -f "argocd-apps/gateway-api-crds/manifests/standard-install.yaml" ] || missing+=("gateway-api-crds/manifests")
[ ${#missing[@]} -eq 0 ] \
  && echo "charts: OK" \
  || echo "charts: MISSING — run /helm pull: ${missing[*]}"
```

Stop if any check fails before proceeding.

## Step 1 — Deploy ArgoCD via Helm

```bash
helm upgrade --install argocd argocd-apps/argocd/chart \
  --namespace argocd \
  --create-namespace \
  -f argocd-apps/argocd/values/values.yaml \
  --wait

kubectl get pods -n argocd
```

Retrieve the initial admin password:

```bash
kubectl get secret argocd-initial-admin-secret -n argocd \
  -o jsonpath='{.data.password}' | base64 -d && echo
```

## Step 2 — Register Git repo with ArgoCD

ArgoCD needs access to this Git repo to read app manifests and charts.

```bash
# Install ArgoCD CLI if not present
# https://argo-cd.readthedocs.io/en/stable/cli_installation/

# Port-forward ArgoCD server (run in background)
kubectl port-forward svc/argocd-server -n argocd 8080:443 &
ARGOCD_PF_PID=$!

# Login
ARGOCD_PASS=$(kubectl get secret argocd-initial-admin-secret -n argocd \
  -o jsonpath='{.data.password}' | base64 -d)
argocd login localhost:8080 --username admin --password "$ARGOCD_PASS" --insecure

# Register the repo (SSH key or HTTPS token required for private repos)
argocd repo add https://github.com/nikhil-pagote/k8s_observebility.git

kill $ARGOCD_PF_PID 2>/dev/null
```

## Step 3 — Bootstrap the App of Apps

Apply `root-app.yaml` once. ArgoCD then takes ownership of every app listed in `argocd-apps/kustomization.yaml`:

```bash
kubectl apply -f root-app.yaml
kubectl get applications -n argocd
```

ArgoCD reconciles in sync-wave order:

| Wave | Apps |
|---|---|
| -2 | gateway-api-crds |
| -1 | istio-base |
| 0 | istio-istiod, traefik |
| 1 | istio-mesh-config, victoria-metrics, grafana, jaeger, loki, opentelemetry-collector, node-exporter |
| 2 | kiali |

After this bootstrap, all future changes are GitOps — push to `istio-envoy` and ArgoCD picks them up automatically.

**ArgoCD itself is NOT managed by the App of Apps.** To update ArgoCD config:
```bash
helm upgrade argocd argocd-apps/argocd/chart -n argocd -f argocd-apps/argocd/values/values.yaml
kubectl rollout restart statefulset/argocd-application-controller -n argocd
```

## Step 4 — Monitor and activate sidecars

```bash
# Watch all applications converge
kubectl get applications -n argocd -w

# Wait for istiod before restarting observability pods
kubectl rollout status deployment/istiod -n istio-system

# Watch observability pods come up
kubectl get pods -n observability -w

# Rolling restart to inject Envoy sidecars into existing pods
kubectl rollout restart deployment -n observability
kubectl rollout restart daemonset -n observability
kubectl get pods -n observability   # expect 2/2 READY per pod

# Verify ingress
curl -sI http://localhost:30080/grafana | head -1
curl -sI http://localhost:30080/jaeger | head -1
curl -sI http://localhost:30080/kiali | head -1
```

### Access the UIs

| UI | URL | Credentials |
|---|---|---|
| Grafana | http://localhost:30080/grafana | admin / admin123 |
| VictoriaMetrics | http://localhost:30080/vmui | set Server URL to `http://localhost:30080/vmui` in vmui settings (gear icon) |
| Jaeger | http://localhost:30080/jaeger | — |
| Kiali | http://localhost:30080/kiali | anonymous |
| Traefik | http://localhost:30080/traefik | — |
| ArgoCD | http://localhost:30080/argocd | admin / see Step 1 |

Run `/verify-otel` to confirm all three pillars are flowing end-to-end.
