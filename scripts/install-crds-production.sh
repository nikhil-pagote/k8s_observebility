#!/bin/bash
# Production-ready Prometheus CRD installation script
# This script installs CRDs directly from prometheus-operator repository
# with proper versioning and annotation stripping

set -euo pipefail

# Configuration
PROMETHEUS_OPERATOR_VERSION="v0.68.0"  # Use stable version
CRD_BASE_URL="https://raw.githubusercontent.com/prometheus-operator/prometheus-operator/${PROMETHEUS_OPERATOR_VERSION}/example/prometheus-operator-crd"
TEMP_DIR="./tmp_crds_$(date +%s)"
NAMESPACE="observability"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    if ! command -v kubectl &> /dev/null; then
        log_error "kubectl is required but not installed"
        exit 1
    fi
    
    if ! command -v yq &> /dev/null; then
        log_error "yq is required but not installed"
        exit 1
    fi
    
    log_info "Prerequisites check passed"
}

# Create namespace
create_namespace() {
    log_info "Creating namespace: $NAMESPACE"
    kubectl create namespace "$NAMESPACE" --dry-run=client -o yaml | kubectl apply -f -
}

# Download and process CRD
download_and_process_crd() {
    local crd_name=$1
    local url="${CRD_BASE_URL}/${crd_name}"
    local output_file="${TEMP_DIR}/${crd_name}"
    
    log_info "Downloading: $crd_name"
    
    # Download CRD
    if ! curl -s -f "$url" > "$output_file"; then
        log_error "Failed to download $crd_name from $url"
        return 1
    fi
    
    # Strip annotations and clean up
    log_info "Processing: $crd_name"
    yq eval 'del(.metadata.annotations)' "$output_file" > "${output_file}.clean"
    mv "${output_file}.clean" "$output_file"
    
    # Apply CRD
    if kubectl apply -f "$output_file"; then
        log_info "✅ Successfully applied: $crd_name"
    else
        log_error "❌ Failed to apply: $crd_name"
        return 1
    fi
}

# Install all CRDs
install_crds() {
    log_info "Installing Prometheus CRDs (version: $PROMETHEUS_OPERATOR_VERSION)"
    
    # Create temporary directory
    mkdir -p "$TEMP_DIR"
    
    # List of CRDs to install
    local crds=(
        "monitoring.coreos.com_alertmanagerconfigs.yaml"
        "monitoring.coreos.com_alertmanagers.yaml"
        "monitoring.coreos.com_podmonitors.yaml"
        "monitoring.coreos.com_probes.yaml"
        "monitoring.coreos.com_prometheuses.yaml"
        "monitoring.coreos.com_prometheusrules.yaml"
        "monitoring.coreos.com_servicemonitors.yaml"
        "monitoring.coreos.com_thanosrulers.yaml"
        "monitoring.coreos.com_scrapeconfigs.yaml"
        "monitoring.coreos.com_prometheusagents.yaml"
    )
    
    # Install each CRD
    local failed_crds=()
    for crd in "${crds[@]}"; do
        if ! download_and_process_crd "$crd"; then
            failed_crds+=("$crd")
        fi
    done
    
    # Report results
    if [ ${#failed_crds[@]} -eq 0 ]; then
        log_info "🎉 All CRDs installed successfully!"
    else
        log_error "❌ Failed to install CRDs: ${failed_crds[*]}"
        exit 1
    fi
}

# Verify installation
verify_installation() {
    log_info "Verifying CRD installation..."
    
    local expected_crds=(
        "alertmanagerconfigs.monitoring.coreos.com"
        "alertmanagers.monitoring.coreos.com"
        "podmonitors.monitoring.coreos.com"
        "probes.monitoring.coreos.com"
        "prometheuses.monitoring.coreos.com"
        "prometheusrules.monitoring.coreos.com"
        "servicemonitors.monitoring.coreos.com"
        "thanosrulers.monitoring.coreos.com"
        "scrapeconfigs.monitoring.coreos.com"
        "prometheusagents.monitoring.coreos.com"
    )
    
    local missing_crds=()
    for crd in "${expected_crds[@]}"; do
        if ! kubectl get crd "$crd" &> /dev/null; then
            missing_crds+=("$crd")
        fi
    done
    
    if [ ${#missing_crds[@]} -eq 0 ]; then
        log_info "✅ All CRDs verified successfully!"
    else
        log_error "❌ Missing CRDs: ${missing_crds[*]}"
        exit 1
    fi
}

# Cleanup
cleanup() {
    log_info "Cleaning up temporary files..."
    rm -rf "$TEMP_DIR"
}

# Main execution
main() {
    log_info "Starting Prometheus CRD installation..."
    
    check_prerequisites
    create_namespace
    install_crds
    verify_installation
    cleanup
    
    log_info "🎉 Prometheus CRD installation completed successfully!"
    log_info "You can now deploy the Prometheus stack via ArgoCD"
}

# Handle script interruption
trap cleanup EXIT

# Run main function
main "$@" 