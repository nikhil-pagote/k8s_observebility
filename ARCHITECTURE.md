# Observability Stack Architecture

## Overview
This observability stack uses a **clean separation of concerns** where:
- **OpenTelemetry Collector**: Primary data collection from all sources
- **Prometheus**: Storage and querying of metrics only
- **Grafana**: Visualization and dashboards
- **ArgoCD**: GitOps deployment and management

## Data Flow
```
Applications → OpenTelemetry Collector → Prometheus → Grafana
     ↓              ↓                        ↓         ↓
   OTLP         Scraping                Storage    Dashboards
   Metrics      Kubernetes              & Query    & Alerts
   Traces       Services                Engine
   Logs         Nodes
```

## Component Responsibilities

### OpenTelemetry Collector
**Primary Role**: Data Collection
- **OTLP Receiver**: Collects traces, metrics, logs from applications
- **Prometheus Receiver**: Scrapes all Kubernetes resources
  - Kubernetes pods with Prometheus annotations
  - Kubernetes service endpoints
  - Kubernetes nodes (system metrics)
  - CoreDNS metrics
  - Grafana metrics
  - Alertmanager metrics
- **Processors**: Batch, memory limiter, resource labeling
- **Prometheus Exporter**: Exports all collected metrics to Prometheus

### Prometheus
**Primary Role**: Storage & Querying
- **No Scraping**: All data comes from OpenTelemetry Collector
- **Storage**: Time-series database for metrics
- **Query Engine**: PromQL queries for dashboards and alerts
- **Self-Monitoring**: Only monitors its own health

### Grafana
**Primary Role**: Visualization
- **Dashboards**: Kubernetes, Prometheus, custom dashboards
- **Data Source**: Prometheus
- **Alerts**: Based on Prometheus queries

## Benefits of This Architecture

### 1. **Single Source of Truth**
- OpenTelemetry Collector is the only component collecting data
- No duplicate scraping or data collection
- Consistent data format and labeling

### 2. **Resource Optimization**
- Reduced CPU usage (no duplicate scraping)
- Lower memory consumption
- Less network traffic
- Smaller storage footprint

### 3. **Simplified Management**
- Fewer ServiceMonitors to maintain
- Centralized collection configuration
- Easier troubleshooting

### 4. **Enhanced Capabilities**
- Unified collection pipeline for metrics, traces, and logs
- Better data correlation
- Future-ready for advanced observability features

## Configuration Details

### OpenTelemetry Collector Scraping Jobs
1. **kubernetes-pods**: Pods with `prometheus.io/scrape=true`
2. **kubernetes-service-endpoints**: Services with Prometheus annotations
3. **kubernetes-nodes**: Node system metrics via kubelet
4. **coredns**: DNS performance metrics
5. **grafana**: Grafana application metrics
6. **alertmanager**: Alert processing metrics

### Prometheus Configuration
- **No ServiceMonitors**: All collection handled by OpenTelemetry
- **No kube-state-metrics**: OpenTelemetry handles Kubernetes metrics
- **Minimal Rules**: Only Prometheus self-monitoring rules
- **Storage**: 30Gi PVC for metrics retention

### Grafana Dashboards
- **Kubernetes Cluster**: Overall cluster health
- **Kubernetes Pods**: Pod-level metrics
- **Kubernetes Nodes**: Node system metrics
- **Prometheus**: Prometheus server health
- **Custom**: Sample application dashboard

## Monitoring Stack

### Available Metrics
- **Infrastructure**: CPU, memory, disk, network (via OpenTelemetry)
- **Kubernetes**: Pods, services, deployments, nodes
- **Applications**: HTTP metrics, custom business metrics
- **System**: CoreDNS, Grafana, Alertmanager performance

### Available Dashboards
- **Kubernetes**: 8 comprehensive dashboards
- **Prometheus**: 3 monitoring dashboards
- **Custom**: 1 sample application dashboard

## Future Enhancements
1. **Jaeger**: Distributed tracing visualization
2. **Loki**: Log aggregation and querying
3. **Custom Application Metrics**: Business-specific dashboards
4. **Advanced Alerting**: More sophisticated alert rules
5. **Long-term Storage**: Thanos or Cortex integration

## Troubleshooting

### Common Issues
1. **No Metrics in Grafana**: Check OpenTelemetry Collector logs
2. **Missing Data**: Verify scraping jobs in OpenTelemetry config
3. **High Resource Usage**: Monitor OpenTelemetry Collector metrics
4. **Sync Issues**: Check ArgoCD application status

### Useful Commands
```bash
# Check OpenTelemetry Collector status
kubectl get pods -n observability -l app.kubernetes.io/name=opentelemetry-collector

# View OpenTelemetry logs
kubectl logs -n observability -l app.kubernetes.io/name=opentelemetry-collector

# Check Prometheus targets
kubectl port-forward -n observability svc/prometheus-operated 9090:9090
# Then visit http://localhost:9090/targets

# Check Grafana dashboards
kubectl port-forward -n observability svc/prometheus-stack-poc-grafana 3000:80
# Then visit http://localhost:3000
``` 