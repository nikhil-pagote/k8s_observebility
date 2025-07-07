# Kubernetes Observability Stack PowerShell Script
# This script provides convenient commands for managing the observability stack

param(
    [Parameter(Position=0)]
    [string]$Command = "help",
    
    [Parameter()]
    [string]$Namespace = "observability"
)

# Colors for output
$Green = "Green"
$Yellow = "Yellow"
$Red = "Red"
$Cyan = "Cyan"
$White = "White"

function Write-Status {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    Write-Host $Message -ForegroundColor $Color
}

function Write-Error-Status {
    param([string]$Message)
    Write-Status $Message $Red
}

function Write-Success-Status {
    param([string]$Message)
    Write-Status $Message $Green
}

function Write-Info-Status {
    param([string]$Message)
    Write-Status $Message $Cyan
}

function Write-Warning-Status {
    param([string]$Message)
    Write-Status $Message $Yellow
}

function Invoke-Command {
    param(
        [string]$Command,
        [bool]$CheckExitCode = $true
    )
    
    Write-Info-Status "Executing: $Command"
    $result = Invoke-Expression $Command
    
    if ($CheckExitCode -and $LASTEXITCODE -ne 0) {
        Write-Error-Status "Command failed with exit code: $LASTEXITCODE"
        throw "Command failed: $Command"
    }
    
    return $result
}

function Show-Help {
    Write-Status "Kubernetes Observability Stack Management" $Cyan
    Write-Status "==========================================" $Cyan
    Write-Status ""
    Write-Status "Available commands:"
    Write-Status "  setup-cluster      - Create and configure Kind cluster"
    Write-Status "  deploy-argocd      - Deploy ArgoCD to the cluster"
    Write-Status "  deploy-stack       - Deploy observability stack via ArgoCD"
    Write-Status "  deploy-stack       - Deploy observability stack via ArgoCD"
    Write-Status "  deploy-sample-apps - Deploy sample applications for testing"
    Write-Status "  build-scripts      - Build Rust deployment scripts using Docker"
    Write-Status "  status            - Show status of all components"
    Write-Status "  logs              - Show logs for key components"
    Write-Status "  cleanup           - Remove sample apps and ArgoCD apps"
    Write-Status "  clean-all         - Remove everything including Kind cluster"
    Write-Status "  quick-start       - Complete setup from scratch"
    Write-Status "  dev-setup         - Development environment setup"
    Write-Status "  port-forward      - Set up port forwarding for local access"
    Write-Status "  get-urls          - Get service URLs"
    Write-Status "  troubleshoot      - Show troubleshooting information"
    Write-Status "  help              - Show this help message"
    Write-Status ""
    Write-Status "Components:"
    Write-Status "  📊 Prometheus + Grafana - Metrics and visualization"
    Write-Status "  📊 Prometheus + Grafana - Metrics and visualization"
    Write-Status "  🔍 Jaeger - Distributed tracing"
    Write-Status "  📡 OpenTelemetry Collector - Data collection"
    Write-Status ""
    Write-Status "Usage: .\deploy.ps1 <command> [options]"
    Write-Status "Example: .\deploy.ps1 deploy-stack"
}

function Test-Prerequisites {
    Write-Warning-Status "🔍 Checking prerequisites..."
    
    # Check kubectl
    try {
        $null = Get-Command kubectl -ErrorAction Stop
        Write-Success-Status "✅ kubectl is available"
    }
    catch {
        Write-Error-Status "❌ kubectl is required but not installed"
        return $false
    }
    
    # Check kind
    try {
        $null = Get-Command kind -ErrorAction Stop
        Write-Success-Status "✅ kind is available"
    }
    catch {
        Write-Error-Status "❌ kind is required but not installed"
        return $false
    }
    
    # Check docker
    try {
        $null = Get-Command docker -ErrorAction Stop
        Write-Success-Status "✅ docker is available"
    }
    catch {
        Write-Error-Status "❌ docker is required but not installed"
        return $false
    }
    
    Write-Success-Status "✅ All prerequisites are satisfied"
    return $true
}

function Test-Binaries {
    $binaries = @("setup_kind_cluster", "deploy_argocd", "deploy_observability_stack")
    $missing = $false
    
    foreach ($binary in $binaries) {
        if (-not (Test-Path ".\bin\$binary.exe")) {
            $missing = $true
            break
        }
    }
    
    if ($missing) {
        Write-Warning-Status "🔨 Binaries not found, building..."
        Build-Scripts
    }
}

