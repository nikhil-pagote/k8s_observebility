use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use tokio::time::sleep;

#[derive(Parser, Debug)]
#[command(author, version, about = "Deploy complete Kubernetes observability stack with ArgoCD", long_about = None)]
struct Args {
    #[arg(long, default_value = "observability")]
    namespace: String,
}

struct ObservabilityStackDeployer {
    namespace: String,
}

impl ObservabilityStackDeployer {
    fn new(namespace: String) -> Self {
        Self {
            namespace,
        }
    }

    fn print_status(&self, message: &str, color: &str) {
        let colored_message = match color {
            "green" => message.green(),
            "yellow" => message.yellow(),
            "red" => message.red(),
            "cyan" => message.cyan(),
            "white" => message.white(),
            _ => message.normal(),
        };
        println!("{}", colored_message);
    }

    fn run_command(&self, command: &str, check: bool) -> Result<std::process::Output> {
        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("powershell");
            c.args(&["-NoProfile", "-Command", command]);
            c
        } else {
            let mut c = Command::new("sh");
            c.args(&["-c", command]);
            c
        };

        let output = cmd.output().context(format!("Failed to execute command: {}", command))?;

        if check && !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Command failed: {}\nError: {}", command, stderr);
        }

        Ok(output)
    }

    fn ensure_kind_context(&self) -> Result<()> {
        self.print_status("🔧 Ensuring correct Kind context...", "yellow");
        
        // First, check if the cluster exists
        match self.run_command("kind get clusters", false) {
            Ok(output) => {
                let clusters = String::from_utf8_lossy(&output.stdout);
                if !clusters.contains("observability-cluster") {
                    anyhow::bail!("Kind cluster 'observability-cluster' not found. Please run setup_kind_cluster.exe first.");
                }
            }
            Err(_) => {
                anyhow::bail!("Kind is not available or cluster not found. Please run setup_kind_cluster.exe first.");
            }
        }
        
        // Export kubeconfig to default location and fix the server endpoint
        self.run_command(&format!("kind export kubeconfig --name observability-cluster"), false)?;
        self.run_command("kubectl config set-cluster kind-observability-cluster --server=https://127.0.0.1:6443", false)?;
        
        // Test the connection
        match self.run_command("kubectl cluster-info", false) {
            Ok(_) => {
                self.print_status("✅ Kind context set correctly", "green");
                Ok(())
            }
            Err(_) => {
                // Try one more time with explicit context
                match self.run_command("kubectl cluster-info --context kind-observability-cluster", false) {
                    Ok(_) => {
                        self.print_status("✅ Kind context set correctly", "green");
                        Ok(())
                    }
                    Err(_) => {
                        self.print_status("❌ Failed to connect to cluster", "red");
                        anyhow::bail!("Cannot connect to Kind cluster. Please ensure the cluster is running.");
                    }
                }
            }
        }
    }

    fn check_prerequisites(&self) -> Result<bool> {
        self.print_status("🔍 Checking prerequisites...", "yellow");
        
        // Check if kubectl is available
        match self.run_command("kubectl version --client", false) {
            Ok(_) => self.print_status("✅ kubectl is available", "green"),
            Err(_) => {
                self.print_status("❌ kubectl is not available", "red");
                return Ok(false);
            }
        }

        // Ensure we're using the correct Kind context
        self.ensure_kind_context()?;
        
        // Check if cluster is accessible
        match self.run_command("kubectl get nodes", false) {
            Ok(_) => self.print_status("✅ Kubernetes cluster is accessible", "green"),
            Err(_) => {
                self.print_status("❌ Cannot access Kubernetes cluster", "red");
                return Ok(false);
            }
        }

        // Check if ArgoCD is running
        match self.run_command("kubectl get pods -n argocd --no-headers", false) {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if output_str.contains("Running") {
                    self.print_status("✅ ArgoCD is running", "green");
                } else {
                    self.print_status("⚠️ ArgoCD pods not all running", "yellow");
                }
            }
            Err(_) => {
                self.print_status("❌ ArgoCD not found or not accessible", "red");
                return Ok(false);
            }
        }

        Ok(true)
    }

    async fn deploy_argocd_apps(&self) -> Result<()> {
        self.print_status("🚀 Deploying ArgoCD applications for observability stack...", "yellow");
        
        // Create the observability namespace first
        self.print_status("📁 Creating observability namespace...", "yellow");
        let namespace_command = format!("kubectl create namespace {} --dry-run=client -o yaml | kubectl apply -f -", self.namespace);
        self.run_command(&namespace_command, false)?;
        
        // Deploy ArgoCD applications using kustomize
        let kustomize_command = "kubectl apply -k argocd-apps/";
        match self.run_command(kustomize_command, true) {
            Ok(_) => {
                self.print_status("✅ ArgoCD applications deployed successfully", "green");
            }
            Err(e) => {
                self.print_status(&format!("❌ Failed to deploy ArgoCD applications: {}", e), "red");
                return Err(e);
            }
        }

        // Wait for applications to be created
        self.print_status("⏳ Waiting for ArgoCD applications to be created...", "yellow");
        sleep(Duration::from_secs(5)).await;

        // Check application status
        match self.run_command("kubectl get applications -n argocd", false) {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                self.print_status("📋 ArgoCD Applications:", "cyan");
                println!("{}", output_str);
            }
            Err(_) => {
                self.print_status("⚠️ Could not retrieve application status", "yellow");
            }
        }

        Ok(())
    }

    async fn deploy_sample_apps(&self) -> Result<()> {
        self.print_status("🚀 Deploying sample applications for testing...", "yellow");
        
        // Deploy sample applications
        let sample_apps_command = format!("kubectl apply -f apps/load-generator/ -f apps/sample-app/ -n {}", self.namespace);
        match self.run_command(&sample_apps_command, true) {
            Ok(_) => {
                self.print_status("✅ Sample applications deployed successfully", "green");
            }
            Err(e) => {
                self.print_status(&format!("❌ Failed to deploy sample applications: {}", e), "red");
                return Err(e);
            }
        }

        // Wait for sample applications to be ready
        self.print_status("⏳ Waiting for sample applications to be ready...", "yellow");
        let max_attempts = 30;
        let mut attempt = 0;
        
        while attempt < max_attempts {
            match self.run_command(&format!("kubectl get pods -n {} --no-headers | grep -v Running | grep -v Completed", self.namespace), false) {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    if output_str.trim().is_empty() {
                        self.print_status("✅ Sample applications are ready", "green");
                        break;
                    }
                }
                Err(_) => {}
            }
            
            attempt += 1;
            self.print_status(&format!("⏳ Waiting for sample applications... (Attempt {}/{})", attempt, max_attempts), "yellow");
            sleep(Duration::from_secs(10)).await;
        }

        Ok(())
    }

    fn get_service_urls(&self) -> Result<()> {
        self.print_status("🌐 Getting service URLs...", "yellow");
        
        // Get Grafana URL
        match self.run_command(&format!("kubectl get svc -n {} prometheus-stack-grafana -o jsonpath='{{.status.loadBalancer.ingress[0].ip}}'", self.namespace), false) {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let ip = output_str.trim();
                if !ip.is_empty() {
                    self.print_status(&format!("🔗 Grafana: http://{}:80", ip), "cyan");
                    self.print_status("   Username: admin, Password: admin123", "white");
                }
            }
            Err(_) => {
                self.print_status("🔗 Grafana: Use port-forward: kubectl port-forward svc/prometheus-stack-grafana -n observability 3000:80", "cyan");
                self.print_status("   Username: admin, Password: admin123", "white");
            }
        }

        // Get Prometheus URL
        match self.run_command(&format!("kubectl get svc -n {} prometheus-stack-kube-prom-prometheus -o jsonpath='{{.status.loadBalancer.ingress[0].ip}}'", self.namespace), false) {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let ip = output_str.trim();
                if !ip.is_empty() {
                    self.print_status(&format!("🔗 Prometheus: http://{}:9090", ip), "cyan");
                }
            }
            Err(_) => {
                self.print_status("🔗 Prometheus: Use port-forward: kubectl port-forward svc/prometheus-stack-kube-prom-prometheus -n observability 9090:9090", "cyan");
            }
        }

        // Get Jaeger URL
        match self.run_command(&format!("kubectl get svc -n {} jaeger-query -o jsonpath='{{.status.loadBalancer.ingress[0].ip}}'", self.namespace), false) {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let ip = output_str.trim();
                if !ip.is_empty() {
                    self.print_status(&format!("🔗 Jaeger UI: http://{}:16686", ip), "cyan");
                }
            }
            Err(_) => {
                self.print_status("🔗 Jaeger UI: Use port-forward: kubectl port-forward svc/jaeger-query -n observability 16686:16686", "cyan");
            }
        }

        Ok(())
    }

    async fn deploy(&self) -> Result<bool> {
        self.print_status("🚀 Deploying Complete Kubernetes Observability Stack", "green");
        self.print_status(&format!("Namespace: {}", self.namespace), "cyan");
        
        // Check prerequisites
        if !self.check_prerequisites()? {
            return Ok(false);
        }
        
        // Deploy ArgoCD applications
        self.deploy_argocd_apps().await?;
        
        // Deploy sample applications for testing
        self.deploy_sample_apps().await?;
        
        // Get service URLs
        self.get_service_urls()?;
        
        self.print_status("", "white");
        self.print_status("🎉 Observability Stack Deployment Completed Successfully!", "green");
        self.print_status("", "white");
        self.print_status("📋 What was deployed:", "cyan");
        self.print_status("   ✅ Prometheus Stack with Grafana (Sync Wave 1)", "white");
        self.print_status("   ✅ Jaeger - Distributed Tracing (Sync Wave 2)", "white");
        self.print_status("   ✅ OpenTelemetry Collector (Sync Wave 2)", "white");
        self.print_status("   ✅ Sample Applications for Testing", "white");
        self.print_status("", "white");
        self.print_status("📋 Access URLs:", "cyan");
        self.print_status("   1. ArgoCD UI: https://localhost:8080", "white");
        self.print_status("      Port forwarding: kubectl port-forward svc/argocd-server -n argocd 8080:443", "white");
        self.print_status("   2. Grafana: http://localhost:3000", "white");
        self.print_status("      Port forwarding: kubectl port-forward svc/prometheus-stack-grafana -n observability 3000:80", "white");
        self.print_status("   3. Prometheus: http://localhost:9090", "white");
        self.print_status("      Port forwarding: kubectl port-forward svc/prometheus-stack-kube-prom-prometheus -n observability 9090:9090", "white");
        self.print_status("   4. Jaeger UI: http://localhost:16686", "white");
        self.print_status("      Port forwarding: kubectl port-forward svc/jaeger-query -n observability 16686:16686", "white");
        self.print_status("", "white");
        self.print_status("🔍 Monitor deployment in ArgoCD UI to see sync waves in action!", "cyan");
        self.print_status("🔍 Grafana is pre-configured with Jaeger data sources", "cyan");
        
        Ok(true)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    let deployer = ObservabilityStackDeployer::new(
        args.namespace,
    );
    
    let success = deployer.deploy().await?;
    
    if success {
        Ok(())
    } else {
        std::process::exit(1);
    }
} 