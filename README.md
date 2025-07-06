# Kubernetes Observability Stack

This project demonstrates a complete Kubernetes observability setup using **Kind (Kubernetes IN Docker)** cluster with **OpenTelemetry** as the central nervous system for all telemetry data, integrated with Prometheus and Grafana.

## Architecture Overview

### OpenTelemetry-Centric Data Flow

```
Applications → OTLP → OpenTelemetry Collector
Kubernetes → kubeletstats → OpenTelemetry Collector  
Node → hostmetrics → OpenTelemetry Collector
Logs → filelog → OpenTelemetry Collector
                    ↓
              Prometheus (scrapes from OTEL)
                    ↓
              Grafana (visualization)
```

### Detailed Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Kubernetes Cluster (Kind)                       │
│                                                                     │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐  │
│  │   Control Plane │    │   Worker Node   │    │   Worker Node   │  │
│  │   (Master)      │    │       #1        │    │       #2        │  │
│  │                 │    │                 │    │                 │  │
│  │ • API Server    │    │ ┌─────────────┐ │    │ ┌─────────────┐ │  │
│  │ • etcd          │    │ │ Applications│ │    │ │ Applications│ │  │
│  │ • Scheduler     │    │ │   (Pods)    │ │    │ │   (Pods)    │ │  │
│  │ • Controller    │    │ └─────────────┘ │    │ └─────────────┘ │  │
│  └─────────────────┘    └─────────────────┘    └─────────────────┘  │
│           │                       │                       │         │
│           └───────────────────────┼───────────────────────┘         │
│                                   │                                 │
│                            ┌──────▼───────┐                         │
│                            │OpenTelemetry │                         │
│                            │  Collector   │                         │
│                            │  (DaemonSet) │                         │
│                            │              │                         │
│                            │ Receivers:   │                         │
│                            │ • OTLP       │                         │
│                            │ • hostmetrics│                         │
│                            │ • kubeletstats│                        │
│                            │ • filelog    │                         │
│                            │              │                         │
│                            │ Processors:  │                         │
│                            │ • batch      │                         │
│                            │ • k8sattributes│                       │
│                            │ • resource   │                         │
│                            │              │                         │
│                            │ Exporters:   │                         │
│                            │ • prometheus │                         │
│                            │ • logging    │                         │
│                            └──────┬───────┘                         │
│                                   │                                 │
│                            ┌──────▼───────┐                         │
│                            │  Prometheus  │                         │
│                            │ (Metrics Only│                         │
│                            │  Storage)    │                         │
│                            └──────┬───────┘                         │
│                                   │                                 │
│                            ┌──────▼───────┐                         │
│                            │   Grafana    │                         │
│                            │(Visualization│                         │
│                            │ & Dashboards)│                         │
│                            └──────────────┘                         │
└─────────────────────────────────────────────────────────────────────┘
```

### Key Benefits of OpenTelemetry-Centric Architecture

1. **Single Collection Point**: All telemetry data flows through OpenTelemetry Collector
2. **Rich Metadata**: Kubernetes attributes automatically added to all telemetry
3. **Unified Processing**: Consistent filtering, batching, and transformation
4. **Future-Ready**: Easy to add Jaeger (traces) and Loki (logs) exporters
5. **No Duplication**: Clean separation of concerns - OpenTelemetry collects, Prometheus stores, Grafana visualizes

## Why Kind?

Kind is the recommended choice for this test environment because:

- **Fast & Lightweight**: Runs Kubernetes in Docker containers, no VM overhead
- **Multi-node Support**: Easy to create 1 control-plane + 2 worker node clusters
- **Official Kubernetes**: Uses actual Kubernetes binaries, not a distribution
- **Perfect for Testing**: Designed specifically for testing and CI environments
- **Easy Cleanup**: Just delete Docker containers to clean up
- **Reproducible**: Same environment every time you create a cluster

## Current Status ✅

**Ready to Use**: All Rust binaries have been successfully built and are ready for deployment:
- ✅ `setup_kind_cluster.exe` - Creates a multi-node Kind cluster
- ✅ `deploy_argocd.exe` - Deploys ArgoCD + Prometheus + Grafana + Jaeger
- ✅ `deploy_sample_apps.exe` - Deploys sample applications for testing
- ✅ `cleanup.exe` - Cleans up all resources

## Prerequisites

- Windows 10/11 with WSL2 enabled
- Docker Desktop with Kubernetes disabled
- At least 8GB RAM and 4 CPU cores
- [Kind](https://kind.sigs.k8s.io/) installed (the Rust binary will attempt to install if missing)
- **Docker** (for building Rust binaries - no local Rust installation needed)

### Build Rust Binaries (Docker-based)

This project uses Docker to build Rust binaries, avoiding the need for local Rust installation and Windows build tools:

```powershell
# Build the Docker image
docker build -f Dockerfile.build -t rust-builder .