function Build-Scripts {
    Write-Warning-Status "🔨 Building Rust deployment scripts using Docker..."
    if (-not (Test-Path ".\bin")) {
        New-Item -ItemType Directory -Path ".\bin" -Force | Out-Null
    }
    
    $dockerCmd = "docker run --rm -v `"${PWD}/src-build:/app`" -v `"${PWD}/bin:/output`" rust-builder"
    Invoke-Command $dockerCmd
    Write-Success-Status "✅ Scripts built successfully in bin/ directory"
}

function Setup-Cluster {
    Test-Binaries
    Write-Warning-Status "🔧 Setting up Kind cluster..."
    Invoke-Command ".\bin\setup_kind_cluster.exe"
    Write-Success-Status "✅ Kind cluster setup complete"
}

function Deploy-ArgoCD {
    Test-Binaries
    Write-Warning-Status "🚀 Deploying ArgoCD..."
    Invoke-Command ".\bin\deploy_argocd.exe"
    Write-Success-Status "✅ ArgoCD deployment complete"
}

function Deploy-Stack {
    Test-Binaries
    Write-Warning-Status "🚀 Deploying observability stack..."
    Invoke-Command ".\bin\deploy_observability_stack.exe"
    Write-Success-Status "✅ Observability stack deployment complete"
}



function Deploy-Sample-Apps {
    Write-Warning-Status "🚀 Deploying sample applications..."
    Invoke-Command "kubectl apply -f apps/load-generator/ -f apps/sample-app/deployment-basic.yaml -n $Namespace"
    Write-Success-Status "✅ Sample applications deployed"
}

function Show-Status {
    Write-Info-Status "📊 Cluster Status"
    Write-Status "================="
    Write-Status ""
    Write-Status "Nodes:"
    kubectl get nodes
    Write-Status ""
    Write-Status "Namespaces:"
    kubectl get namespaces
    Write-Status ""
    Write-Status "ArgoCD Applications:"
    kubectl get applications -n argocd
    Write-Status ""
    Write-Status "ArgoCD Pods:"
    kubectl get pods -n argocd
    Write-Status ""
    Write-Status "Observability Pods:"
    kubectl get pods -n $Namespace
    Write-Status ""
    Write-Status "Services:"
    kubectl get services -n $Namespace
    Write-Status ""
    Write-Status "CRDs:"
    kubectl get crd | Select-String "monitoring.coreos.com"
    Write-Status ""

    Write-Status ""
    Write-Status "Jaeger Status:"
    kubectl get pods -n $Namespace | Select-String "jaeger"
}

function Show-Logs {
    Write-Info-Status "📋 Component Logs"
    Write-Status "================="
    Write-Status ""
    Write-Status "ArgoCD Server:"
    kubectl logs -n argocd deployment/argocd-server --tail=20
    Write-Status ""
    Write-Status "ArgoCD Application Controller:"
    kubectl logs -n argocd deployment/argocd-application-controller --tail=20
    Write-Status ""
    Write-Status "Prometheus Operator:"
    kubectl logs -n $Namespace deployment/prometheus-stack-kube-prom-operator --tail=20
    Write-Status ""
    Write-Status "Grafana:"
    kubectl logs -n $Namespace deployment/prometheus-stack-grafana --tail=20
    Write-Status ""

    Write-Status ""
    Write-Status "Jaeger:"
    kubectl logs -n $Namespace deployment/jaeger --tail=20
    Write-Status ""
    Write-Status "OpenTelemetry Collector:"
    kubectl logs -n $Namespace deployment/opentelemetry-collector --tail=20
}

function Cleanup {
    Write-Warning-Status "🧹 Cleaning up applications..."
    Write-Status "Removing ArgoCD applications..."
    kubectl delete application --all -n argocd
    Write-Status "Removing sample applications..."
    kubectl delete -f apps/load-generator/ -f apps/sample-app/ -n $Namespace --ignore-not-found=true
    Write-Status "Removing observability namespace..."
    kubectl delete namespace $Namespace --ignore-not-found=true
    Write-Success-Status "✅ Cleanup complete"
}

function Clean-Binaries {
    Write-Warning-Status "🧹 Cleaning binaries..."
    if (Test-Path ".\bin") {
        Remove-Item -Recurse -Force ".\bin"
    }
    Write-Success-Status "✅ Binaries cleaned"
}

function Clean-All {
    Write-Warning-Status "🧹 Complete cleanup..."
    Write-Status "Removing applications..."
    Cleanup
    Write-Status "Removing Kind cluster..."
    kind delete cluster --name observability-cluster
    Write-Status "Removing temporary files..."
    if (Test-Path ".\tmp_crds") {
        Remove-Item -Recurse -Force ".\tmp_crds"
    }
    Write-Status "Removing binaries..."
    Clean-Binaries
    Write-Status "Cleaning Docker system..."
    docker system prune -f
    Write-Success-Status "✅ Complete cleanup finished"
}

