# Troubleshooting Guide

Covers issues specific to this stack: Kind + Podman, ArgoCD GitOps, OTel-first ingestion, VictoriaMetrics, Loki, Jaeger, Traefik.

---

## Cluster / Kind

### Cluster not reachable after host reboot

```bash
source .envrc
systemctl --user start podman.socket
kind export kubeconfig --name observability-cluster
kubectl get nodes
```

Podman socket does not auto-start on reboot by default. Enable it permanently:

```bash
systemctl --user enable podman.socket
```

### Nodes stuck in NotReady

```bash
kubectl get nodes
kubectl describe node <node-name>
```

Usually CNI not ready yet — wait 30s and retry. If persistent, restart the cluster:

```bash
/kind-cluster restart
```

---

## ArgoCD

### Apps stuck in OutOfSync / Unknown

Check the error:

```bash
kubectl describe application <app-name> -n argocd | grep -A5 "Message:"
```

Common causes:

| Message | Fix |
|---|---|
| `app path does not exist` | `targetRevision` points to wrong branch — check `argocd-apps/<app>/app.yaml` |
| `failed to generate manifest` | Bad Helm values — `helm template argocd-apps/<app>/chart -f argocd-apps/<app>/values/values.yaml` |
| `namespace not found` | `kubectl create namespace <ns>` then re-apply |

Force a hard refresh and sync:

```bash
kubectl annotate application <app> -n argocd argocd.argoproj.io/refresh="hard" --overwrite
kubectl patch application <app> -n argocd --type merge -p '{"operation":{"sync":{"revision":"otel-first"}}}'
```

### ArgoCD UI not accessible at /argocd

Verify the server is in insecure mode:

```bash
kubectl get configmap argocd-cmd-params-cm -n argocd -o jsonpath='{.data}'
```

Should contain `server.insecure: "true"` and `server.rootpath: "/argocd"`. If missing, redeploy:

```bash
helm upgrade --install argocd argocd-apps/argocd/chart \
  --namespace argocd \
  -f argocd-apps/argocd/values/values.yaml \
  --wait
```

---

## OTel Collector

### CrashLoopBackOff

```bash
kubectl logs -n observability deployment/opentelemetry-collector --previous
```

Usually a config YAML error. Validate locally:

```bash
helm template argocd-apps/opentelemetry-collector/chart \
  -f argocd-apps/opentelemetry-collector/values/values.yaml | grep -A200 "relay:"
```

### Metrics not reaching VictoriaMetrics

```bash
kubectl logs -n observability deployment/opentelemetry-collector | grep -E "ERROR|Exporting failed"
```

| Error | Cause | Fix |
|---|---|---|
| `context deadline exceeded` | VictoriaMetrics service unreachable | Check `kubectl get svc victoria-metrics -n observability` — must not be headless (ClusterIP: None) |
| `connection refused` | VM pod down | `kubectl get pod victoria-metrics-0 -n observability` |
| `permanent error 400` | Bad metric format | Check OTel config for invalid label names |

Confirm the remote_write endpoint is reachable from the collector pod:

```bash
kubectl exec -n observability deployment/opentelemetry-collector -- \
  curl -s http://victoria-metrics.observability.svc.cluster.local:8428/health
```

### Scrape targets not appearing

Check which targets are active:

```bash
kubectl port-forward -n observability svc/victoria-metrics 8428:8428 &
curl -s http://localhost:8428/api/v1/query?query=up | python3 -c "
import sys,json
for r in json.load(sys.stdin)['data']['result']:
    print(r['metric'].get('job'), r['value'][1])"
```

---

## VictoriaMetrics

### No data / empty queries

1. Confirm the service has a real ClusterIP (not None):

```bash
kubectl get svc victoria-metrics -n observability
```

If `CLUSTER-IP` is `None`, delete the service and force ArgoCD to recreate it (requires `clusterIP: ""` in values).

2. Check data exists:

```bash
kubectl port-forward -n observability svc/victoria-metrics 8428:8428 &
curl -s "http://localhost:8428/api/v1/label/__name__/values" | python3 -c "
import sys,json; names=json.load(sys.stdin)['data']; print(len(names), 'metric names')"
```

3. Check OTel is pushing (no errors in logs, see above).