# Create bin directory first (required for bind mount)
mkdir bin

# Run container with bind mounts
docker run --rm -v "${PWD}/src-build:/app" -v "${PWD}/bin:/output" rust-builder
```

This will create a `bin/` directory with the following executables:
- `setup_kind_cluster.exe`
- `deploy_argocd.exe`
- `deploy_sample_apps.exe`
- `cleanup.exe`

## Quick Start

1. **Build the Rust Binaries** (if not already built):
   ```powershell
   docker build -f Dockerfile.build -t rust-builder .
   mkdir bin
   docker run --rm -v "${PWD}/src-build:/app" -v "${PWD}/bin:/output" rust-builder
   ```

2. **Setup Kind Cluster**:
   ```powershell
   .\bin\setup_kind_cluster.exe
   ```

3. **Deploy ArgoCD**:
   ```powershell
   .\bin\deploy_argocd.exe
   ```

4. **Deploy ArgoCD**:
   ```powershell
   .\bin\deploy_argocd.exe
   ```
   
   The script will automatically:
   - Deploy ArgoCD using Helm
   - Set up port forwarding to https://localhost:8080
   - Display instructions for retrieving the admin password
   
   **Get the admin password** (run this command to retrieve the password):
   ```powershell
   kubectl -n argocd get secret argocd-initial-admin-secret -o jsonpath="{.data.password}" | [System.Text.Encoding]::UTF8.GetString([System.Convert]::FromBase64String($input))
   ```
   
   Then open https://localhost:8080 (Username: `admin`, Password: use the password from the command above)

5. **Create Required Namespace** (if not exists):
   ```powershell
   kubectl create namespace observability
   ```

6. **Deploy Observability Stack** (Fully Automated):
   
   Apply the ApplicationSet that automatically creates all applications in the correct order:
   ```powershell
   kubectl apply -f argocd-apps/applicationset.yaml
   ```
   
   This will create:
   - **CRDs Application** (syncWave 0): Installs required Custom Resource Definitions using `prometheus-crds-values.yaml`
   - **Prometheus Stack Application** (syncWave 1): Deploys Prometheus + Grafana using `prometheus-stack-values.yaml`
   - **OpenTelemetry Collector Application** (syncWave 2): Deploys OpenTelemetry Collector using `opentelemetry-values.yaml`
   
   **Option B: Manual Application Creation (via UI)**
   
   If you prefer manual creation, create applications in this order:
   
   **A. Create Prometheus Stack Application:**
   - Navigate to https://localhost:8080
   - Click **"New App"**
   - **General Settings**:
     - Application Name: `prometheus-stack`
     - Project: `default`
   - **Source Settings**:
     - Repository URL: `https://prometheus-community.github.io/helm-charts`
     - Chart: `kube-prometheus-stack`
     - Version: `45.0.0`
   - **Destination Settings**:
     - Cluster: `https://kubernetes.default.svc`
     - Namespace: `observability`
   - **Sync Policy**:
     - ✅ Enable auto-sync
     - ✅ Self Heal
     - ✅ Prune
   - **Value File**: Use the content from `argocd-apps/prometheus-stack-values.yaml`
   - Click **"Create"**
   
   **B. Create OpenTelemetry Collector Application:**
   - Click **"New App"** again
   - **General Settings**:
     - Application Name: `opentelemetry-collector`
     - Project: `default`
   - **Source Settings**:
     - Repository URL: `https://open-telemetry.github.io/opentelemetry-helm-charts`
     - Chart: `opentelemetry-collector`
     - Version: `0.50.0`
   - **Destination Settings**:
     - Cluster: `https://kubernetes.default.svc`
     - Namespace: `observability`
   - **Sync Policy**:
     - ✅ Enable auto-sync
     - ✅ Self Heal
     - ✅ Prune
   - **Value File**: Use the content from `argocd-apps/opentelemetry-values.yaml`
   - Click **"Create"**

