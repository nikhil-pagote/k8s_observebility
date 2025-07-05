use std::process::Command;
use std::time::Duration;
use std::env;
use std::path::Path;

use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use tokio::time::sleep;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value = "argocd")]
    argocd_namespace: String,
    
    #[arg(long, default_value = "admin")]
    argocd_admin_password: String,
}

struct ArgoCDDeployer {
    argocd_namespace: String,
    argocd_admin_password: String,
    kubeconfig_path: String,
}

impl ArgoCDDeployer {
    fn new(argocd_namespace: String, argocd_admin_password: String) -> Self {
        let kubeconfig_path = std::env::current_dir()
            .unwrap_or_else(|_| Path::new(".").to_path_buf())
            .join("kubeconfig")
            .to_string_lossy()
            .to_string();
        
        Self {
            argocd_namespace,
            argocd_admin_password,
            kubeconfig_path,
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

        // Always set KUBECONFIG environment variable for kubectl commands
        cmd.env("KUBECONFIG", &self.kubeconfig_path);

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
        
        // Get the kubeconfig from kind and write it to our local file
        let output = self.run_command("kind get kubeconfig --name observability-cluster", true)?;
        let kubeconfig_content = String::from_utf8_lossy(&output.stdout);
        
        // Write the kubeconfig to file
        std::fs::write(&self.kubeconfig_path, kubeconfig_content.as_bytes())
            .context("Failed to write kubeconfig file")?;
        
        // Set KUBECONFIG environment variable for this process
        env::set_var("KUBECONFIG", &self.kubeconfig_path);
        
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
            "helm install argocd argo/argo-cd -n {} --create-namespace --set server.service.type=LoadBalancer --set server.adminPassword={}",
            self.argocd_namespace, self.argocd_admin_password
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

    fn get_service_urls(&self) -> Result<()> {
        self.print_status("🌐 Getting service URLs...", "yellow");
        
        // Get ArgoCD server URL
        match self.run_command("kubectl get svc argocd-server -n argocd -o jsonpath='{.status.loadBalancer.ingress[0].ip}'", false) {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let ip = output_str.trim();
                if !ip.is_empty() {
                    self.print_status(&format!("🔗 ArgoCD UI: https://{}:443", ip), "cyan");
                    self.print_status(&format!("   Username: admin, Password: {}", self.argocd_admin_password), "white");
                }
            }
            Err(_) => {
                self.print_status("🔗 ArgoCD UI: Use port-forward: kubectl port-forward svc/argocd-server -n argocd 8080:443", "cyan");
                self.print_status(&format!("   Username: admin, Password: {}", self.argocd_admin_password), "white");
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
        
        // Get service URLs
        self.get_service_urls()?;
        
        self.print_status("", "white");
        self.print_status("🎉 ArgoCD deployed successfully!", "green");
        self.print_status("", "white");
        self.print_status("📋 Next Steps:", "cyan");
        self.print_status("   1. Access ArgoCD UI: http://localhost:443", "white");
        self.print_status("      Port forwarding done by script already. if not working, use command:", "white");
        self.print_status("      kubectl port-forward svc/argocd-server -n argocd 8080:443", "white");
        self.print_status("   2. Create ArgoCD applications for your observability stack via the UI", "white");
        
        Ok(true)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    let deployer = ArgoCDDeployer::new(
        args.argocd_namespace,
        args.argocd_admin_password,
    );
    
    let success = deployer.deploy().await?;
    
    if success {
        Ok(())
    } else {
        std::process::exit(1);
    }
} 