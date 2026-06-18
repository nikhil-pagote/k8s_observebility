---
description: Deploy the observability stack — ArgoCD via Helm, then ArgoCD manages all apps from argocd-apps/
argument-hint: "[--step <1-4>]"
allowed-tools:
  - Bash
  - Read
---

Deploy flow:
1. Pre-flight checks
2. Helm installs ArgoCD onto the Kind cluster
3. ArgoCD Application CRDs are applied from `argocd-apps/`
4. ArgoCD reads each app's local `chart/` and `values/values.yaml` and deploys them

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
for app in traefik grafana victoria-metrics node-exporter kube-state-metrics pushgateway jaeger loki opentelemetry-collector; do
  [ -f "argocd-apps/$app/chart/Chart.yaml" ] || missing+=("$app")
done
[ ${#missing[@]} -eq 0 ] \
  && echo "charts: OK" \
  || echo "charts: MISSING — run /helm add-repos then /helm pull: ${missing[*]}"
```

Stop if any check fails before proceeding.

## Step 1 — Deploy ArgoCD via Helm

```bash
helm repo add argo https://argoproj.github.io/argo-helm
helm repo update argo

helm upgrade --install argocd argo/argo-cd \
  --namespace argocd \
  --create-namespace \
  --wait

kubectl get pods -n argocd
```

Retrieve the initial admin password:

```bash
kubectl get secret argocd-initial-admin-secret -n argocd \
  -o jsonpath='{.data.password}' | base64 -d && echo
```

## Step 2 — Register Git repo with ArgoCD

ArgoCD needs access to this Git repo to read the local `chart/` and `values/` directories.

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

## Step 3 — Apply ArgoCD Application manifests

This tells ArgoCD what apps to deploy and where to find their charts and values:

```bash
kubectl apply -k argocd-apps/
kubectl get applications -n argocd
```

ArgoCD will now deploy all apps from the local `chart/` directories using `values/values.yaml`.

Sync order (controlled by `argocd.argoproj.io/sync-wave`):
- Wave 0: Traefik
- Wave 1: VictoriaMetrics, Grafana, node-exporter, kube-state-metrics, Jaeger, Loki, OTel Collector

## Step 4 — Monitor deployment

```bash
# Watch all applications converge
kubectl get applications -n argocd -w

# Watch pods come up
kubectl get pods -n observability -w
kubectl get pods -n traefik -w

# Verify ingress once Traefik is Ready
curl -sI http://localhost:30080/grafana | head -1
curl -sI http://localhost:30080/jaeger | head -1
```

Run `/verify-otel` to confirm all three pillars are flowing end-to-end.
