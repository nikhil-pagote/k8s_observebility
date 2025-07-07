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

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Applications  │───▶│ OpenTelemetry    │───▶│   ClickHouse    │
│   (Sample Apps) │    │   Collector      │    │   (Logs)        │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │
                                ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Prometheus    │◀───│ OpenTelemetry    │───▶│     Jaeger      │
│   (Metrics)     │    │   Collector      │    │   (Traces)      │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌─────────────────┐
                       │    Grafana      │
                       │ (Visualization) │
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
.\deploy.ps1 quick-start
```

### 3. Access the UIs

```powershell
# Set up port forwarding
.\deploy.ps1 port-forward
```

**Access URLs:**
- **Grafana**: http://localhost:3000 (admin/admin123)
- **Prometheus**: http://localhost:9090
- **Jaeger**: http://localhost:16686
- **ArgoCD**: http://localhost:8080 (admin/admin)

## 📦 Components

### Core Stack

| Component | Purpose | Helm Chart | Status |
|-----------|---------|------------|--------|
| **Prometheus Stack** | Metrics collection & alerting | `prometheus-community/kube-prometheus-stack` | ✅ Working |
| **Jaeger** | Distributed tracing | `jaegertracing/jaeger` | ✅ Working |
| **ClickHouse** | Log storage & querying | `bitnami/clickhouse` | ✅ Working |
| **OpenTelemetry Collector** | Unified data collection | `open-telemetry/opentelemetry-collector` | ✅ Working |

### Sample Applications

- **Load Generator**: Simulates application traffic
- **Sample App**: Basic application with telemetry instrumentation

## 🔧 Configuration

### ArgoCD Applications

All components are deployed via ArgoCD applications in `argocd-apps/`:

- `prometheus-stack-app.yaml` - Prometheus + Grafana
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
.\deploy.ps1 quick-start

# Individual commands
.\deploy.ps1 setup-cluster      # Create Kind cluster
.\deploy.ps1 deploy-argocd      # Deploy ArgoCD
.\deploy.ps1 deploy-stack       # Deploy observability stack
.\deploy.ps1 port-forward       # Set up port forwarding
.\deploy.ps1 status            # Check component status
.\deploy.ps1 logs              # View component logs
.\deploy.ps1 cleanup           # Remove applications
.\deploy.ps1 clean-all         # Complete cleanup
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
- **Prometheus Stack**: 2GB RAM, 2 CPU cores
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

- **Prometheus Community** for the monitoring stack
- **Jaeger** for distributed tracing
- **ClickHouse** for high-performance log storage
- **OpenTelemetry** for unified observability
- **ArgoCD** for GitOps deployment 