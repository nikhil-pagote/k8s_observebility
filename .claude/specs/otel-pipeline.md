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

Processors (order matters — memory_limiter must be first):
  memory_limiter: limit=1500MiB, check_interval=1s
  resource: k8s.cluster.name=observability-cluster
  batch: timeout=1s, send_batch_size=1024

Exporters:
  prometheusremotewrite: http://victoria-metrics.observability.svc.cluster.local:8428/api/v1/write
  otlp/jaeger: jaeger.observability.svc.cluster.local:4317   (OTLP gRPC — Jaeger v2 receiver)
  otlphttp/loki: http://loki.observability.svc.cluster.local:3100/otlp
  debug: verbosity=detailed

Pipelines:
  metrics: [otlp, prometheus] → [memory_limiter, resource, batch] → [prometheusremotewrite, debug]
  traces:  [otlp]             → [memory_limiter, resource, batch] → [otlp/jaeger, debug]
  logs:    [otlp]             → [memory_limiter, resource, batch] → [otlphttp/loki, debug]
```

> Image: `otel/opentelemetry-collector-contrib` (contrib required for prometheusremotewrite, otlphttp/loki, and prometheus receiver).
> OTel pushes metrics to VictoriaMetrics via remote_write — VictoriaMetrics does not scrape anything.
> `memory_limiter` must precede `batch` so it can shed load before data is buffered.

## Scrape Jobs

| Job | Target | Method |
|---|---|---|
| `victoria-metrics` | victoria-metrics.observability.svc:8428 | static |
| `opentelemetry-collector` | opentelemetry-collector.observability.svc:8888 | static |
| `pushgateway` | pushgateway.observability.svc:9091 | static, honor_labels |
| `kube-state-metrics` | kube-state-metrics.observability.svc:8080 | static |
| `node-exporter` | kubernetes_sd endpoints, ns=observability | kubernetes_sd |
| `kubernetes-cadvisor` | kubelet /metrics/cadvisor via API server proxy | kubernetes_sd nodes |
| `traefik` | kubernetes_sd pods, ns=traefik, port 8082 | kubernetes_sd |
| `kubernetes-pods` | pods with `prometheus.io/scrape: "true"` annotation | kubernetes_sd |
| `kubernetes-service-endpoints` | services with `prometheus.io/scrape: "true"` annotation | kubernetes_sd |

All scraped metrics carry `origin_prometheus=otel-collector` via `external_labels` on the remote_write exporter.

## Application Instrumentation

Send all telemetry (metrics, traces, logs) to the OTel Collector via OTLP:

```yaml
env:
  - name: OTEL_EXPORTER_OTLP_ENDPOINT
    value: "http://opentelemetry-collector.observability.svc.cluster.local:4317"
  - name: OTEL_SERVICE_NAME
    value: "my-service"
```

## Troubleshooting Checklist

| Symptom | Check |
|---|---|
| Collector CrashLoop | `kubectl logs deployment/opentelemetry-collector -n observability --previous` — config YAML parse error |
| No metrics in VictoriaMetrics | Check logs for `prometheusremotewrite` errors; verify VM pod running |
| No traces in Jaeger | Check logs for `otlp/jaeger` errors; verify `jaeger` service exists in observability ns |
| No logs in Loki | Check logs for `otlphttp/loki` errors; verify Loki pod running and `/ready` returns 200 |
| Grafana /grafana 404 | Check `serve_from_sub_path=true` and `root_url` in Grafana Helm values |
