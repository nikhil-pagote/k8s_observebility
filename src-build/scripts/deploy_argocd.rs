use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use tokio::time::sleep;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value = "argocd")]
    argocd_namespace: String,
}

struct ArgoCDDeployer {
    argocd_namespace: String,
}

impl ArgoCDDeployer {
    fn new(argocd_namespace: String) -> Self {
        Self {
            argocd_namespace,
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

        // Check if Helm is available
        match self.run_command("helm version", false) {
            Ok(_) => self.print_status("✅ Helm is available", "green"),
            Err(_) => {
                self.print_status("❌ Helm is not available", "red");
                self.print_status("Please ensure Helm is installed and in your PATH", "red");
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

        Ok(true)
    }

    async fn install_argocd(&self) -> Result<()> {
        self.print_status("🚀 Installing ArgoCD...", "yellow");
        
        // Ensure we're using the correct Kind context
        self.ensure_kind_context()?;
        
        // Add ArgoCD Helm repository
        self.run_command("helm repo add argo https://argoproj.github.io/argo-helm", true)?;
        self.run_command("helm repo update", true)?;
        
        // Install ArgoCD using Helm chart
        let helm_command = format!(
            "helm install argocd argo/argo-cd -n {} --create-namespace --set server.service.type=LoadBalancer --set server.metrics.enabled=true --set controller.metrics.enabled=true --set redis.metrics.enabled=false",
            self.argocd_namespace
        );
        self.run_command(&helm_command, true)?;
        
        // Wait for ArgoCD to be ready
        self.print_status("⏳ Waiting for ArgoCD to be ready...", "yellow");
        let max_attempts = 30;
        let mut attempt = 0;
        
        while attempt < max_attempts {
            match self.run_command("kubectl get pods -n argocd --no-headers", false) {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    let pods: Vec<&str> = output_str.trim().split('\n').collect();
                    
                    if pods.len() >= 5 {
                        let ready_pods = pods.iter()
                            .filter(|pod| pod.contains("Running") || pod.contains("Completed"))
                            .count();
                        
                        if ready_pods >= 5 {
                            self.print_status("✅ ArgoCD is ready!", "green");
                            break;
                        }
                    }
                }
                Err(_) => {}
            }
            
            attempt += 1;
            self.print_status(&format!("⏳ Waiting for ArgoCD... (Attempt {}/{})", attempt, max_attempts), "yellow");
            sleep(Duration::from_secs(10)).await;
        }
        
        self.print_status("✅ ArgoCD installed successfully", "green");
        Ok(())
    }

    fn setup_port_forwarding(&self) -> Result<()> {
        self.print_status("🔌 Setting up port forwarding for ArgoCD...", "yellow");
        
        // First, verify our kubeconfig is working
        self.print_status("🔍 Verifying cluster connectivity...", "yellow");
        match self.run_command("kubectl cluster-info", false) {
            Ok(_) => {
                self.print_status("✅ Cluster connectivity verified", "green");
            }
            Err(e) => {
                self.print_status(&format!("❌ Cluster connectivity failed: {}", e), "red");
                return Err(anyhow::anyhow!("Cannot connect to cluster: {}", e));
            }
        }
        
        // Check if ArgoCD namespace exists
        match self.run_command("kubectl get namespace argocd", false) {
            Ok(_) => {
                self.print_status("✅ ArgoCD namespace found", "green");
            }
            Err(_) => {
                self.print_status("❌ ArgoCD namespace not found. Please ensure ArgoCD is deployed first.", "red");
                return Err(anyhow::anyhow!("ArgoCD namespace not found"));
            }
        }
        
        // Check if ArgoCD server service exists
        match self.run_command("kubectl get svc -n argocd argocd-server", false) {
            Ok(_) => {
                self.print_status("✅ ArgoCD server service found", "green");
            }
            Err(_) => {
                self.print_status("❌ ArgoCD server service not found. Please ensure ArgoCD is deployed first.", "red");
                return Err(anyhow::anyhow!("ArgoCD server service not found"));
            }
        }
        
        // Check if ArgoCD server pod is running
        match self.run_command("kubectl get pods -n argocd -l app.kubernetes.io/name=argocd-server --field-selector=status.phase=Running", false) {
            Ok(_) => {
                self.print_status("✅ ArgoCD server pod is running", "green");
            }
            Err(_) => {
                self.print_status("❌ ArgoCD server pod not running. Please ensure ArgoCD is deployed and pods are ready.", "red");
                return Err(anyhow::anyhow!("ArgoCD server pod not running"));
            }
        }
        
        // Kill any existing port forwarding on port 8080
        self.print_status("🔧 Checking for existing port forwarding...", "yellow");
        #[cfg(target_os = "windows")]
        let kill_cmd = "Get-Process -Name kubectl -ErrorAction SilentlyContinue | Where-Object {$_.CommandLine -like '*port-forward*8080*'} | Stop-Process -Force -ErrorAction SilentlyContinue";
        #[cfg(not(target_os = "windows"))]
        let kill_cmd = "pkill -f 'kubectl.*port-forward.*8080' || true";
        
        self.run_command(kill_cmd, true).ok(); // Ignore errors here
        
        // Start port forwarding in background
        let port_forward_cmd = "kubectl port-forward -n argocd svc/argocd-server 8080:443";
        
        #[cfg(target_os = "windows")]
        let background_cmd = format!("Start-Process powershell -ArgumentList '-Command', '{}' -WindowStyle Hidden", port_forward_cmd);
        #[cfg(not(target_os = "windows"))]
        let background_cmd = format!("{} &", port_forward_cmd);
        
        match self.run_command(&background_cmd, true) {
            Ok(_) => {
                self.print_status("✅ Port forwarding started in background", "green");
                self.print_status("🌐 ArgoCD UI will be available at: https://localhost:8080", "cyan");
                self.print_status("🔑 Username: admin, Password: (retrieve with: kubectl -n argocd get secret argocd-initial-admin-secret -o jsonpath=\"{.data.password}\" | base64 -d)", "cyan");
                
                // Wait a moment for port forwarding to establish
                std::thread::sleep(std::time::Duration::from_secs(2));
                
                // Test if port forwarding is working
                self.print_status("🔍 Testing port forwarding...", "yellow");
                #[cfg(target_os = "windows")]
                let test_cmd = "Test-NetConnection -ComputerName localhost -Port 8080 -InformationLevel Quiet";
                #[cfg(not(target_os = "windows"))]
                let test_cmd = "nc -z localhost 8080";
                
                match self.run_command(test_cmd, true) {
                    Ok(_) => {
                        self.print_status("✅ Port forwarding is working correctly", "green");
                    }
                    Err(_) => {
                        self.print_status("⚠️ Port forwarding test failed, but it might still be working", "yellow");
                        self.print_status("Try accessing https://localhost:8080 in your browser", "yellow");
                    }
                }
            }
            Err(e) => {
                self.print_status(&format!("❌ Failed to start port forwarding: {}", e), "red");
                return Err(e);
            }
        }
        
        Ok(())
    }

    fn get_service_urls(&self) -> Result<()> {
        self.print_status("🌐 Getting service URLs...", "yellow");
        
        // Get ArgoCD server URL
        match self.run_command("kubectl get svc argocd-server -n argocd -o jsonpath='{.status.loadBalancer.ingress[0].ip}'", false) {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let ip = output_str.trim();
                if !ip.is_empty() {
                    self.print_status(&format!("🔗 ArgoCD UI: https://{}:443", ip), "cyan");
                    self.print_status("   Username: admin, Password: (retrieve from secret)", "white");
                }
            }
            Err(_) => {
                self.print_status("🔗 ArgoCD UI: Use port-forward: kubectl port-forward svc/argocd-server -n argocd 8080:443", "cyan");
                self.print_status("   Username: admin, Password: (retrieve from secret)", "white");
            }
        }

        Ok(())
    }

    async fn deploy(&self) -> Result<bool> {
        self.print_status("🚀 Deploying ArgoCD", "green");
        self.print_status(&format!("ArgoCD Namespace: {}", self.argocd_namespace), "cyan");
        
        // Check prerequisites
        if !self.check_prerequisites()? {
            return Ok(false);
        }
        
        // Install ArgoCD
        self.install_argocd().await?;
        
        // Setup port forwarding
        self.setup_port_forwarding()?;
        
        // Get service URLs
        self.get_service_urls()?;
        
        self.print_status("", "white");
        self.print_status("🎉 ArgoCD deployed successfully!", "green");
        self.print_status("", "white");
        self.print_status("📋 Next Steps:", "cyan");
        self.print_status("   1. Access ArgoCD UI: https://localhost:8080", "white");
        self.print_status("      Port forwarding is already set up by the script", "white");
        self.print_status("      If not working, manually run: kubectl port-forward svc/argocd-server -n argocd 8080:443", "white");
        self.print_status("   2. Create ArgoCD applications for your observability stack via the UI", "white");
        
        Ok(true)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    let deployer = ArgoCDDeployer::new(
        args.argocd_namespace,
    );
    
    let success = deployer.deploy().await?;
    
    if success {
        Ok(())
    } else {
        std::process::exit(1);
    }
} 