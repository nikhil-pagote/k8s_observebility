use std::env;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use clap::Parser;
use colored::*;

#[derive(Parser)]
#[command(name = "cleanup")]
#[command(about = "Cleanup Kind cluster and all resources")]
struct Args {
    #[arg(long, default_value = "observability-cluster")]
    cluster_name: String,
    
    #[arg(long, default_value = "observability")]
    namespace: String,
}

struct Cleanup {
    cluster_name: String,
    namespace: String,
}

impl Cleanup {
    fn new(cluster_name: String, namespace: String) -> Self {
        Self {
            cluster_name,
            namespace,
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
        let output = Command::new("cmd")
            .args(&["/C", command])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context(format!("Failed to execute command: {}", command))?;

        if check && !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            self.print_status(&format!("❌ Command failed: {}", command), "red");
            self.print_status(&format!("Error: {}", error), "red");
            anyhow::bail!("Command failed: {}", command);
        }

        Ok(output)
    }

    fn uninstall_helm_releases(&self) -> Result<()> {
        self.print_status("📦 Uninstalling Helm releases...", "yellow");
        
        if std::path::Path::new("./kubeconfig").exists() {
            env::set_var("KUBECONFIG", "./kubeconfig");
            
            let releases = vec!["prometheus", "grafana", "opentelemetry"];
            for release in releases {
                let _ = self.run_command(&format!("helm uninstall {} -n {}", release, self.namespace), false);
            }
            
            self.print_status("✅ Helm releases uninstalled", "green");
        } else {
            self.print_status("ℹ️  Kubeconfig not found, skipping Helm uninstall", "blue");
        }
        
        Ok(())
    }

    fn remove_kubernetes_resources(&self) -> Result<()> {
        self.print_status("🗑️  Removing Kubernetes resources...", "yellow");
        
        if std::path::Path::new("./kubeconfig").exists() {
            env::set_var("KUBECONFIG", "./kubeconfig");
            let _ = self.run_command(&format!("kubectl delete namespace {} --ignore-not-found=true", self.namespace), false);
            self.print_status("✅ Kubernetes resources removed", "green");
        } else {
            self.print_status("ℹ️  Kubeconfig not found, skipping Kubernetes cleanup", "blue");
        }
        
        Ok(())
    }

    fn remove_kind_cluster(&self) -> Result<()> {
        self.print_status(&format!("🛑 Deleting Kind cluster: {}", self.cluster_name), "yellow");
        let _ = self.run_command(&format!("kind delete cluster --name {}", self.cluster_name), false);
        self.print_status("✅ Kind cluster deleted", "green");
        Ok(())
    }

    fn remove_local_files(&self) -> Result<()> {
        self.print_status("🗑️  Removing local files...", "yellow");
        
        let files_to_remove = vec!["./kubeconfig", "./helm.zip", "./kind-config.yaml"];
        for file in files_to_remove {
            if std::path::Path::new(file).exists() {
                std::fs::remove_file(file)
                    .context(format!("Failed to remove file: {}", file))?;
            }
        }
        
        if std::path::Path::new("./helm").exists() {
            std::fs::remove_dir_all("./helm")
                .context("Failed to remove helm directory")?;
        }
        
        self.print_status("✅ Local files removed", "green");
        Ok(())
    }

    fn cleanup(&self) -> Result<()> {
        self.print_status("⚠️  This will remove the entire Kind cluster and all resources!", "red");
        println!("Are you sure you want to continue? (y/N): ");
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)
            .context("Failed to read user input")?;
        
        if input.trim().to_lowercase() != "y" {
            self.print_status("❌ Cleanup cancelled", "yellow");
            return Ok(());
        }

        self.uninstall_helm_releases()?;
        self.remove_kubernetes_resources()?;
        self.remove_kind_cluster()?;
        self.remove_local_files()?;
        
        self.print_status("\n🎉 Cleanup completed successfully!", "green");
        self.print_status("All Kind cluster resources have been removed.", "white");
        
        Ok(())
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    let cleanup = Cleanup::new(args.cluster_name, args.namespace);
    cleanup.cleanup()?;
    
    Ok(())
} 