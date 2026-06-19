# PromQL Query Reference

Queries for the observability stack. Run them in vmui at `http://localhost:30080/vmui` — set **Server URL to `http://localhost:30080/vmui`** in the settings (gear icon, top-right).

Active scrape jobs: `node-exporter`, `kubernetes-cadvisor`, `kubernetes-kubelet`, `kubernetes-pods` (Istio), `traefik`, `victoria-metrics`, `otelcol-contrib`, `jaeger`.

---

## Explore

```promql
# All active jobs and series count per job
count by (job) ({job=~".+"})
```

```promql
# Search metric names by prefix (e.g. all istio_* metrics)
{__name__=~"istio_.+"}
```

---

## Node (node-exporter)

```promql
# CPU usage % per node (idle inverted)
100 - (avg by (node) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)
```

```promql
# Memory used % per node
100 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes * 100)
```

```promql
# Disk used % per mount (excludes tmpfs and overlay)
100 - (node_filesystem_avail_bytes{fstype!~"tmpfs|overlay"} / node_filesystem_size_bytes * 100)
```

```promql
# Network receive rate per node (bytes/s)
rate(node_network_receive_bytes_total{device!~"lo|veth.+"}[5m])
```

---

## Containers / Pods (cAdvisor via kubernetes-cadvisor)

```promql
# CPU usage per container in observability namespace (cores)
rate(container_cpu_usage_seconds_total{namespace="observability", container!=""}[5m])
```

```promql
# Memory working set per pod (bytes)
container_memory_working_set_bytes{namespace="observability", container!=""}
```

```promql
# Container restarts in the last hour
increase(container_restart_count_total{namespace="observability"}[1h])
```

---

## Kubernetes (kubernetes-kubelet)

```promql
# PVC usage % per volume
kubelet_volume_stats_used_bytes / kubelet_volume_stats_capacity_bytes * 100
```

---

## Istio mesh (kubernetes-pods job, port 15020)

```promql
# Request rate between services (req/s)
sum by (source_workload, destination_workload) (
  rate(istio_requests_total[5m])
)
```

```promql
# HTTP success rate per destination (non-5xx %)
sum by (destination_workload) (rate(istio_requests_total{response_code!~"5.."}[5m]))
/
sum by (destination_workload) (rate(istio_requests_total[5m]))
* 100
```

```promql
# p99 request latency per destination (ms)
histogram_quantile(0.99,
  sum by (destination_workload, le) (
    rate(istio_request_duration_milliseconds_bucket[5m])
  )
)
```

```promql
# mTLS vs plain-text traffic split
sum by (connection_security_policy) (rate(istio_requests_total[5m]))
```

```promql
# Inbound request rate per workload (reporter=destination avoids double-counting)
sum by (destination_workload) (
  rate(istio_requests_total{reporter="destination"}[5m])
)
```

---

## Traefik ingress (traefik job)

```promql
# Total request rate through Traefik (req/s)
sum(rate(traefik_router_requests_total[5m]))
```

```promql
# Request rate per router
sum by (router) (rate(traefik_router_requests_total[5m]))
```

```promql
# 5xx error rate per router
sum by (router) (rate(traefik_router_requests_total{code=~"5.."}[5m]))
```

```promql
# p99 request duration per router (seconds)
histogram_quantile(0.99,
  sum by (router, le) (rate(traefik_router_request_duration_seconds_bucket[5m]))
)
```

---

## OTel Collector self-metrics (otelcol-contrib job)

```promql
# Metrics received per receiver (per second)
rate(otelcol_receiver_accepted_metric_points_total[5m])
```

```promql
# Export failures — should be 0
rate(otelcol_exporter_send_failed_metric_points_total[5m])
```

```promql
# Queue size (batch processor backpressure)
otelcol_exporter_queue_size
```

---

## VictoriaMetrics self-metrics (victoria-metrics job)

```promql
# Ingestion rate (samples/s)
rate(vm_rows_inserted_total[5m])
```

```promql
# Active time series
vm_active_time_series
```

```promql
# Storage size on disk (bytes)
vm_data_size_bytes
```

```promql
# Slow queries (taking > 1s)
rate(vm_slow_queries_total[5m])
```

---

## Scrape health

```promql
# All scrape targets and their up/down status
up
```

```promql
# Only DOWN targets
up == 0
```

```promql
# Scrape duration per target (seconds)
scrape_duration_seconds
```
