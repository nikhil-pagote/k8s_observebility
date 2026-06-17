---
description: Show health of every component in the observability stack across all namespaces
allowed-tools:
  - Bash
---

Check and summarize the status of the full stack.

## Steps

```bash
# Cluster nodes
kubectl get nodes -o wide

# ArgoCD sync status
kubectl get applications -n argocd -o wide 2>/dev/null || echo "ArgoCD not deployed"

# Traefik
kubectl get pods -n traefik
kubectl get svc -n traefik traefik 2>/dev/null

# Observability namespace
kubectl get pods -n observability -o wide
kubectl get svc -n observability

# IngressRoutes
kubectl get ingressroute -n observability 2>/dev/null || kubectl get ingress -n observability 2>/dev/null

# Any pods not Running
kubectl get pods --all-namespaces | grep -vE "Running|Completed|NAME"
```

## Output format

Report as a table:

| Component | Namespace | Status | Notes |
|---|---|---|---|
| Kind cluster | — | Ready / Error | node count |
| ArgoCD | argocd | Synced / OutOfSync | |
| Traefik | traefik | Running / Error | NodePort 30080 |
| Prometheus | observability | Running / Error | |
| Grafana | observability | Running / Error | |
| Jaeger | observability | Running / Error | |
| Loki | observability | Running / Error | |
| OTel Collector | observability | Running / Error | |

List any pods in CrashLoopBackOff, Pending, or Error state with a brief log snippet.