7. **Access Dashboards** (after creating ArgoCD applications):
   
   **Start Port Forwarding Sessions** (run these in separate terminals):
   ```powershell
   # ArgoCD UI (already running from deploy_argocd.exe script)
   # Port forwarding is automatically set up by the script
   
   # Grafana (after ArgoCD deploys it)
   kubectl port-forward svc/kube-prometheus-stack-grafana -n observability 3000:80
   
   # Prometheus (after ArgoCD deploys it)
   kubectl port-forward svc/kube-prometheus-stack-prometheus -n observability 9090:9090
   
   # Jaeger (after ArgoCD deploys it)
   kubectl port-forward svc/jaeger -n observability 16686:16686
   ```
   
   **Access URLs**:
   - **ArgoCD UI**: https://localhost:8080 (Username: `admin`, Password: get from secret with `kubectl -n argocd get secret argocd-initial-admin-secret -o jsonpath="{.data.password}" | [System.Text.Encoding]::UTF8.GetString([System.Convert]::FromBase64String($input))`)
   - **Grafana**: http://localhost:3000 (Username: `admin`, Password: `admin`)
   - **Prometheus**: http://localhost:9090
   - **Jaeger**: http://localhost:16686

## Project Structure

```
k8s_observebility/
├── README.md
├── Dockerfile.build          # Docker build configuration
├── src-build/                # Source code for building
│   ├── Cargo.toml
│   ├── Cargo.lock
│   ├── target/               # Build cache (generated)
│   └── scripts/
│       ├── setup_kind_cluster.rs
│       ├── deploy_argocd.rs
│       ├── deploy_sample_apps.rs
│       └── cleanup.rs
├── bin/                      # Compiled executables (generated)
├── argocd-apps/
│   ├── applicationset.yaml              # Automated deployment of all applications
│   ├── prometheus-crds-values.yaml      # CRDs-only configuration
│   ├── prometheus-stack-values.yaml     # Prometheus Stack configuration
│   └── opentelemetry-values.yaml        # OpenTelemetry Collector configuration
├── helm-chart/
│   ├── Chart.yaml
│   ├── values.yaml
│   └── templates/
│       └── deployment.yaml
├── apps/
│   ├── sample-app/
│   └── load-generator/
└── docs/
    ├── architecture.md
    └── troubleshooting.md
```

## Features

- **Multi-node Kind Cluster**: 1 control-plane + 2 worker nodes
- **ArgoCD**: GitOps continuous deployment tool with ApplicationSet automation
- **OpenTelemetry-Centric Architecture**: Single collection point for all telemetry data
- **OpenTelemetry Collector**: Collects metrics, logs, and traces with rich Kubernetes metadata
- **Prometheus**: Metrics storage and querying (scrapes from OpenTelemetry)
- **Grafana**: Visualization and dashboards
- **Automated Deployment**: ApplicationSet ensures proper deployment order (CRDs → Prometheus → OpenTelemetry)
- **Sample Applications**: For testing observability
- **Load Generator**: To simulate traffic and metrics

## Next Steps

1. Deploy the Kind cluster using the provided Rust binaries
2. Deploy ArgoCD using the simplified script
3. Create the observability namespace
4. Deploy the complete observability stack using ApplicationSet
5. Deploy sample applications to generate metrics
6. Configure dashboards and alerts
7. Monitor and analyze the observability data

## What Gets Deployed

### OpenTelemetry Collector (DaemonSet)
- **Receivers**: OTLP, hostmetrics, kubeletstats, filelog
- **Processors**: batch, k8sattributes, resource, filter
- **Exporters**: prometheus, logging
- **Pipelines**: metrics, traces, logs

### Prometheus Stack
- **Prometheus**: Scrapes metrics from OpenTelemetry Collector
- **Grafana**: Visualization with pre-configured dashboards
- **Alertmanager**: Alert management
- **Node Exporter**: Disabled (replaced by OpenTelemetry hostmetrics)

### Data Collection
- **Application Metrics**: Via OTLP protocol
- **Node Metrics**: Via OpenTelemetry hostmetrics receiver
- **Kubernetes Metrics**: Via OpenTelemetry kubeletstats receiver
- **Container Logs**: Via OpenTelemetry filelog receiver
- **Distributed Traces**: Via OTLP protocol