### /vmui not loading at localhost:30080/vmui

Confirm the ingress service name matches:

```bash
kubectl get svc -n observability | grep victoria
```

The service must be named `victoria-metrics`. The ingress backend in `argocd-apps/observability-ingress.yaml` must match.

---

## Grafana

### Datasource error: "no such host"

Wrong service DNS in datasource URL. The correct URL is:

```
http://victoria-metrics.observability.svc.cluster.local:8428
```

Check via API:

```bash
kubectl exec -n observability deployment/grafana -- \
  curl -s "http://admin:admin123@localhost:3000/api/datasources" | python3 -c "
import sys,json; [print(d['name'], d['url']) for d in json.load(sys.stdin)]"
```

Delete stale datasources by ID:

```bash
kubectl exec -n observability deployment/grafana -- \
  curl -s -X DELETE "http://admin:admin123@localhost:3000/api/datasources/<id>"
```

### Dashboard shows "No data"

1. Check datasource health:

```bash
kubectl exec -n observability deployment/grafana -- \
  curl -s "http://admin:admin123@localhost:3000/api/datasources/uid/<uid>/health"
```

2. Test a query through Grafana proxy:

```bash
kubectl exec -n observability deployment/grafana -- \
  curl -s "http://admin:admin123@localhost:3000/api/datasources/proxy/uid/<uid>/api/v1/query?query=up"
```

3. Check template variables — open the dashboard and confirm `$job` and `$node` dropdowns are populated.

### CSS/JS broken at /grafana

`serve_from_sub_path` not set. Check `argocd-apps/grafana/values/values.yaml`:

```yaml
grafana.ini:
  server:
    root_url: http://localhost:30080/grafana
    serve_from_sub_path: true
```

---

## Jaeger

### "No services" in Jaeger UI

No traces have been sent yet — Jaeger in-memory storage starts empty. Real data appears only when an instrumented app sends traces via OTLP to the collector (`opentelemetry-collector.observability.svc.cluster.local:4317`).

The collector exports traces to `jaeger.observability.svc.cluster.local:4317` (OTLP gRPC). Jaeger v2 accepts traces on its OTLP receiver, not the legacy 14250 port.

### Jaeger UI at /jaeger returns 404 or blank

Check base_path config in Jaeger values:

```yaml
userconfig:
  extensions:
    jaeger_query:
      base_path: /jaeger
```

Verify pod is running and the service exists:

```bash
kubectl get pod,svc -n observability | grep jaeger
```

---

## Loki

### No logs appearing

OTel sends logs via OTLP HTTP. Check the endpoint is reachable:

```bash
kubectl exec -n observability deployment/opentelemetry-collector -- \
  curl -s http://loki.observability.svc.cluster.local:3100/ready
```

Check OTel logs for Loki export errors:

```bash
kubectl logs -n observability deployment/opentelemetry-collector | grep loki
```

---

## Ingress / Traefik

### Path returns 404

Check Traefik picked up the ingress:

```bash
kubectl get ingress -n observability
kubectl get ingressroute -A
kubectl logs -n traefik deployment/traefik | grep -i error
```

### /traefik dashboard not loading

The `replacePathRegex` middleware rewrites `/traefik` → `/dashboard/`. If the dashboard JS/CSS breaks, the rewrite may not be covering sub-paths. Access directly via port-forward to isolate:

```bash
kubectl port-forward -n traefik svc/traefik 9000:9000
open http://localhost:9000/dashboard/
```

---

## Useful one-liners

```bash
# All pods across all namespaces
kubectl get pods --all-namespaces

# Tail OTel collector logs
kubectl logs -n observability deployment/opentelemetry-collector -f

# All ArgoCD app statuses
kubectl get applications -n argocd

# All ingress routes
kubectl get ingress,ingressroute -A

# Query VictoriaMetrics for all active jobs
kubectl port-forward -n observability svc/victoria-metrics 8428:8428 &
curl -s "http://localhost:8428/api/v1/query?query=up" | python3 -c "
import sys,json
for r in json.load(sys.stdin)['data']['result']:
    print(r['metric'].get('job','?'), '→', 'UP' if r['value'][1]=='1' else 'DOWN')"
```
