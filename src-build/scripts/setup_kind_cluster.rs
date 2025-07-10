use std::env;
use std::process::{Command, Stdio};
use std::time::Duration;


use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;


#[derive(Parser)]
#[command(name = "setup_kind_cluster")]
#[command(about = "Setup Kind cluster for Kubernetes observability")]
struct Args {
    #[arg(long, default_value = "observability-cluster")]
    cluster_name: String,
    
    #[arg(long, default_value = "v1.33.1")]
    kubernetes_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Networking {
    api_server_address: String,
    api_server_port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
struct KindConfig {
    kind: String,
    #[serde(rename = "apiVersion")]
    api_version: String,
    networking: Networking,
    nodes: Vec<KindNode>,
}

#[derive(Debug, Serialize, Deserialize)]
struct KindNode {
    role: String,
    image: String,
    #[serde(rename = "kubeadmConfigPatches")]
    #[serde(skip_serializing_if = "Option::is_none")]
    kubeadm_config_patches: Option<Vec<String>>,
    #[serde(rename = "extraPortMappings")]
    #[serde(skip_serializing_if = "Option::is_none")]
    extra_port_mappings: Option<Vec<PortMapping>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PortMapping {
    container_port: u16,
    host_port: u16,
    protocol: String,
}

struct KindClusterSetup {
    kubernetes_version: String,
    cluster_name: String,
}

impl KindClusterSetup {
    fn new(kubernetes_version: String, cluster_name: String) -> Self {
        Self {
            kubernetes_version,
            cluster_name,
        }
    }

    fn print_status(&self, message: &str, color: &str) {
        let colored_message = match color {
            "green" => message.green(),
            "yellow" => message.yellow(),
            "red" => message.red(),
            "cyan" => message.cyan(),
            "blue" => message.blue(),
            _ => message.white(),
        };
        println!("{}", colored_message);
    }

    fn run_command(&self, command: &str, check: bool) -> Result<std::process::Output> {
        let output = if cfg!(target_os = "windows") {
            Command::new("powershell")
                .args(&["-NoProfile", "-Command", command])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
        } else {
            Command::new("sh")
                .args(&["-c", command])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
        }.context(format!("Failed to execute command: {}", command))?;

        if check && !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            self.print_status(&format!("❌ Command failed: {}", command), "red");
            self.print_status(&format!("Error: {}", error), "red");
            anyhow::bail!("Command failed: {}", command);
        }

        Ok(output)
    }

    fn check_docker_running(&self) -> Result<bool> {
        match self.run_command("docker version", false) {
            Ok(_) => {
                self.print_status("✅ Docker is running", "green");
                
                // Check Docker resources
                match self.run_command("docker system df", false) {
                    Ok(output) => {
                        let output_str = String::from_utf8_lossy(&output.stdout);
                        if output_str.contains("GB") {
                            self.print_status("⚠️ Docker has significant resource usage. Consider running 'docker system prune' to free up space.", "yellow");
                        }
                    }
                    Err(_) => {}
                }
                
                Ok(true)
            }
            Err(_) => {
                self.print_status("❌ Docker is not running. Please start Docker Desktop first.", "red");
                Ok(false)
            }
        }
    }

    fn check_kind_installed(&self) -> Result<bool> {
        match self.run_command("kind version", false) {
            Ok(output) => {
                let version = String::from_utf8_lossy(&output.stdout);
                self.print_status(&format!("✅ Kind is installed: {}", version.trim()), "green");
                Ok(true)
            }
            Err(_) => {
                self.print_status("❌ Kind is not installed", "red");
                Ok(false)
            }
        }
    }

    async fn install_kind(&self) -> Result<bool> {
        self.print_status("📦 Installing Kind...", "yellow");
        
        let kind_version = "v0.20.0";
        let kind_url = format!("https://kind.sigs.k8s.io/dl/{}/kind-windows-amd64", kind_version);
        
        // Download Kind binary
        let response = reqwest::get(&kind_url).await
            .context("Failed to download Kind binary")?;
        
        let bytes = response.bytes().await
            .context("Failed to read response bytes")?;
        
        std::fs::write("./kind.exe", &bytes)
            .context("Failed to write Kind binary")?;
        
        // Add current directory to PATH
        let current_path = env::var("PATH").unwrap_or_default();
        let current_dir = env::current_dir()?.to_string_lossy().to_string();
        env::set_var("PATH", format!("{};{}", current_dir, current_path));
        
        self.print_status("✅ Kind installed successfully", "green");
        Ok(true)
    }

    fn create_kind_config(&self) -> Result<String> {
        self.print_status("📝 Creating Kind cluster configuration...", "yellow");
        
        let config = KindConfig {
            kind: "Cluster".to_string(),
            api_version: "kind.x-k8s.io/v1alpha4".to_string(),
            networking: Networking {
                api_server_address: "127.0.0.1".to_string(),
                api_server_port: 6443,
            },
            nodes: vec![
                KindNode {
                    role: "control-plane".to_string(),
                    image: format!("kindest/node:{}", self.kubernetes_version),
                    kubeadm_config_patches: Some(vec![
                        "kind: InitConfiguration\nnodeRegistration:\n  kubeletExtraArgs:\n    node-labels: \"ingress-ready=true\"".to_string()
                    ]),
                    extra_port_mappings: Some(vec![
                        PortMapping {
                            container_port: 30080,
                            host_port: 30080,
                            protocol: "TCP".to_string(),
                        },
                        PortMapping {
                            container_port: 30443,
                            host_port: 30443,
                            protocol: "TCP".to_string(),
                        },
                    ]),
                },
                KindNode {
                    role: "worker".to_string(),
                    image: format!("kindest/node:{}", self.kubernetes_version),
                    kubeadm_config_patches: None,
                    extra_port_mappings: None,
                },
                KindNode {
                    role: "worker".to_string(),
                    image: format!("kindest/node:{}", self.kubernetes_version),
                    kubeadm_config_patches: None,
                    extra_port_mappings: None,
                },
            ],
        };
        
        let yaml = serde_yaml::to_string(&config)
            .context("Failed to serialize Kind config to YAML")?;
        
        std::fs::write("./kind-config.yaml", &yaml)
            .context("Failed to write Kind config file")?;
        
        // Debug: Print the configuration
        self.print_status("📋 Kind configuration:", "cyan");
        for line in yaml.lines() {
            if line.trim().starts_with("- role:") || line.trim().starts_with("nodes:") {
                self.print_status(&format!("   {}", line.trim()), "white");
            }
        }
        
        self.print_status("✅ Kind configuration created", "green");
        Ok("./kind-config.yaml".to_string())
    }

    fn create_kind_cluster(&self, config_path: &str) -> Result<bool> {
        self.print_status(&format!("🚀 Creating Kind cluster: {}", self.cluster_name), "yellow");
        
        let command = format!("kind create cluster --name {} --config {}", self.cluster_name, config_path);
        
        match self.run_command(&command, false) {
            Ok(output) => {
                if output.status.success() {
                    self.print_status("✅ Kind cluster created successfully", "green");
                    // Debug: Print the command output
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if !stdout.trim().is_empty() {
                        self.print_status(&format!("📋 Output: {}", stdout.trim()), "cyan");
                    }
                    if !stderr.trim().is_empty() {
                        self.print_status(&format!("⚠️ Warnings: {}", stderr.trim()), "yellow");
                    }
                    Ok(true)
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    self.print_status(&format!("❌ Failed to create Kind cluster: {}", stderr), "red");
                    Ok(false)
                }
            }
            Err(e) => {
                self.print_status(&format!("❌ Failed to create Kind cluster: {}", e), "red");
                Ok(false)
            }
        }
    }

    

    async fn verify_cluster_setup(&self) -> Result<bool> {
        self.print_status("🔍 Verifying cluster setup...", "yellow");
        
        // Use kind export kubeconfig to get the proper context
        self.run_command(&format!("kind export kubeconfig --name {}", self.cluster_name), false)?;
        
        let max_attempts = 30;
        let mut attempt = 0;
        
        while attempt < max_attempts {
            match self.run_command("kubectl get nodes --no-headers", false) {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    let nodes: Vec<&str> = output_str.trim().split('\n').collect();
                    
                    // Debug: Print what we're seeing
                    if attempt == 0 {
                        self.print_status(&format!("📋 Found {} nodes", nodes.len()), "cyan");
                        for (i, node) in nodes.iter().enumerate() {
                            self.print_status(&format!("   Node {}: {}", i + 1, node), "white");
                        }
                    }
                    
                    if nodes.len() >= 3 {
                        let ready_nodes = nodes.iter()
                            .filter(|node| node.contains("Ready"))
                            .count();
                        
                        if ready_nodes >= 3 {
                            self.print_status("✅ All nodes are ready!", "green");
                            self.run_command("kubectl get nodes", true)?;
                            return Ok(true);
                        } else {
                            self.print_status(&format!("⏳ {}/{} nodes ready", ready_nodes, nodes.len()), "yellow");
                        }
                    } else if !nodes.is_empty() {
                        self.print_status(&format!("⏳ Found {} nodes, waiting for more...", nodes.len()), "yellow");
                    }
                }
                Err(e) => {
                    if attempt == 0 {
                        self.print_status(&format!("⚠️ kubectl error: {}", e), "yellow");
                    }
                }
            }
            
            attempt += 1;
            self.print_status(&format!("⏳ Waiting for nodes to be ready... (Attempt {}/{})", attempt, max_attempts), "yellow");
            sleep(Duration::from_secs(10)).await;
        }
        
        self.print_status("❌ Cluster verification failed", "red");
        self.print_status("📋 Final cluster status:", "yellow");
        let _ = self.run_command("kubectl get nodes", false);
        Ok(false)
    }

    async fn install_helm(&self) -> Result<()> {
        self.print_status("📦 Installing Helm...", "yellow");
        
        // Use PowerShell to install Helm via winget or chocolatey
        let install_commands = vec![
            "winget install --id=Helm.Helm -e",
            "choco install kubernetes-helm -y",
            "scoop install helm"
        ];
        
        for command in install_commands {
            match self.run_command(command, false) {
                Ok(_) => {
                    self.print_status("✅ Helm installed successfully", "green");
                    return Ok(());
                }
                Err(_) => {
                    continue;
                }
            }
        }
        
        // If all package managers fail, try direct download
        self.print_status("📥 Downloading Helm directly...", "yellow");
        
        let helm_version = "v3.13.0";
        let helm_url = format!("https://get.helm.sh/helm-{}-windows-amd64.tar.gz", helm_version);
        
        // Download Helm
        let response = reqwest::get(&helm_url).await
            .context("Failed to download Helm")?;
        
        let bytes = response.bytes().await
            .context("Failed to read Helm response")?;
        
        // Save to temporary file
        std::fs::write("./helm.tar.gz", &bytes)
            .context("Failed to write Helm tar file")?;
        
        // Extract using tar command (available on Windows 10+)
        self.run_command("tar -xzf helm.tar.gz", false)?;
        
        // Move helm.exe to a directory in PATH
        std::fs::create_dir_all("./bin").context("Failed to create bin directory")?;
        std::fs::rename("./windows-amd64/helm.exe", "./bin/helm.exe")
            .context("Failed to move helm.exe")?;
        
        // Add to PATH
        let current_path = env::var("PATH").unwrap_or_default();
        let helm_path = format!("{}/bin", env::current_dir()?.to_string_lossy());
        env::set_var("PATH", format!("{};{}", helm_path, current_path));
        
        // Verify installation
        self.run_command("helm version", true)?;
        
        self.print_status("✅ Helm installed successfully", "green");
        
        // Clean up
        std::fs::remove_file("./helm.tar.gz").ok();
        std::fs::remove_dir_all("./windows-amd64").ok();
        
        Ok(())
    }

    async fn setup(&self) -> Result<bool> {
        self.print_status("🚀 Setting up Kind Cluster for Kubernetes Observability", "green");
        self.print_status(&format!("Cluster Name: {}", self.cluster_name), "cyan");
        self.print_status(&format!("Kubernetes Version: {}", self.kubernetes_version), "cyan");
        
        // Check Docker
        if !self.check_docker_running()? {
            return Ok(false);
        }
        
        // Check for potential port conflicts
        self.print_status("🔍 Checking for potential port conflicts...", "yellow");
        match self.run_command("netstat -an | findstr :6443", false) {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if !output_str.trim().is_empty() {
                    self.print_status("⚠️ Port 6443 might be in use. Consider stopping other services using this port.", "yellow");
                }
            }
            Err(_) => {}
        }
        
        // Check/Install Kind
        if !self.check_kind_installed()? {
            if !self.install_kind().await? {
                return Ok(false);
            }
        }
        
        // Create Kind configuration
        let config_path = self.create_kind_config()?;
        
        // Create Kind cluster
        if !self.create_kind_cluster(&config_path)? {
            return Ok(false);
        }
        
        
            
            if self.verify_cluster_setup().await? {
            self.print_status("🎉 Kind cluster setup completed successfully!", "green");

            // Install Helm
            self.install_helm().await?;

            // Final verification
            self.print_status("🔍 Final cluster verification...", "yellow");
            match self.run_command("kubectl get nodes", false) {
                Ok(_) => self.print_status("✅ Cluster is fully operational", "green"),
                Err(_) => {
                    self.print_status("⚠️ Cluster verification failed, but continuing...", "yellow")
                }
            }

            self.print_status("", "white");
            self.print_status("📋 Cluster Information:", "cyan");
            self.print_status(&format!("   Cluster Name: {}", self.cluster_name), "white");
            self.print_status("   Nodes: 1 control-plane + 2 workers", "white");
            self.print_status("   Kubeconfig: ~/.kube/config (default)", "white");
            self.print_status("", "white");
            self.print_status("🚀 Next Steps:", "cyan");
            self.print_status("   1. Run: .\\bin\\k8s-obs.exe deploy-argocd", "white");
            self.print_status("   2. Access ArgoCD: http://localhost:8080 (after running k8s-obs deploy-argocd)", "white");
            self.print_status("   3. Deploy observability stack: Use k8s-obs deploy-stack", "white");
            self.print_status("   4. Access observability stack: Use k8s-obs port-forward", "white");

            Ok(true)
        } else {
            self.print_status("❌ Cluster setup failed", "red");
            Ok(false)
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    let setup = KindClusterSetup::new(args.kubernetes_version, args.cluster_name);
    
    let success = setup.setup().await?;
    
    if success {
        Ok(())
    } else {
        std::process::exit(1);
    }
} 