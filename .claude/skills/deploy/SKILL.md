---
description: Guided step-by-step deployment of the full observability stack with pre-flight checks at each stage
argument-hint: "[--step <1-5>]"
allowed-tools:
  - Bash
  - Read
---

Deploy the full stack using kubectl, helm, and kind with Podman as the container runtime.
Pass `--step N` to run only a specific step.

## Pre-flight

```bash
# Podman (required — Docker is not used)
podman version 2>/dev/null && echo "Podman: OK" || echo "Podman: NOT FOUND — install podman"

# Verify Podman socket is running (needed by kind)
ls "${XDG_RUNTIME_DIR}/podman/podman.sock" 2>/dev/null \
  && echo "Podman socket: OK" \
  || echo "Podman socket: NOT RUNNING — run: systemctl --user start podman.socket"

# Verify env vars are set (sourced from .envrc or shell profile)
[ "$KIND_EXPERIMENTAL_PROVIDER" = "podman" ] \
  && echo "KIND_EXPERIMENTAL_PROVIDER: OK" \
  || echo "KIND_EXPERIMENTAL_PROVIDER not set — run: source .envrc"

[ -n "$DOCKER_HOST" ] \
  && echo "DOCKER_HOST: $DOCKER_HOST" \
  || echo "DOCKER_HOST not set — run: source .envrc"

kubectl version --client 2>/dev/null && echo "kubectl: OK" || echo "kubectl: MISSING"
kind version 2>/dev/null && echo "kind: OK" || echo "kind: MISSING"
helm version 2>/dev/null && echo "helm: OK" || echo "helm: MISSING"
```

Stop if any check fails.

## Steps

### Step 1 — Kind cluster (via Podman)

```bash
kind create cluster --name observability-cluster --config kind-config.yaml
kubectl get nodes  # verify: 1 control-plane + 3 workers, all Ready
```

### Step 2 — ArgoCD

```bash
kubectl create namespace argocd
kubectl apply -n argocd -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml
kubectl wait --for=condition=Available deployment/argocd-server -n argocd --timeout=120s
```

### Step 3 — Observability stack

```bash
kubectl apply -k argocd-apps/
```

Wait for ArgoCD to sync: `kubectl get applications -n argocd` — all Synced/Healthy.
ArgoCD sync can take 5–10 minutes as Helm charts pull from upstream.

### Step 4 — Sample apps

```bash
kubectl apply -f apps/load-generator/ -n observability
kubectl apply -f apps/sample-app/deployment-basic.yaml -n observability
```

### Step 5 — Verify ingress

```bash
kubectl get ingressroute -n observability
curl -sI http://localhost:30080/grafana | head -1  # expect HTTP/1.1 200 or 302
```

## Post-deploy

Run `/verify-otel` to confirm all three pillars are flowing.