## Cleanup

To clean up the entire environment:
```powershell
.\bin\cleanup.exe
```

## Troubleshooting

### Common Issues

#### 1. ArgoCD Application Issues

**Problem**: "namespaces 'observability' not found" error during sync.

**Solution**:
```powershell
# Create the required namespace first
kubectl create namespace observability

# Verify namespace exists
kubectl get namespaces | grep observability

# Then retry the ArgoCD application sync
```

**Problem**: "repository not accessible" error.

**Solution**:
- Check repository URL for typos or extra spaces
- Ensure Helm chart repositories are accessible
- Verify chart name and version are correct

**Problem**: "mode must be set" error for OpenTelemetry.

**Solution**:
- Ensure OpenTelemetry values include `mode: daemonset`
- Check that the value file content is valid YAML
- Verify all required fields are filled in ArgoCD UI

#### 2. Kind Cluster Creation Fails

**Problem**: Cluster creation fails with port mapping errors or "node(s) already exist" errors.

**Solutions**:
```powershell
# Check if cluster already exists
kind get clusters

# Delete existing cluster if needed
kind delete cluster --name observability-cluster

# Check Docker containers
docker ps -a

# Verify Kind is in PATH
kind version
```

#### 2. Kind Not Found in PATH

**Problem**: `'kind' is not recognized as an internal or external command`

**Solution**:
```powershell
# Add Kind to PATH (after installing via winget)
$env:PATH += ";$env:LOCALAPPDATA\Microsoft\WinGet\Packages\Kubernetes.kind_Microsoft.Winget.Source_8wekyb3d8bbwe"
$env:PATH += ";$env:LOCALAPPDATA\Microsoft\WinGet\Packages\Helm.Helm_Microsoft.Winget.Source_8wekyb3d8bbwe\windows-amd64"

# Verify installation
kind version
```

#### 3. Docker Build Issues

**Problem**: Cross-compilation errors or missing dependencies.

**Solutions**:
```powershell
# Rebuild Docker image
docker build -f Dockerfile.build -t rust-builder .

# Rebuild binaries
docker run --rm -v "${PWD}/src-build:/app" -v "${PWD}/bin:/output" rust-builder
```

#### 4. Port Conflicts

**Problem**: Port 6443, 80, 443, or other ports are already in use.

**Solutions**:
```powershell
# Check what's using the ports
netstat -ano | findstr :6443
netstat -ano | findstr :80
netstat -ano | findstr :443

# Stop conflicting services or modify port mappings in kind-config.yaml
```

#### 5. Docker Desktop Issues

**Problem**: Docker not running or insufficient resources.

**Solutions**:
```powershell
# Check Docker status
docker version
docker info

# Ensure Docker Desktop is running with sufficient resources:
# - At least 8GB RAM allocated
# - At least 4 CPU cores allocated
# - Kubernetes disabled in Docker Desktop settings
```

#### 6. ArgoCD Issues

**Problem**: ArgoCD deployment issues or need to completely remove ArgoCD.

**Solution**: Manually uninstall ArgoCD using Helm and delete the namespace:
```powershell
# Uninstall ArgoCD Helm release
helm uninstall argocd -n argocd

# Delete the ArgoCD namespace (this removes all remaining resources)
kubectl delete namespace argocd

# Verify ArgoCD is completely removed
kubectl get namespaces | grep argocd
kubectl get pods -A | grep argocd
```

**Note**: The CustomResourceDefinitions (CRDs) for ArgoCD applications will be kept, which is normal and expected. If you want to completely remove everything, you can also delete the CRDs:
```powershell
# Optional: Remove ArgoCD CRDs (only if you want to completely remove everything)
kubectl delete crd applications.argoproj.io
kubectl delete crd applicationsets.argoproj.io
kubectl delete crd appprojects.argoproj.io
```

### Getting Help

For additional issues and questions, check the Rust source code in the `src-build/scripts/` directory or refer to the official documentation:

- [Kind Documentation](https://kind.sigs.k8s.io/)
- [Docker Desktop Documentation](https://docs.docker.com/desktop/)
- [Kubernetes Documentation](https://kubernetes.io/docs/) 