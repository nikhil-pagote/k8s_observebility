# Kubernetes Observability Stack

A complete, production-ready Kubernetes observability stack deployed using GitOps principles with ArgoCD and Helm charts.

## 🎯 Overview

This project provides a comprehensive observability solution for Kubernetes clusters with:

- **📊 Metrics**: Prometheus + Grafana for monitoring and visualization
- **🔍 Traces**: Jaeger for distributed tracing
- **📝 Logs**: ClickHouse for log storage and querying
- **📡 Data Collection**: OpenTelemetry Collector for unified telemetry collection
- **🚀 GitOps**: ArgoCD for declarative deployment and management

## 🏗️ Architecture

### Data Flow Architecture

```
       ┌─────────────────┐
       │   Applications  │
       │   (Sample Apps) │
       └─────────────────┘
                │
                ▼
       ┌──────────────────┐
       │ OpenTelemetry    │
       │   Collector      │
       └──────────────────┘
                │
    ┌───────────┼────────────┐
    │           │            │
    ▼           ▼            ▼
┌─────────┐ ┌─────────┐ ┌─────────┐
│Prometheus│ │  Jaeger │ │ClickHouse│
│(Metrics) │ │(Traces) │ │ (Logs)  │
└─────────┘ └─────────┘ └─────────┘
    │            │           │
    └────────────┼───────────┘
                 │
                 ▼
         ┌───────────────┐
         │   Grafana     │
         │(Visualization)│
         └───────────────┘
```

### Deployment Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    ArgoCD Applications                          │
├─────────────────┬─────────────────┬─────────────────┬───────────┤
│   Grafana App   │  Prometheus App │   Jaeger App    │ClickHouse │
│   (bitnami)     │    (bitnami)    │  (jaegertracing)│  (bitnami)│
│   v12.0.8       │    v2.1.10      │    v0.71.0      │  v1.0.0  │
└─────────────────┴─────────────────┴─────────────────┴───────────┘
         │                │                │              │
         ▼                ▼                ▼              ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────┐ ┌─────────────┐
│    Grafana      │ │   Prometheus    │ │   Jaeger    │ │  ClickHouse │
│   (Port 3000)   │ │   (Port 9090)   │ │ (Port 16686)│ │ (Port 8123) │
│   Namespace:    │ │   Namespace:    │ │ Namespace:  │ │ Namespace:  │
│  observability  │ │  observability  │ │observability│ │observability│
└─────────────────┘ └─────────────────┘ └─────────────┘ └─────────────┘
         │                │                │              │
         └────────────────┼────────────────┼──────────────┘
                          │                │
                          ▼                ▼
                   ┌─────────────────────────────────┐
                   │      OpenTelemetry Collector    │
                   │        (Data Routing)           │
                   │      Namespace: observability   │
                   └─────────────────────────────────┘
                                   │
                                   ▼
                          ┌─────────────────┐
                          │   Applications  │
                          │  (Sample Apps)  │
                          │  Namespace:     │
                          │  observability  │
                          └─────────────────┘
```

## 🚀 Quick Start

### Prerequisites

- **Docker Desktop** with Kubernetes enabled
- **kubectl** configured
- **Helm** v3+
- **PowerShell** (Windows) or **Bash** (Linux/Mac)

### 1. Clone and Setup

```bash
git clone <repository-url>
cd k8s_observebility
```

### 2. Build Deployment Scripts

```powershell
# Windows PowerShell
./build-scripts.ps1
```

### 3. Deploy Everything

```powershell
# Windows PowerShell
.\bin\k8s-obs.exe quick-start
```

### 4. Access the UIs

```powershell
# Set up Traefik ingress for observability stack
.\bin\k8s-obs.exe setup-ingress
```

**Access URLs:**
- **Traefik Dashboard**: http://localhost:30080/traefik (admin/admin)
- **Grafana**: http://localhost:30080/grafana (admin/admin123)
- **Prometheus**: http://localhost:30080/prometheus
- **Jaeger**: http://localhost:30080/jaeger
- **ClickHouse**: http://localhost:30080/clickhouse
- **ArgoCD**: http://localhost:30080/argocd (admin/admin)

> **Note**: All services are accessible through Traefik Ingress Controller on port 30080 using path-based routing. No port-forwarding required!

## 📊 Grafana Configuration

### Auto-Provisioned Data Sources

The Grafana deployment is configured to automatically provision:

- **Prometheus**: Primary metrics data source
  - URL: `http://prometheus-server.observability.svc.cluster.local:80`
  - Default data source for dashboards

### Auto-Provisioned Dashboards

The following dashboards are automatically imported:

- **Kubernetes Cluster Overview** (ID: 7249)
- **Node Exporter** (ID: 1860) 
- **Prometheus Overview** (ID: 3662)

### Manual Data Source Configuration

If you need to add additional data sources manually:

1. **Access Grafana**: http://localhost:3000
2. **Login**: admin / admin123
3. **Go to**: Configuration (⚙️) → Data Sources
4. **Click**: "Add data source"

