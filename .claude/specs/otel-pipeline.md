# OTel Pipeline Spec

## Collector Pipelines

### OTel Collector (single Deployment — all three pillars)

One Deployment handles metrics, traces, and logs. No DaemonSet.

```
Receivers:
  otlp:
    grpc: 0.0.0.0:4317
    http: 0.0.0.0:4318
  prometheus: (kubernetes_sd scrape configs for pods/endpoints/nodes)

Processors:
  memory_limiter: limit=1500MiB, check_interval=1s
  batch: timeout=1s, send_batch_size=1024
  resource: k8s.cluster.name=observability-cluster

Exporters:
  prometheusremotewrite: http://victoria-metrics-server.observability.svc.cluster.local:8428/api/v1/write
  otlp/jaeger: jaeger-collector.observability.svc.cluster.local:14250
  otlphttp/loki: http://loki.observability.svc.cluster.local:3100/otlp
  debug: verbosity=detailed

Pipelines:
  metrics: [otlp, prometheus] → [batch, memory_limiter, resource] → [prometheusremotewrite, debug]
  traces:  [otlp]             → [batch, memory_limiter, resource] → [otlp/jaeger, debug]
  logs:    [otlp]             → [batch, memory_limiter, resource] → [otlphttp/loki, debug]
```

> Image: `otel/opentelemetry-collector-contrib` (contrib required for prometheusremotewrite, otlphttp/loki, and prometheus receiver).
> OTel pushes metrics to VictoriaMetrics via remote_write — VictoriaMetrics does not scrape anything.

## ServiceMonitor

The Prometheus Operator `ServiceMonitor` scrapes the OTel Collector Deployment's `/metrics` endpoint:

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

**Manual OTLP env vars:**
```yaml
env:
  - name: OTEL_EXPORTER_OTLP_ENDPOINT
    value: "http://otel-collector-collector.observability.svc.cluster.local:4317"
  - name: OTEL_SERVICE_NAME
    value: "my-service"
```

## Troubleshooting Checklist

| Symptom | Check |
|---|---|
| No metrics in Prometheus | `kubectl logs deployment/opentelemetry-collector -n observability` — check prometheus exporter and scrape config |
| Collector CrashLoop | `kubectl logs deployment/opentelemetry-collector -n observability` — config YAML parse error |
| No traces in Jaeger | Verify `jaeger-collector` service exists; check `insecure: true` on OTLP exporter |
| No logs in Loki | `kubectl logs deployment/opentelemetry-collector -n observability`; verify Loki pod running; check otlphttp/loki endpoint |
| Grafana /grafana 404 | Check `serve_from_sub_path=true` and `root_url` in Grafana Helm values |
