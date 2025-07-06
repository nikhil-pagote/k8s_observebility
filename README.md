# Kubernetes Observability Stack with ArgoCD

A complete Kubernetes observability stack deployed using ArgoCD GitOps, featuring Prometheus, Grafana, and OpenTelemetry Collector with production-ready security practices.

## 🏗️ Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   ArgoCD UI     │    │   Grafana UI     │    │  Prometheus UI  │
│   (Port 8080)   │    │   (Port 3000)    │    │   (Port 9090)   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌──────────────────┐
                    │   ArgoCD Apps    │
                    │   (GitOps)       │
                    └──────────────────┘
                                 │
         ┌───────────────────────┼───────────────────────┐
         │                       │                       │
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│  OpenTelemetry  │    │   Prometheus     │    │   Sample Apps   │
│   Collector     │    │   Stack          │    │   (Load Gen)    │
│   (Wave 2)      │    │   (Wave 3)       │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## 🚀 Features

- **GitOps Deployment**: All components deployed via ArgoCD
- **Sync Waves**: Proper deployment order using ArgoCD sync waves
- **Production Security**: Non-root users, security contexts, RBAC
- **Helm-managed CRDs**: Simplified CRD management for POC
- **Resource Management**: Optimized resource limits and requests
- **Monitoring**: Complete monitoring of Kubernetes components
- **Dashboards**: Pre-configured Grafana dashboards
- **Load Generation**: Sample applications with load generator
- **OpenTelemetry**: Unified observability with OTel Collector
- **Cross-Platform**: Works on Windows, macOS, and Linux

## 📋 Prerequisites

- Docker Desktop
- Kind (Kubernetes in Docker)
- kubectl
- Helm
- Docker (for building Rust scripts)
- Make (optional, for Unix-like systems)
- PowerShell (for Windows deployment)

## 🛠️ Quick Start

### Option 1: Using PowerShell Script (Windows)

```powershell
# Complete setup from scratch
.\deploy.ps1 quick-start

# Or step by step:
.\deploy.ps1 build-scripts
.\deploy.ps1 setup-cluster
.\deploy.ps1 deploy-argocd
.\deploy.ps1 deploy-stack
.\deploy.ps1 deploy-sample-apps
```

### Option 2: Using Makefile (Unix-like systems)

```bash
# Complete setup from scratch
make quick-start

# Or step by step:
make build-scripts
make setup-cluster
make deploy-argocd
make deploy-stack
make deploy-sample-apps
```

### Option 3: Manual Deployment

```bash
# Build scripts using Docker
docker run --rm -v "${PWD}/src-build:/app" -v "${PWD}/bin:/output" rust-builder

# Setup Kind cluster
./bin/setup_kind_cluster

# Deploy ArgoCD
./bin/deploy_argocd

# Deploy observability stack
./bin/deploy_observability_stack

# Deploy sample applications
kubectl apply -f apps/load-generator/ -f apps/sample-app/ -n observability
```

## 📁 Project Structure

```
k8s_observebility/
├── apps/                          # Sample applications
│   ├── load-generator/
│   │   └── deployment.yaml
│   └── sample-app/
│       ├── deployment.yaml
│       ├── servicemonitor-test.yaml
│       └── kube-state-metrics-servicemonitor.yaml
├── argocd-apps/                   # ArgoCD application manifests
│   ├── opentelemetry-collector-app.yaml  # OTel (Sync Wave 2)
│   └── prometheus-stack-app.yaml  # Prometheus/Grafana (Sync Wave 3)
├── src-build/                     # Rust deployment scripts
│   ├── Cargo.toml
│   └── scripts/
│       ├── setup_kind_cluster.rs
│       ├── deploy_argocd.rs
│       ├── deploy_sample_apps.rs
│       └── cleanup.rs
├── bin/                          # Compiled binaries (generated)
├── deploy.ps1                    # PowerShell deployment script
├── Makefile                      # Make deployment targets
├── kind-config.yaml              # Kind cluster configuration
└── README.md
```

## 🔧 Configuration

### ArgoCD Applications

The observability stack is deployed using ArgoCD applications with sync waves:

1. **opentelemetry-collector** (Wave 2): Deploys OTel Collector
2. **prometheus-stack-poc** (Wave 3): Deploys Prometheus and Grafana with production security

### Resource Limits

All components have optimized resource limits:

- **Prometheus Operator**: 512Mi memory, 500m CPU
- **Grafana**: 1Gi memory, 1000m CPU
- **Prometheus**: 2Gi memory, 1000m CPU
- **Alertmanager**: 256Mi memory, 250m CPU
- **OpenTelemetry Collector**: 512Mi memory, 500m CPU

### Security Configuration

- **Non-root users**: All containers run as non-root
- **Security contexts**: Proper security contexts configured
- **RBAC**: Role-based access control implemented
- **Network policies**: Basic network isolation

## 🎯 Production Readiness

### ✅ **What's Production-Ready:**
- GitOps approach with ArgoCD
- Security contexts and non-root users
- Resource limits and requests
- Namespace isolation
- Cross-platform deployment scripts
- Helm-managed CRDs (simplified for POC)

### ⚠️ **POC-Level (Intentionally):**
- Backup strategy: Manual (not automated)
- Alerting: Basic webhook (not Slack/PagerDuty)
- Storage: Local storage (not distributed)
- High Availability: Single node (not multi-zone)
- Compliance: Basic (not enterprise-level)

## 🔍 Troubleshooting

### Common Issues

1. **OpenTelemetry Collector in CrashLoopBackOff**
   - Check for invalid configuration fields in Helm values
   - Common issue: `add_metric_suffixes` field is not valid for current OTel version
   - Fix: Remove invalid fields from `argocd-apps/opentelemetry-collector-app.yaml`

2. **Prometheus not scraping targets**
   - Check if ServiceMonitors have the correct `release: prometheus-stack-poc` label
   - Verify namespace has the `name: observability` label
   - Check Prometheus targets page at http://localhost:9090/targets

3. **ArgoCD sync errors**
   - Check for invalid fields in Helm values
   - Verify CRDs are installed correctly
   - Check ArgoCD application logs

4. **Storage issues**
   - Ensure storage class is available in the cluster
   - Check PVC status: `kubectl get pvc -n observability`

5. **Port-forwarding not working**
   - Use correct service names:
     - Grafana: `prometheus-stack-poc-grafana`
     - Prometheus: `prometheus-stack-poc-kube-prometheus`
   - Check if pods are running: `kubectl get pods -n observability`

### Verification Commands

```bash
# Check all pods are running
kubectl get pods -n observability

# Check ArgoCD applications
kubectl get applications -n argocd

# Check all resources in observability namespace
kubectl get all -n observability

# Check ServiceMonitors
kubectl get servicemonitor -n observability

# Check Prometheus targets
kubectl port-forward svc/prometheus-stack-poc-kube-prometheus -n observability 9090:9090
# Then visit http://localhost:9090/targets
```

## 🚀 Accessing the Stack

After deployment:

- **ArgoCD UI**: http://localhost:8080 (admin/admin)
- **Grafana**: http://localhost:3000 (admin/admin123)
- **Prometheus**: http://localhost:9090

Use port-forwarding to access the UIs:
```bash
# ArgoCD
kubectl port-forward svc/argocd-server -n argocd 8080:443

# Grafana
kubectl port-forward svc/prometheus-stack-poc-grafana -n observability 3000:80

# Prometheus
kubectl port-forward svc/prometheus-stack-poc-kube-prometheus -n observability 9090:9090
```

## 📊 Current Status

### ✅ **Working Components:**
- ArgoCD (GitOps controller)
- Prometheus Stack (Prometheus, Grafana, Alertmanager)
- Sample Applications (nginx, load generator)
- Kube-state-metrics
- ServiceMonitors and monitoring

### 🔧 **Known Issues:**
- OpenTelemetry Collector may need configuration fixes for newer versions
- Some Helm chart fields may be deprecated and need updates

### 🎯 **Next Steps:**
- Monitor OpenTelemetry Collector logs for configuration issues
- Update Helm chart versions as needed
- Add more comprehensive dashboards and alerts

## 🎉 Summary

This observability stack provides a **production-ready foundation** for Kubernetes monitoring with:
- ✅ GitOps deployment via ArgoCD
- ✅ Production security practices
- ✅ Optimized resource management
- ✅ Complete monitoring stack (Prometheus, Grafana, Alertmanager)
- ✅ Sample applications for testing
- ✅ Cross-platform deployment scripts

Ready for development and testing, with a clear path to full production deployment. 