#### Adding Prometheus (if not auto-provisioned)

1. Select **Prometheus**
2. **URL**: `http://prometheus-server.observability.svc.cluster.local:80`
3. **Access**: Server (default)
4. **Save & Test**

#### Adding ClickHouse (for logs)

1. Install ClickHouse plugin first:
   - Go to Configuration (⚙️) → Plugins
   - Search for "ClickHouse"
   - Install the plugin
2. Add ClickHouse data source:
   - **Type**: ClickHouse
   - **URL**: `http://clickhouse.observability.svc.cluster.local:8123`
   - **Database**: default
   - **Username**: default
   - **Password**: `clickhouse123` (get from secret)

### Manual Dashboard Import

To import additional dashboards:

1. **Access Grafana**: http://localhost:3000
2. **Go to**: Dashboards → Import
3. **Choose method**:
   - **Upload JSON**: Upload dashboard JSON file
   - **Import from Grafana.com**: Enter dashboard ID
   - **Paste JSON**: Paste dashboard configuration

#### Popular Dashboard IDs

- **Kubernetes Cluster**: 7249
- **Node Exporter**: 1860
- **Prometheus**: 3662
- **Jaeger**: 13332
- **OpenTelemetry Collector**: 13332

#### Importing via Grafana.com

1. Click **Import**
2. Enter **Grafana.com Dashboard URL** or **Dashboard ID**
3. Select **Data Source** (Prometheus for metrics)
4. Click **Load**
5. Review and click **Import**

### Dashboard Configuration

#### Creating Custom Dashboards

1. **Go to**: Dashboards → New Dashboard
2. **Add Panel**: Click "Add panel"
3. **Query**: Use PromQL for Prometheus data
4. **Visualization**: Choose chart type
5. **Save**: Name and save dashboard

#### Example PromQL Queries

```promql
# CPU Usage
rate(container_cpu_usage_seconds_total{container!=""}[5m])

# Memory Usage
container_memory_usage_bytes{container!=""}

# Pod Status
kube_pod_status_phase

# OpenTelemetry Collector Metrics
otelcol_processor_batch_batch_send_size
```

## 📦 Components

### Core Stack

| Component | Purpose | Helm Chart | Version | Status |
|-----------|---------|------------|---------|--------|
| **Grafana** | Visualization & dashboards | `bitnami/grafana` | 12.0.8 | ✅ Working |
| **Prometheus** | Metrics collection & alerting | `bitnami/prometheus` | 2.1.10 | ✅ Working |
| **Jaeger** | Distributed tracing | `jaegertracing/jaeger` | 0.71.0 | ✅ Working |
| **ClickHouse** | Log storage & querying | `bitnami/clickhouse` | 1.0.0 | ✅ Working |
| **OpenTelemetry Collector** | Unified data collection | `open-telemetry/opentelemetry-collector` | 0.60.0 | ✅ Working |

### Key Features

- **🎯 Unified Data Collection**: OpenTelemetry Collector replaces kube-state-metrics and node-exporter
- **🔍 In-Memory Jaeger**: Uses official Jaeger chart with in-memory storage (no Cassandra dependencies)
- **📊 Optimized Prometheus**: Disabled redundant components since OpenTelemetry handles metrics collection
- **🚀 Smart Port-Forwarding**: Separate port-forward management for ArgoCD and observability stack
- **✅ Improved Error Reporting**: Enhanced port-forwarding with accurate success/failure reporting

### Sample Applications

- **Load Generator**: Simulates application traffic
- **Sample App**: Basic application with telemetry instrumentation

### ArgoCD Applications

All components are deployed via separate ArgoCD applications in `argocd-apps/`:

- `grafana-app.yaml` - Grafana visualization platform (with auto-provisioned data sources)
- `prometheus-app.yaml` - Prometheus metrics collection (optimized for OpenTelemetry)
- `jaeger-app.yaml` - Jaeger distributed tracing (in-memory storage)
- `clickhouse-app.yaml` - ClickHouse log storage
- `opentelemetry-collector-app.yaml` - Unified data collection

### Data Flow

1. **Applications** send telemetry data to OpenTelemetry Collector
2. **OpenTelemetry Collector** processes and routes data:
   - **Metrics** → Prometheus
   - **Traces** → Jaeger
   - **Logs** → ClickHouse
3. **Grafana** visualizes all data sources

## 🎮 Usage

### PowerShell Commands

```powershell
# Quick setup
.\bin\k8s-obs.exe quick-start

# Individual commands
.\bin\k8s-obs.exe setup-cluster      # Create Kind cluster
.\bin\k8s-obs.exe deploy-argocd      # Deploy ArgoCD
.\bin\k8s-obs.exe deploy-stack       # Deploy observability stack
.\bin\k8s-obs.exe deploy-sample-apps # Deploy sample applications
.\bin\k8s-obs.exe setup-ingress      # Set up Traefik ingress access
.\bin\k8s-obs.exe status            # Check component status
.\bin\k8s-obs.exe logs              # View component logs
.\bin\k8s-obs.exe get-urls          # Get service URLs and credentials
.\bin\k8s-obs.exe cleanup           # Remove applications
.\bin\k8s-obs.exe clean-all         # Complete cleanup
```

