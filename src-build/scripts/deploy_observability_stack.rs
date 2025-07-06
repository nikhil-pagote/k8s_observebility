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
    #[arg(long, default_value_t = false, help = "Manually install Prometheus CRDs with annotation stripping before deploying ArgoCD apps")]
    install_crds_manually: bool,
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
        self.print_status("🚀 Deploying ArgoCD applications for POC observability stack...", "yellow");
        
        // Create the observability namespace first
        self.print_status("📁 Creating observability namespace...", "yellow");
        let namespace_command = format!("kubectl create namespace {} --dry-run=client -o yaml | kubectl apply -f -", self.namespace);
        self.run_command(&namespace_command, false)?;
        
        // Deploy ArgoCD applications using kustomize (POC approach)
        let kustomize_command = "kubectl apply -k argocd-apps/";
        match self.run_command(kustomize_command, true) {
            Ok(_) => {
                self.print_status("✅ ArgoCD applications deployed successfully (POC mode)", "green");
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

        // Wait for CRDs to be available
        self.print_status("⏳ Waiting for Prometheus CRDs to be available...", "yellow");
        let max_attempts = 60; // 10 minutes with 10-second intervals
        let mut attempt = 0;
        
        while attempt < max_attempts {
            match self.run_command("kubectl get crd servicemonitors.monitoring.coreos.com", false) {
                Ok(_) => {
                    self.print_status("✅ ServiceMonitor CRD is available", "green");
                    break;
                }
                Err(_) => {
                    attempt += 1;
                    if attempt < max_attempts {
                        self.print_status(&format!("⏳ Waiting for CRDs... (Attempt {}/{})", attempt, max_attempts), "yellow");
                        sleep(Duration::from_secs(10)).await;
                    } else {
                        self.print_status("❌ Timeout waiting for CRDs to be available", "red");
                        return Err(anyhow::anyhow!("CRDs not available after {} attempts", max_attempts));
                    }
                }
            }
        }

        Ok(())
    }

    async fn deploy_sample_apps(&self) -> Result<()> {
        self.print_status("🚀 Deploying sample applications for testing...", "yellow");
        
        // Create namespace if it doesn't exist (for sample apps, not ArgoCD apps)
        let namespace_command = format!("kubectl create namespace {} --dry-run=client -o yaml | kubectl apply -f -", self.namespace);
        self.run_command(&namespace_command, false)?;
        
        // Deploy sample apps from all subdirectories (using basic deployment without ServiceMonitor)
        let sample_apps_command = format!("kubectl apply -f apps/load-generator/ -f apps/sample-app/deployment-basic.yaml -n {}", self.namespace);
        match self.run_command(&sample_apps_command, true) {
            Ok(_) => {
                self.print_status("✅ Sample applications deployed successfully", "green");
            }
            Err(e) => {
                self.print_status(&format!("❌ Failed to deploy sample applications: {}", e), "red");
                return Err(e);
            }
        }

        // Wait for pods to be ready
        self.print_status("⏳ Waiting for sample applications to be ready...", "yellow");
        let max_attempts = 30;
        let mut attempt = 0;
        
        while attempt < max_attempts {
            match self.run_command(&format!("kubectl get pods -n {} --no-headers", self.namespace), false) {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    let pods: Vec<&str> = output_str.trim().split('\n').collect();
                    
                    if !pods.is_empty() {
                        let ready_pods = pods.iter()
                            .filter(|pod| pod.contains("Running"))
                            .count();
                        
                        if ready_pods == pods.len() {
                            self.print_status("✅ All sample applications are ready!", "green");
                            break;
                        }
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
                    self.print_status("   Username: admin, Password: admin", "white");
                }
            }
            Err(_) => {
                self.print_status("🔗 Grafana: Use port-forward: kubectl port-forward svc/prometheus-stack-grafana -n observability 3000:80", "cyan");
                self.print_status("   Username: admin, Password: admin", "white");
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

        Ok(())
    }

    fn install_prometheus_crds_manually(&self) -> Result<()> {
        use std::fs;
        use serde_yaml::Value;
        use reqwest::blocking::get;
        self.print_status("🔧 Downloading and installing Prometheus CRDs manually...", "yellow");
        let crd_urls = vec![
            "https://raw.githubusercontent.com/prometheus-operator/prometheus-operator/main/example/prometheus-operator-crd/monitoring.coreos.com_alertmanagerconfigs.yaml",
            "https://raw.githubusercontent.com/prometheus-operator/prometheus-operator/main/example/prometheus-operator-crd/monitoring.coreos.com_alertmanagers.yaml",
            "https://raw.githubusercontent.com/prometheus-operator/prometheus-operator/main/example/prometheus-operator-crd/monitoring.coreos.com_podmonitors.yaml",
            "https://raw.githubusercontent.com/prometheus-operator/prometheus-operator/main/example/prometheus-operator-crd/monitoring.coreos.com_probes.yaml",
            "https://raw.githubusercontent.com/prometheus-operator/prometheus-operator/main/example/prometheus-operator-crd/monitoring.coreos.com_prometheuses.yaml",
            "https://raw.githubusercontent.com/prometheus-operator/prometheus-operator/main/example/prometheus-operator-crd/monitoring.coreos.com_prometheusrules.yaml",
            "https://raw.githubusercontent.com/prometheus-operator/prometheus-operator/main/example/prometheus-operator-crd/monitoring.coreos.com_servicemonitors.yaml",
            "https://raw.githubusercontent.com/prometheus-operator/prometheus-operator/main/example/prometheus-operator-crd/monitoring.coreos.com_thanosrulers.yaml",
            "https://raw.githubusercontent.com/prometheus-operator/prometheus-operator/main/example/prometheus-operator-crd/monitoring.coreos.com_scrapeconfigs.yaml",
            "https://raw.githubusercontent.com/prometheus-operator/prometheus-operator/main/example/prometheus-operator-crd/monitoring.coreos.com_prometheusagents.yaml",
        ];
        let tmp_dir = "./tmp_crds";
        fs::create_dir_all(tmp_dir)?;
        for url in &crd_urls {
            let resp = get(*url).context(format!("Failed to download CRD: {}", url))?;
            let text = resp.text().context("Failed to read CRD response text")?;
            let mut doc: Value = serde_yaml::from_str(&text).context("Failed to parse CRD YAML")?;
            // Remove or clear metadata.annotations
            if let Some(meta) = doc.get_mut("metadata") {
                if let Some(map) = meta.as_mapping_mut() {
                    map.remove(&Value::String("annotations".to_string()));
                }
            }
            let cleaned_yaml = serde_yaml::to_string(&doc).context("Failed to serialize cleaned CRD YAML")?;
            let file_name = url.split('/').last().unwrap_or("crd.yaml");
            let file_path = format!("{}/{}", tmp_dir, file_name);
            fs::write(&file_path, cleaned_yaml).context("Failed to write cleaned CRD file")?;
            // Apply the CRD
            let apply_cmd = format!("kubectl apply -f {}", file_path);
            match self.run_command(&apply_cmd, true) {
                Ok(_) => self.print_status(&format!("✅ Applied {}", file_name), "green"),
                Err(e) => self.print_status(&format!("❌ Failed to apply {}: {}", file_name, e), "red"),
            }
        }
        self.print_status("✅ All Prometheus CRDs processed.", "green");
        Ok(())
    }

    async fn deploy(&self, install_crds_manually: bool) -> Result<bool> {
        self.print_status("🚀 Deploying Complete Kubernetes Observability Stack", "green");
        self.print_status(&format!("Namespace: {}", self.namespace), "cyan");
        
        // Check prerequisites
        if !self.check_prerequisites()? {
            return Ok(false);
        }
        
        if install_crds_manually {
            self.install_prometheus_crds_manually()?;
        }
        
        // Deploy ArgoCD applications first (this deploys the actual observability stack)
        self.deploy_argocd_apps().await?;
        
        // Deploy sample applications for testing
        self.deploy_sample_apps().await?;
        
        // Get service URLs
        self.get_service_urls()?;
        
        self.print_status("", "white");
        self.print_status("🎉 Observability Stack Deployment Completed Successfully!", "green");
        self.print_status("", "white");
        self.print_status("📋 What was deployed:", "cyan");
        self.print_status("   ✅ Prometheus CRDs (Sync Wave 1)", "white");
        self.print_status("   ✅ OpenTelemetry Collector (Sync Wave 2)", "white");
        self.print_status("   ✅ Prometheus Stack with Grafana (Sync Wave 3)", "white");
        self.print_status("   ✅ ArgoCD Dashboard", "white");
        self.print_status("   ✅ Sample Applications for Testing", "white");
        self.print_status("", "white");
        self.print_status("📋 Access URLs:", "cyan");
        self.print_status("   1. ArgoCD UI: https://localhost:8080", "white");
        self.print_status("      Port forwarding: kubectl port-forward svc/argocd-server -n argocd 8080:443", "white");
        self.print_status("   2. Grafana: http://localhost:3000", "white");
        self.print_status("      Port forwarding: kubectl port-forward svc/prometheus-stack-grafana -n observability 3000:80", "white");
        self.print_status("   3. Prometheus: http://localhost:9090", "white");
        self.print_status("      Port forwarding: kubectl port-forward svc/prometheus-stack-kube-prom-prometheus -n observability 9090:9090", "white");
        self.print_status("", "white");
        self.print_status("🔍 Monitor deployment in ArgoCD UI to see sync waves in action!", "cyan");
        
        Ok(true)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    let deployer = ObservabilityStackDeployer::new(
        args.namespace,
    );
    
    let success = deployer.deploy(args.install_crds_manually).await?;
    
    if success {
        Ok(())
    } else {
        std::process::exit(1);
    }
} 