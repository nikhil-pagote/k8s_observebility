# Kubernetes Observability Stack with ArgoCD (POC)

A complete Kubernetes observability stack deployed using ArgoCD GitOps, featuring Prometheus, Grafana, and OpenTelemetry Collector with POC-optimized configurations focused on critical production security practices.

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
│  Prometheus     │    │   OpenTelemetry  │    │   Sample Apps   │
│  CRDs (Wave 1)  │    │   Collector      │    │   (Load Gen)    │
│                 │    │   (Wave 2)       │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌──────────────────┐
                    │  Prometheus      │
                    │  Stack (Wave 3)  │
                    │  (Grafana +      │
                    │   Prometheus)    │
                    └──────────────────┘
```

## 🚀 Features

- **GitOps Deployment**: All components deployed via ArgoCD
- **Sync Waves**: Proper deployment order using ArgoCD sync waves
- **Production Security**: Non-root users, security contexts, RBAC
- **CRD Management**: Direct CRD installation (no Helm chart CRDs)
- **Resource Management**: Optimized resource limits and requests
- **Monitoring**: Complete monitoring of ArgoCD components
- **Dashboards**: Pre-configured Grafana dashboards
- **Load Generation**: Sample applications with load generator
- **OpenTelemetry**: Unified observability with OTel Collector
- **Docker-Based Builds**: Consistent builds across environments
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
.\deploy.ps1 setup-cluster
.\deploy.ps1 deploy-argocd
.\deploy.ps1 deploy-stack-manual  # Includes manual CRD installation
.\deploy.ps1 deploy-sample-apps
```

### Option 2: Using Makefile (Unix-like systems)

```bash
# Complete setup from scratch
make quick-start

# Or step by step:
make setup-cluster
make deploy-argocd
make deploy-stack-manual  # Includes manual CRD installation
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

# Deploy observability stack with manual CRD installation
./bin/deploy_observability_stack --install-crds-manually

# Deploy sample applications
kubectl apply -f apps/load-generator/ -f apps/sample-app/ -n observability
```

## 🔧 CRD Installation Issue

**Note**: The Prometheus stack CRDs have large annotations that exceed Kubernetes' 262,144 byte limit. This is a common issue in production environments. This project provides multiple solutions:

### **Development/Testing**
- **Automatic**: Use `deploy-stack-manual` which automatically installs CRDs with stripped annotations
- **Manual**: Use `install-crds` command to manually install CRDs
- **Standalone**: Use `.\deploy.ps1 install-crds` or `make install-crds`

### **Production-Ready Solutions**

#### **Option 1: Direct CRD Installation Script**
```bash
# Use the production script
chmod +x scripts/install-crds-production.sh
./scripts/install-crds-production.sh
```

#### **Option 2: Kustomize-Based CRD Management**
```yaml
# Use argocd-apps/prometheus-crds-production.yaml
# This uses Kustomize to manage CRDs directly from prometheus-operator repository
```

#### **Option 3: Separate CRD Repository**
- Store CRDs in a separate Git repository
- Use ArgoCD to manage CRDs independently
- Version CRDs separately from application stack

### **Why Helm Charts Are Not Production-Ready for CRDs**

1. **Version Conflicts**: CRD changes between chart versions cause conflicts
2. **Rollback Issues**: CRD changes can't be easily rolled back
3. **Annotation Size**: Large annotations cause ArgoCD sync failures
4. **Dependency Management**: CRDs must be installed before operators
5. **GitOps Challenges**: CRDs should be version-controlled separately

## 📁 Project Structure