### Ingress Access

The project uses Traefik Ingress Controller for external access:

- **Traefik Ingress**: Lightweight, modern ingress controller with dashboard
- **Single Port Access**: All services accessible on port 30080
- **Path-based Routing**: Each service accessible via path (e.g., /grafana, /prometheus)
- **No Port Forwarding**: Eliminates the need for multiple kubectl port-forward commands
- **Built-in Metrics**: Traefik provides its own metrics for monitoring

### Monitoring Your Applications

1. **Add OpenTelemetry SDK** to your applications
2. **Configure OTLP endpoint**: `http://opentelemetry-collector.observability.svc.cluster.local:4317`
3. **View traces** in Jaeger UI
4. **View metrics** in Grafana
5. **View logs** in ClickHouse (via Grafana)

## 🔍 Troubleshooting

### Common Issues

1. **Ingress Access Not Working**
   - Check if Traefik is running: `kubectl get pods -n traefik`
   - Verify ingress resources: `kubectl get ingress -n observability`
   - Ensure hosts file entries are added: `k8s-obs setup-ingress`
   - Check if port 30080 is accessible: `curl http://localhost:30080`
   - Verify path-based routing: `curl http://localhost:30080/grafana` should return Grafana

2. **Jaeger "Progressing" Status**
   - This is normal for in-memory Jaeger deployments
   - The application is working correctly despite the status

3. **Prometheus "Progressing" Status**
   - Expected when kube-state-metrics and node-exporter are disabled
   - OpenTelemetry Collector provides the same functionality
   - Application is working correctly

4. **ClickHouse Connection Issues**
   - Verify ClickHouse pods are running
   - Check service endpoints: `kubectl get svc -n observability`
   - Default password: `clickhouse123`

5. **OpenTelemetry Collector Issues**
   - Check collector logs: `kubectl logs -n observability -l app.kubernetes.io/name=opentelemetry-collector`

6. **Grafana Data Sources Not Showing**
   - Check if auto-provisioning worked: Configuration → Data Sources
   - Verify Prometheus service is accessible
   - Check Grafana logs for provisioning errors

7. **Helm Schema Validation Errors**
   - Fixed: OpenTelemetry service.ports configuration
   - Fixed: Prometheus extraScrapeConfigs array format

### Useful Commands

```bash
# Check all components
kubectl get pods -n observability

# Check ArgoCD applications
kubectl get applications -n argocd

# View component logs
kubectl logs -n observability <pod-name>

# Check services
kubectl get svc -n observability

# Check Traefik ingress
kubectl get pods -n traefik
kubectl get ingress -n observability

# Get Grafana password
kubectl get secret -n observability grafana-admin -o jsonpath='{.data.GF_SECURITY_ADMIN_PASSWORD}' | base64 -d

# Get ClickHouse password
kubectl get secret -n observability clickhouse -o jsonpath='{.data.admin-password}' | base64 -d
```

## 🏭 Production Considerations

### Current Setup (Development/POC)
- Single-node ClickHouse cluster
- Local storage (not distributed)
- Basic resource limits
- No high availability
- In-memory Jaeger storage

### Production Recommendations
- **Storage**: Use distributed storage (e.g., EBS, Azure Disk)
- **High Availability**: Deploy multiple replicas
- **Security**: Enable TLS, RBAC, network policies
- **Monitoring**: Add monitoring for the observability stack itself
- **Backup**: Implement backup strategies for ClickHouse data
- **Jaeger Storage**: Use persistent storage (Elasticsearch/Cassandra) for production

## 📊 Resource Requirements

### Minimum Requirements
- **CPU**: 4 cores
- **Memory**: 8GB RAM
- **Storage**: 20GB available space

### Component Resources
- **Grafana**: 512MB RAM, 500m CPU
- **Prometheus**: 2GB RAM, 1 CPU core
- **Jaeger**: 512MB RAM, 500m CPU
- **ClickHouse**: 1GB RAM, 1 CPU core
- **OpenTelemetry Collector**: 512MB RAM, 500m CPU

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- **Bitnami** for production-ready Helm charts
- **Jaeger** for distributed tracing and official Helm chart
- **Prometheus Community** for the monitoring stack
- **ClickHouse** for high-performance log storage
- **OpenTelemetry** for unified observability
- **ArgoCD** for GitOps deployment

## 🛠️ Building Deployment Binaries

All deployment binaries (such as `k8s-obs.exe`, `setup_kind_cluster.exe`, etc.) are built using the provided PowerShell script:

```powershell
./build-scripts.ps1
```

- This script uses Docker to cross-compile the Rust binaries for Windows.
- The `bin/` directory is used for all build outputs and is not tracked in version control.
- The build process **requires** `Dockerfile.build` in the repository, which defines the `rust-builder` image used by the script. 