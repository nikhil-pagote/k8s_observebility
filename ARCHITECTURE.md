# Observability Stack Architecture

## Overview
This observability stack provides a comprehensive solution for Kubernetes monitoring with:
- **OpenTelemetry Collector**: Unified data collection from all sources
- **Prometheus**: Metrics storage and querying
- **Grafana**: Visualization and dashboards
- **Jaeger**: Distributed tracing
- **ClickHouse**: High-performance log storage
- **ArgoCD**: GitOps deployment and management

## Data Flow
```
Applications → OpenTelemetry Collector → [Prometheus, Jaeger, ClickHouse] → Grafana
     ↓              ↓                        ↓         ↓         ↓         ↓
   OTLP         Processing              Metrics   Traces    Logs   Dashboards
   Metrics      & Routing              Storage   Storage   Storage & Alerts
   Traces       Kubernetes
   Logs         Services
```

## Component Responsibilities

### OpenTelemetry Collector
**Primary Role**: Unified Data Collection
- **OTLP Receiver**: Collects traces, metrics, logs from applications
- **Prometheus Receiver**: Scrapes Kubernetes resources
  - Kubernetes pods with Prometheus annotations
  - Kubernetes service endpoints
  - Kubernetes nodes (system metrics)
  - CoreDNS metrics
  - Grafana metrics
  - Alertmanager metrics
- **Processors**: Batch, memory limiter, resource labeling
- **Exporters**: 
  - **Prometheus**: Exports metrics to Prometheus
  - **Jaeger**: Exports traces to Jaeger
  - **ClickHouse**: Exports logs to ClickHouse

### Prometheus
**Primary Role**: Metrics Storage & Querying
- **Storage**: Time-series database for metrics
- **Query Engine**: PromQL queries for dashboards and alerts
- **Self-Monitoring**: Monitors its own health

### Jaeger
**Primary Role**: Distributed Tracing
- **Trace Storage**: Stores distributed traces
- **Query Service**: Provides trace querying capabilities
- **UI**: Web interface for trace visualization

### ClickHouse
**Primary Role**: Log Storage & Querying
- **High-Performance**: Columnar database optimized for analytics
- **Log Storage**: Stores application and system logs
- **Query Engine**: SQL-like queries for log analysis

### Grafana
**Primary Role**: Visualization
- **Dashboards**: Kubernetes, Prometheus, Jaeger, ClickHouse dashboards
- **Data Sources**: Prometheus, Jaeger, ClickHouse
- **Alerts**: Based on Prometheus queries

## Benefits of This Architecture

### 1. **Unified Observability**
- Single OpenTelemetry Collector for all telemetry data
- Consistent data format and labeling across metrics, traces, and logs
- Better correlation between different data types

### 2. **High Performance**
- ClickHouse provides fast log querying and storage
- Prometheus optimized for time-series metrics
- Jaeger specialized for trace analysis

### 3. **Scalability**
- Each component can scale independently
- ClickHouse supports horizontal scaling
- Prometheus can be federated or use remote storage

### 4. **GitOps Management**
- ArgoCD manages all deployments
- Declarative configuration
- Version-controlled deployments

## Configuration Details

### OpenTelemetry Collector Configuration
- **Receivers**: OTLP, Prometheus
- **Processors**: Batch, memory limiter, resource labeling
- **Exporters**: Prometheus, Jaeger, ClickHouse
- **Service Pipelines**: Metrics, Traces, Logs

### Prometheus Configuration
- **Storage**: 30Gi PVC for metrics retention
- **Scraping**: Via OpenTelemetry Collector
- **Rules**: Kubernetes and application monitoring rules

### ClickHouse Configuration
- **Storage**: 20Gi PVC for log retention
- **Replicas**: Single node (development setup)
- **Compression**: Optimized for log storage

### Jaeger Configuration
- **Storage**: In-memory (development setup)
- **UI**: Web interface for trace visualization
- **Sampling**: 100% sampling rate

## Monitoring Stack

### Available Data Types
- **Metrics**: Infrastructure, Kubernetes, application metrics
- **Traces**: Distributed traces with service dependencies
- **Logs**: Application and system logs with structured data

### Available Dashboards
- **Kubernetes**: Cluster health, pods, nodes
- **Prometheus**: Server health and performance
- **Jaeger**: Trace visualization and analysis
- **ClickHouse**: Log analysis and querying

## Production Considerations

### Current Setup (Development/POC)
- Single-node ClickHouse cluster
- In-memory Jaeger storage
- Local storage (not distributed)
- Basic resource limits

### Production Recommendations
- **ClickHouse**: Multi-node cluster with replication
- **Jaeger**: Persistent storage (Elasticsearch/Cassandra)
- **Prometheus**: Remote storage (Thanos/Cortex)
- **High Availability**: Multiple replicas for all components
- **Security**: TLS, RBAC, network policies
- **Backup**: Automated backup strategies

## Troubleshooting

### Common Issues
1. **No Data in Grafana**: Check OpenTelemetry Collector logs
2. **ClickHouse Connection Issues**: Verify ClickHouse pods are running
3. **Jaeger UI Not Loading**: Check Jaeger service endpoints
4. **High Resource Usage**: Monitor component metrics

### Useful Commands
```bash
# Check all components
kubectl get pods -n observability

# View OpenTelemetry logs
kubectl logs -n observability -l app.kubernetes.io/name=opentelemetry-collector

# Check ClickHouse status
kubectl logs -n observability -l app.kubernetes.io/name=clickhouse

# Check Jaeger status
kubectl logs -n observability -l app.kubernetes.io/name=jaeger

# Port forwarding for UIs
kubectl port-forward -n observability svc/prometheus-stack-grafana 3000:80
kubectl port-forward -n observability svc/jaeger-query 16686:80
```

## Future Enhancements
1. **High Availability**: Multi-node deployments
2. **Advanced Alerting**: More sophisticated alert rules
3. **Custom Dashboards**: Application-specific visualizations
4. **Security**: TLS, authentication, authorization
5. **Backup & Recovery**: Automated data protection 