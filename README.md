# Kubernetes Observability Stack

A complete, production-ready Kubernetes observability stack deployed using GitOps principles with ArgoCD and Bitnami Helm charts.

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
│   (bitnami)     │    (bitnami)    │    (bitnami)    │  (bitnami)│
│   v9.5.0        │    v11.2.8      │    v1.0.0       │  v1.0.0  │
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

### 2. Deploy Everything

```powershell
# Windows PowerShell
.\bin\k8s-obs.exe quick-start
```

### 3. Access the UIs

```powershell
# Set up port forwarding
.\bin\k8s-obs.exe port-forward
```

**Access URLs:**
- **Grafana**: http://localhost:3000 (admin/hVfUTCsAbA)
- **Prometheus**: http://localhost:9090
- **Jaeger**: http://localhost:16686
- **ArgoCD**: http://localhost:8080 (admin/admin)

## 📦 Components

### Core Stack

| Component | Purpose | Helm Chart | Version | Status |
|-----------|---------|------------|---------|--------|
| **Grafana** | Visualization & dashboards | `bitnami/grafana` | 9.5.0 | ✅ Working |
| **Prometheus** | Metrics collection & alerting | `bitnami/kube-prometheus` | 11.2.8 | ✅ Working |
| **Jaeger** | Distributed tracing | `bitnami/jaeger` | 1.0.0 | ✅ Working |
| **ClickHouse** | Log storage & querying | `bitnami/clickhouse` | 1.0.0 | ✅ Working |
| **OpenTelemetry Collector** | Unified data collection | `open-telemetry/opentelemetry-collector` | 0.1.0 | ✅ Working |

### Sample Applications

- **Load Generator**: Simulates application traffic
- **Sample App**: Basic application with telemetry instrumentation

### ArgoCD Applications

All components are deployed via separate ArgoCD applications in `argocd-apps/`:

- `grafana-app.yaml` - Grafana visualization platform
- `prometheus-app.yaml` - Prometheus metrics collection
- `jaeger-app.yaml` - Jaeger distributed tracing
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
.\bin\k8s-obs.exe port-forward       # Set up port forwarding
.\bin\k8s-obs.exe status            # Check component status
.\bin\k8s-obs.exe logs              # View component logs
.\bin\k8s-obs.exe cleanup           # Remove applications
.\bin\k8s-obs.exe clean-all         # Complete cleanup
```

### Monitoring Your Applications

1. **Add OpenTelemetry SDK** to your applications
2. **Configure OTLP endpoint**: `http://opentelemetry-collector.observability.svc.cluster.local:4317`
3. **View traces** in Jaeger UI
4. **View metrics** in Grafana
5. **View logs** in ClickHouse (via Grafana)

## 🔍 Troubleshooting

### Common Issues

1. **Port Forwarding Not Working**
   - Ensure no other services are using the ports
   - Check if pods are running: `kubectl get pods -n observability`

2. **ClickHouse Connection Issues**
   - Verify ClickHouse pods are running
   - Check service endpoints: `kubectl get svc -n observability`

3. **OpenTelemetry Collector Issues**
   - Check collector logs: `kubectl logs -n observability -l app.kubernetes.io/name=opentelemetry-collector`

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
```

## 🏭 Production Considerations

### Current Setup (Development/POC)
- Single-node ClickHouse cluster
- Local storage (not distributed)
- Basic resource limits
- No high availability

### Production Recommendations
- **Storage**: Use distributed storage (e.g., EBS, Azure Disk)
- **High Availability**: Deploy multiple replicas
- **Security**: Enable TLS, RBAC, network policies
- **Monitoring**: Add monitoring for the observability stack itself
- **Backup**: Implement backup strategies for ClickHouse data

## 📊 Resource Requirements

### Minimum Requirements
- **CPU**: 4 cores
- **Memory**: 8GB RAM
- **Storage**: 20GB available space

### Component Resources
- **Grafana**: 512MB RAM, 500m CPU
- **Prometheus**: 2GB RAM, 2 CPU cores
- **Jaeger**: 1GB RAM, 1 CPU core
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
- **Prometheus Community** for the monitoring stack
- **Jaeger** for distributed tracing
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
- The old architecture documentation (ARCHITECTURE.md) is now fully integrated into this README. 