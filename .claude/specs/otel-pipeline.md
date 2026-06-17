# OTel Pipeline Spec

## Collector Pipelines

### Metrics + Traces Collector (Deployment)

Receives OTLP from instrumented apps and fans out:

```
Receivers:
  otlp:
    grpc: 0.0.0.0:4317
    http: 0.0.0.0:4318

Processors:
  memory_limiter: limit=400MiB, spike=100MiB, check_interval=5s
  batch: timeout=10s

Exporters:
  prometheus: 0.0.0.0:8889          → scraped by Prometheus ServiceMonitor
  otlp/jaeger: jaeger-collector:4317 → Jaeger trace storage
  clickhouse: clickhouse:8123         → ClickHouse log/event storage
  debug: verbosity=basic

Pipelines:
  metrics: otlp → [memory_limiter, batch] → [prometheus, debug]
  traces:  otlp → [memory_limiter, batch] → [otlp/jaeger, debug]
```

### Log Collector (DaemonSet)

Tails pod logs per node and ships to ClickHouse:

```
Receivers:
  filelog:
    include: /var/log/pods/*/*/*.log
    operators: [container parser]   # extracts k8s.pod.name, k8s.namespace.name

Processors:
  k8sattributes: enriches with pod/namespace/node metadata (requires RBAC)
  batch: timeout=5s

Exporters:
  clickhouse: http://clickhouse.observability.svc.cluster.local:8123

Pipeline:
  logs: filelog → [k8sattributes, batch] → clickhouse
```

## ServiceMonitor

The Prometheus Operator `ServiceMonitor` scrapes the OTel Collector's `/metrics` endpoint:

```yaml
spec:
  selector:
    matchLabels:
      app.kubernetes.io/name: otel-collector-collector
  endpoints:
    - port: prometheus
      path: /metrics
      interval: 30s
```

Label `release: kube-prometheus-stack` must match the Helm release name for Prometheus to pick it up.

## Application Instrumentation

**Auto-instrumentation** (OTel Operator target):
```yaml
# Add annotation to Deployment pod template
instrumentation.opentelemetry.io/inject-python: "true"
```

**Manual OTLP env vars:**
```yaml
env:
  - name: OTEL_EXPORTER_OTLP_ENDPOINT
    value: "http://opentelemetry-collector.observability.svc.cluster.local:4317"
  - name: OTEL_SERVICE_NAME
    value: "my-service"
```

## RBAC Requirements (DaemonSet)

The log collector needs a `ClusterRole` with:
```yaml
rules:
  - apiGroups: [""]
    resources: ["pods", "namespaces", "nodes"]
    verbs: ["get", "list", "watch"]
```

## Troubleshooting Checklist

| Symptom | Check |
|---|---|
| No metrics in Prometheus | `kubectl get servicemonitor -n observability` — label `release:` must match |
| Collector CrashLoop | `kubectl logs deployment/opentelemetry-collector -n observability` — config YAML parse error |
| No traces in Jaeger | Verify `jaeger-collector` service exists; check `insecure: true` on OTLP exporter |
| No logs in ClickHouse | Check DaemonSet RBAC; verify ClickHouse pod running |
| Grafana /grafana 404 | Check `serve_from_sub_path=true` and `root_url` in Grafana Helm values |