```
k8s_observebility/
├── apps/                          # Sample applications
│   ├── load-generator/
│   │   └── deployment.yaml
│   └── sample-app/
│       └── deployment.yaml
├── argocd-apps/                   # ArgoCD application manifests
│   ├── kustomization.yaml         # Manages all ArgoCD apps
│   ├── crds-poc-app.yaml          # CRDs (Sync Wave 1) - POC approach
│   ├── opentelemetry-collector-app.yaml  # OTel (Sync Wave 2)
│   ├── prometheus-stack-poc.yaml  # Prometheus/Grafana (Sync Wave 3) - POC
│   └── grafana-dashboards/
│       └── argo-cd.yaml           # ArgoCD dashboard
├── src-build/                     # Rust deployment scripts
│   ├── Cargo.toml
│   └── scripts/
│       ├── setup_kind_cluster.rs
│       ├── deploy_argocd.rs
│       ├── deploy_observability_stack.rs
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

1. **prometheus-crds-poc** (Wave 1): Installs Prometheus CRDs using direct installation
2. **opentelemetry-collector-app** (Wave 2): Deploys OTel Collector
3. **prometheus-stack-poc** (Wave 3): Deploys Prometheus and Grafana with production security

### Resource Limits

All components have optimized resource limits:

- **Prometheus Operator**: 512Mi memory, 500m CPU
- **Grafana**: 512Mi memory, 500m CPU
- **Prometheus**: 2Gi memory, 1000m CPU
- **Alertmanager**: 256Mi memory, 250m CPU
- **OpenTelemetry Collector**: 512Mi memory, 500m CPU

### Monitoring Configuration

- **ArgoCD Metrics**: Server and controller metrics enabled
- **Redis Metrics**: Disabled to reduce overhead
- **Prometheus Scraping**: Configured for ArgoCD components
- **Grafana Dashboards**: Pre-configured Kubernetes and ArgoCD dashboards

## 🌐 Access URLs

After deployment, access the services using port forwarding:

### Using PowerShell Script
```powershell
.\deploy.ps1 port-forward
```

### Using Makefile
```bash
make port-forward
```

### Manual Port Forwarding

#### ArgoCD UI
```bash
kubectl port-forward svc/argocd-server -n argocd 8080:443
# Access: https://localhost:8080
# Username: admin
# Password: kubectl -n argocd get secret argocd-initial-admin-secret -o jsonpath="{.data.password}" | base64 -d
```

#### Grafana
```bash
kubectl port-forward svc/prometheus-stack-grafana -n observability 3000:80
# Access: http://localhost:3000
# Username: admin
# Password: admin
```

#### Prometheus
```bash
kubectl port-forward svc/prometheus-stack-kube-prom-prometheus -n observability 9090:9090
# Access: http://localhost:9090
```

## 📊 Dashboards

### Pre-configured Dashboards

1. **Kubernetes Cluster Overview** (ID: 7249)
2. **Kubernetes Pods** (ID: 6417)
3. **ArgoCD Dashboard** (ID: 14584)

### Custom ArgoCD Dashboard

A custom ArgoCD dashboard is automatically deployed with:
- Application health status
- Application sync metrics
- Repository connections
- Application health by name

## 🔍 Monitoring

### ArgoCD Components Monitored

- **argocd-server**: Application server metrics
- **argocd-repo-server**: Repository server metrics
- **argocd-application-controller**: Application controller metrics

### Sample Applications

- **sample-app**: Nginx-based sample application
- **load-generator**: Generates traffic to sample app

## 🛠️ Management Commands

### PowerShell Script Commands
```powershell
.\deploy.ps1 help              # Show all available commands
.\deploy.ps1 status            # Show cluster status
.\deploy.ps1 logs              # Show component logs
.\deploy.ps1 get-urls          # Get service URLs
.\deploy.ps1 troubleshoot      # Show troubleshooting info
.\deploy.ps1 cleanup           # Remove applications
.\deploy.ps1 clean-all         # Complete cleanup
```

### Makefile Commands
```bash
make help              # Show all available commands
make status            # Show cluster status
make logs              # Show component logs
make get-urls          # Get service URLs
make troubleshoot      # Show troubleshooting info
make cleanup           # Remove applications
make clean-all         # Complete cleanup
```

## 🧹 Cleanup

### Using PowerShell Script
```powershell
# Remove applications only
.\deploy.ps1 cleanup

# Complete cleanup including cluster and Docker
.\deploy.ps1 clean-all
```

### Using Makefile
```bash
# Remove applications only
make cleanup

# Complete cleanup including cluster and Docker
make clean-all
```

### Manual Cleanup
```bash
# Remove ArgoCD applications
kubectl delete application --all -n argocd

# Remove sample applications
kubectl delete -f apps/load-generator/ -f apps/sample-app/ -n observability

# Remove Kind cluster
kind delete cluster --name observability-cluster

# Clean Docker system
docker system prune -f
```

## 🔧 Troubleshooting

### Common Issues

1. **CRD Annotation Size Error**: Use `deploy-stack-manual` or `install-crds` commands
2. **ArgoCD Applications Out of Sync**: Check CRD installation first
3. **Port Forwarding Issues**: Ensure services are running before port forwarding

### Debug Commands

```powershell
# PowerShell
.\deploy.ps1 troubleshoot
.\deploy.ps1 status
.\deploy.ps1 logs
```

```bash
# Makefile
make troubleshoot
make status
make logs
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test with `make quick-start` or `.\deploy.ps1 quick-start`
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License. 