# Kubernetes Observability Stack Makefile
# This Makefile provides convenient commands for managing the observability stack

.PHONY: help setup-cluster deploy-argocd deploy-stack deploy-sample-apps cleanup clean-all build-scripts status logs

# Default target
help:
	@echo "Kubernetes Observability Stack Management"
	@echo "=========================================="
	@echo ""
	@echo "Available targets:"
	@echo "  setup-cluster      - Create and configure Kind cluster"
	@echo "  deploy-argocd      - Deploy ArgoCD to the cluster"
	@echo "  deploy-stack       - Deploy observability stack via ArgoCD"
	@echo "  deploy-stack       - Deploy observability stack via ArgoCD"
	@echo "  deploy-sample-apps - Deploy sample applications for testing"
	@echo "  build-scripts      - Build Rust deployment scripts using Docker"
	@echo "  status            - Show status of all components"
	@echo "  logs              - Show logs for key components"
	@echo "  cleanup           - Remove sample apps and ArgoCD apps"
	@echo "  clean-all         - Remove everything including Kind cluster"
	@echo "  help              - Show this help message"
	@echo ""

# Setup Kind cluster
setup-cluster: check-binaries
	@echo "🔧 Setting up Kind cluster..."
	./bin/setup_kind_cluster
	@echo "✅ Kind cluster setup complete"

# Deploy ArgoCD
deploy-argocd: check-binaries
	@echo "🚀 Deploying ArgoCD..."
	./bin/deploy_argocd
	@echo "✅ ArgoCD deployment complete"

# Deploy observability stack (standard)
deploy-stack: check-binaries
	@echo "🚀 Deploying observability stack..."
	./bin/deploy_observability_stack
	@echo "✅ Observability stack deployment complete"

# Deploy observability stack with manual CRD installation
deploy-stack: check-binaries
	@echo "🚀 Deploying observability stack with manual CRD installation..."
	./bin/deploy_observability_stack --install-crds-manually
	@echo "✅ Observability stack deployment complete"

# Deploy sample applications
deploy-sample-apps:
	@echo "🚀 Deploying sample applications..."
	kubectl apply -f apps/load-generator/ -f apps/sample-app/deployment-basic.yaml -n observability
	@echo "✅ Sample applications deployed"

# Build Rust scripts using Docker
build-scripts:
	@echo "🔨 Building Rust deployment scripts using Docker..."
	@mkdir -p bin
	docker run --rm -v "${PWD}/src-build:/app" -v "${PWD}/bin:/output" rust-builder
	@echo "✅ Scripts built successfully in bin/ directory"

# Show status of all components
status:
	@echo "📊 Cluster Status"
	@echo "================="
	@echo ""
	@echo "Nodes:"
	@kubectl get nodes
	@echo ""
	@echo "Namespaces:"
	@kubectl get namespaces
	@echo ""
	@echo "ArgoCD Applications:"
	@kubectl get applications -n argocd
	@echo ""
	@echo "ArgoCD Pods:"
	@kubectl get pods -n argocd
	@echo ""
	@echo "Observability Pods:"
	@kubectl get pods -n observability
	@echo ""
	@echo "Services:"
	@kubectl get services -n observability
	@echo ""
	@echo "CRDs:"
	@kubectl get crd | grep monitoring.coreos.com

# Show logs for key components
logs:
	@echo "📋 Component Logs"
	@echo "================="
	@echo ""
	@echo "ArgoCD Server:"
	@kubectl logs -n argocd deployment/argocd-server --tail=20
	@echo ""
	@echo "ArgoCD Application Controller:"
	@kubectl logs -n argocd deployment/argocd-application-controller --tail=20
	@echo ""
	@echo "Prometheus Operator:"
	@kubectl logs -n observability deployment/prometheus-stack-kube-prom-operator --tail=20
	@echo ""
	@echo "Grafana:"
	@kubectl logs -n observability deployment/prometheus-stack-grafana --tail=20

# Cleanup sample apps and ArgoCD apps
cleanup:
	@echo "🧹 Cleaning up applications..."
	@echo "Removing ArgoCD applications..."
	kubectl delete application --all -n argocd
	@echo "Removing sample applications..."
	kubectl delete -f apps/load-generator/ -f apps/sample-app/ -n observability --ignore-not-found=true
	@echo "Removing observability namespace..."
	kubectl delete namespace observability --ignore-not-found=true
	@echo "✅ Cleanup complete"

# Clean binaries
clean-binaries:
	@echo "🧹 Cleaning binaries..."
	rm -rf bin/
	@echo "✅ Binaries cleaned"

# Clean everything including Kind cluster
clean-all:
	@echo "🧹 Complete cleanup..."
	@echo "Removing applications..."
	$(MAKE) cleanup
	@echo "Removing Kind cluster..."
	kind delete cluster --name observability-cluster
	@echo "Removing temporary files..."
	rm -rf tmp_crds
	@echo "Removing binaries..."
	$(MAKE) clean-binaries
	@echo "Cleaning Docker system..."
	docker system prune -f
	@echo "✅ Complete cleanup finished"

