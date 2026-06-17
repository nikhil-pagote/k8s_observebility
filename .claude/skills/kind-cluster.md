---
description: Start or stop the observability Kind cluster using Podman as the container runtime
argument-hint: "<start|stop|status>"
allowed-tools:
  - Bash
---

Manage the `observability-cluster` Kind cluster. Requires `KIND_EXPERIMENTAL_PROVIDER=podman`
and `DOCKER_HOST` set — sourced from `.envrc`.

## Start

```bash
source .envrc

# Check if cluster already exists
if kind get clusters 2>/dev/null | grep -q observability-cluster; then
  echo "Cluster already exists — exporting kubeconfig"
  kind export kubeconfig --name observability-cluster
else
  echo "Creating cluster..."
  kind create cluster --name observability-cluster --config kind-config.yaml
fi

kubectl get nodes
```

## Stop

```bash
source .envrc
kind delete cluster --name observability-cluster
```

## Status

```bash
source .envrc
kind get clusters
kubectl get nodes 2>/dev/null || echo "Cluster not reachable"
```