function Quick-Start {
    Write-Warning-Status "🎉 Starting complete setup..."
    Setup-Cluster
    Deploy-ArgoCD
    Deploy-Stack
    Deploy-Sample-Apps
    Write-Success-Status "🎉 Quick start complete! Your observability stack is ready."
}

function Dev-Setup {
    Write-Warning-Status "🔧 Setting up development environment..."
    if (-not (Test-Prerequisites)) {
        return
    }
    Build-Scripts
    Setup-Cluster
    Deploy-ArgoCD
    Write-Success-Status "🔧 Development environment ready"
}

function Port-Forward {
    Write-Info-Status "🔗 Setting up port forwarding..."
    Write-Status "ArgoCD UI: http://localhost:8080 (admin/admin)"
    Write-Status "Grafana: http://localhost:3000 (admin/admin123)"
    Write-Status "Prometheus: http://localhost:9090"
    Write-Status "Jaeger UI: http://localhost:16686"

    Write-Status ""
    Write-Status "Press Ctrl+C to stop port forwarding"
    
    $jobs = @()
    $jobs += Start-Job -ScriptBlock { kubectl port-forward -n argocd svc/argocd-server 8080:80 }
    $jobs += Start-Job -ScriptBlock { kubectl port-forward -n $using:Namespace svc/prometheus-stack-grafana 3000:80 }
    $jobs += Start-Job -ScriptBlock { kubectl port-forward -n $using:Namespace svc/prometheus-stack-kube-prom-prometheus 9090:9090 }
    $jobs += Start-Job -ScriptBlock { kubectl port-forward -n $using:Namespace svc/jaeger-query 16686:80 }

    
    try {
        Wait-Job -Job $jobs
    }
    finally {
        $jobs | Stop-Job
        $jobs | Remove-Job
    }
}

function Get-Urls {
    Write-Info-Status "🌐 Service URLs"
    Write-Status "=============="
    Write-Status ""
    Write-Status "ArgoCD UI:"
    $argocdUrl = kubectl get svc argocd-server -n argocd -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>$null
    if ($argocdUrl) {
        Write-Status $argocdUrl
    } else {
        Write-Status "http://localhost:8080 (use port-forward)"
    }
    Write-Status ""
    Write-Status "Grafana:"
    $grafanaUrl = kubectl get svc prometheus-stack-grafana -n $Namespace -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>$null
    if ($grafanaUrl) {
        Write-Status $grafanaUrl
    } else {
        Write-Status "http://localhost:3000 (use port-forward)"
    }
    Write-Status ""
    Write-Status "Prometheus:"
    $prometheusUrl = kubectl get svc prometheus-stack-kube-prom-prometheus -n $Namespace -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>$null
    if ($prometheusUrl) {
        Write-Status $prometheusUrl
    } else {
        Write-Status "http://localhost:9090 (use port-forward)"
    }
    Write-Status ""
    Write-Status "Jaeger UI:"
    $jaegerUrl = kubectl get svc jaeger-query -n $Namespace -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>$null
    if ($jaegerUrl) {
        Write-Status $jaegerUrl
    } else {
        Write-Status "http://localhost:16686 (use port-forward)"
    }
    Write-Status ""

}

function Troubleshoot {
    Write-Info-Status "🔧 Troubleshooting Information"
    Write-Status "=============================="
    Write-Status ""
    Write-Status "Cluster Events:"
    kubectl get events --sort-by='.lastTimestamp' | Select-Object -Last 20
    Write-Status ""
    Write-Status "ArgoCD Application Details:"
    kubectl describe application prometheus-crds -n argocd
    Write-Status ""
    Write-Status "Prometheus CRDs:"
    kubectl get crd | Select-String "monitoring.coreos.com"
    Write-Status ""
    Write-Status "Namespace Resources:"
    kubectl get all -n $Namespace
}



# Main execution
switch ($Command.ToLower()) {
    "help" { Show-Help }
    "setup-cluster" { Setup-Cluster }
    "deploy-argocd" { Deploy-ArgoCD }
    "deploy-stack" { Deploy-Stack }
    "deploy-stack" { Deploy-Stack }
    "deploy-sample-apps" { Deploy-Sample-Apps }
    "build-scripts" { Build-Scripts }
    "status" { Show-Status }
    "logs" { Show-Logs }
    "cleanup" { Cleanup }
    "clean-all" { Clean-All }
    "quick-start" { Quick-Start }
    "dev-setup" { Dev-Setup }
    "port-forward" { Port-Forward }
    "get-urls" { Get-Urls }
    "troubleshoot" { Troubleshoot }

    default {
        Write-Error-Status "Unknown command: $Command"
        Show-Help
    }
} 