# Quick start - setup everything
quick-start: setup-cluster deploy-argocd deploy-stack deploy-sample-apps
	@echo "🎉 Quick start complete! Your observability stack is ready."

# Development workflow
dev-setup: check-prereqs build-scripts setup-cluster deploy-argocd
	@echo "🔧 Development environment ready"

# Production deployment
prod-deploy: deploy-stack
	@echo "🚀 Production deployment complete"

# Check prerequisites
check-prereqs:
	@echo "🔍 Checking prerequisites..."
	@command -v kubectl >/dev/null 2>&1 || { echo "❌ kubectl is required but not installed"; exit 1; }
	@command -v kind >/dev/null 2>&1 || { echo "❌ kind is required but not installed"; exit 1; }
	@command -v docker >/dev/null 2>&1 || { echo "❌ docker is required but not installed"; exit 1; }
	@echo "✅ All prerequisites are satisfied"

# Check if binaries exist, build if needed
check-binaries:
	@if [ ! -f "./bin/setup_kind_cluster" ] || [ ! -f "./bin/deploy_argocd" ] || [ ! -f "./bin/deploy_observability_stack" ]; then \
		echo "🔨 Binaries not found, building..."; \
		$(MAKE) build-scripts; \
	fi

# Port forwarding for local access
port-forward:
	@echo "🔗 Setting up port forwarding..."
	@echo "ArgoCD UI: http://localhost:8080 (admin/admin)"
	@echo "Grafana: http://localhost:3000 (admin/admin)"
	@echo "Prometheus: http://localhost:9090"
	@echo ""
	@echo "Press Ctrl+C to stop port forwarding"
	kubectl port-forward -n argocd svc/argocd-server 8080:80 & \
	kubectl port-forward -n observability svc/prometheus-stack-grafana 3000:80 & \
	kubectl port-forward -n observability svc/prometheus-stack-kube-prom-prometheus 9090:9090 & \
	wait

# Get service URLs
get-urls:
	@echo "🌐 Service URLs"
	@echo "=============="
	@echo ""
	@echo "ArgoCD UI:"
	@kubectl get svc argocd-server -n argocd -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>/dev/null || echo "http://localhost:8080 (use port-forward)"
	@echo ""
	@echo "Grafana:"
	@kubectl get svc prometheus-stack-grafana -n observability -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>/dev/null || echo "http://localhost:3000 (use port-forward)"
	@echo ""
	@echo "Prometheus:"
	@kubectl get svc prometheus-stack-kube-prom-prometheus -n observability -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>/dev/null || echo "http://localhost:9090 (use port-forward)"

# Troubleshooting commands
troubleshoot:
	@echo "🔧 Troubleshooting Information"
	@echo "=============================="
	@echo ""
	@echo "Cluster Events:"
	@kubectl get events --sort-by='.lastTimestamp' | tail -20
	@echo ""
	@echo "ArgoCD Application Details:"
	@kubectl describe application prometheus-crds -n argocd
	@echo ""
	@echo "Prometheus CRDs:"
	@kubectl get crd | grep monitoring.coreos.com
	@echo ""
	@echo "Namespace Resources:"
	@kubectl get all -n observability

# Install CRDs manually (standalone)
install-crds:
	@echo "🔧 Installing Prometheus CRDs manually..."
	@mkdir -p tmp_crds
	@echo "Downloading CRDs..."
	@curl -s https://raw.githubusercontent.com/prometheus-operator/prometheus-operator/main/example/prometheus-operator-crd/monitoring.coreos.com_prometheuses.yaml | \
		yq eval 'del(.metadata.annotations)' - > tmp_crds/prometheuses.yaml
	@curl -s https://raw.githubusercontent.com/prometheus-operator/prometheus-operator/main/example/prometheus-operator-crd/monitoring.coreos.com_alertmanagerconfigs.yaml | \
		yq eval 'del(.metadata.annotations)' - > tmp_crds/alertmanagerconfigs.yaml
	@curl -s https://raw.githubusercontent.com/prometheus-operator/prometheus-operator/main/example/prometheus-operator-crd/monitoring.coreos.com_alertmanagers.yaml | \
		yq eval 'del(.metadata.annotations)' - > tmp_crds/alertmanagers.yaml
	@curl -s https://raw.githubusercontent.com/prometheus-operator/prometheus-operator/main/example/prometheus-operator-crd/monitoring.coreos.com_thanosrulers.yaml | \
		yq eval 'del(.metadata.annotations)' - > tmp_crds/thanosrulers.yaml
	@curl -s https://raw.githubusercontent.com/prometheus-operator/prometheus-operator/main/example/prometheus-operator-crd/monitoring.coreos.com_scrapeconfigs.yaml | \
		yq eval 'del(.metadata.annotations)' - > tmp_crds/scrapeconfigs.yaml
	@curl -s https://raw.githubusercontent.com/prometheus-operator/prometheus-operator/main/example/prometheus-operator-crd/monitoring.coreos.com_prometheusagents.yaml | \
		yq eval 'del(.metadata.annotations)' - > tmp_crds/prometheusagents.yaml
	@echo "Applying CRDs..."
	@kubectl apply -f tmp_crds/
	@echo "✅ CRDs installed successfully" 