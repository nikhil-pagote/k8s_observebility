use clap::{Parser, Subcommand};
use colored::*;
use std::process::{Command, Stdio};
use std::io;
use std::thread;
use std::time::Duration;
use anyhow::{Result, Context};

#[derive(Parser)]
#[command(name = "k8s-obs")]
#[command(about = "Kubernetes Observability Stack Management Tool")]
#[command(version = "1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    #[arg(short, long, default_value = "observability")]
    namespace: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Quick start - setup everything from scratch
    QuickStart,
    /// Setup Kind cluster
    SetupCluster,
    /// Deploy ArgoCD
    DeployArgoCD,
    /// Deploy observability stack
    DeployStack,
    /// Deploy sample applications
    DeploySampleApps,
    /// Show status of all components
    Status,
    /// Show logs for key components
    Logs,
    /// Setup port forwarding for local access
    PortForward,
    /// Get service URLs
    GetUrls,
    /// Cleanup applications
    Cleanup,
    /// Complete cleanup including cluster
    CleanAll,
    /// Development environment setup
    DevSetup,
    /// Show help information
    Help,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::QuickStart => quick_start(&cli.namespace)?,
        Commands::SetupCluster => setup_cluster()?,
        Commands::DeployArgoCD => deploy_argocd()?,
        Commands::DeployStack => deploy_stack(&cli.namespace)?,
        Commands::DeploySampleApps => deploy_sample_apps(&cli.namespace)?,
        Commands::Status => show_status(&cli.namespace)?,
        Commands::Logs => show_logs(&cli.namespace)?,
        Commands::PortForward => port_forward(&cli.namespace)?,
        Commands::GetUrls => get_urls(&cli.namespace)?,
        Commands::Cleanup => cleanup(&cli.namespace)?,
        Commands::CleanAll => clean_all(&cli.namespace)?,
        Commands::DevSetup => dev_setup()?,
        Commands::Help => show_help(),
    }
    
    Ok(())
}

fn print_status(message: &str, color: &str) {
    let colored_message = match color {
        "green" => message.green(),
        "yellow" => message.yellow(),
        "red" => message.red(),
        "cyan" => message.cyan(),
        _ => message.white(),
    };
    println!("{}", colored_message);
}

fn run_command(cmd: &str, description: &str) -> Result<()> {
    print_status(&format!("🔄 {}", description), "cyan");
    print_status(&format!("📋 Executing: {}", cmd), "cyan");
    
    let output = Command::new("cmd")
        .args(&["/C", cmd])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context(format!("Failed to execute: {}", cmd))?;
    
    if output.status.success() {
        print_status("✅ Command completed successfully", "green");
        if !output.stdout.is_empty() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
    } else {
        print_status("❌ Command failed", "red");
        print_status(&format!("Failed command: {}", cmd), "red");
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
        anyhow::bail!("Command failed: {}", cmd);
    }
    
    Ok(())
}

fn check_prerequisites() -> Result<()> {
    print_status("🔍 Checking prerequisites...", "yellow");
    
    let tools = vec!["kubectl", "kind", "docker"];
    
    for tool in tools {
        let output = Command::new("cmd")
            .args(&["/C", &format!("where {}", tool)])
            .output();
        
        match output {
            Ok(_) => print_status(&format!("✅ {} is available", tool), "green"),
            Err(_) => {
                print_status(&format!("❌ {} is required but not installed", tool), "red");
                anyhow::bail!("Missing prerequisite: {}", tool);
            }
        }
    }
    
    print_status("✅ All prerequisites are satisfied", "green");
    Ok(())
}

fn check_binaries() -> Result<()> {
    let binaries = vec!["setup_kind_cluster.exe", "deploy_argocd.exe"];
    let mut missing = false;
    
    for binary in &binaries {
        if !std::path::Path::new(&format!("bin/{}", binary)).exists() {
            print_status(&format!("❌ {} not found in bin/ directory", binary), "red");
            missing = true;
        }
    }
    
    if missing {
        print_status("🔨 Please run .\\build-scripts.ps1 to build the required binaries", "yellow");
        anyhow::bail!("Required binaries are missing. Run .\\build-scripts.ps1 first.");
    }
    
    print_status("✅ All required binaries found", "green");
    Ok(())
}



fn setup_cluster() -> Result<()> {
    check_binaries()?;
    print_status("🔧 Setting up Kind cluster...", "yellow");
    run_command("bin\\setup_kind_cluster.exe", "Creating and configuring Kind cluster")?;
    print_status("✅ Kind cluster setup complete", "green");
    Ok(())
}

fn deploy_argocd() -> Result<()> {
    check_binaries()?;
    print_status("🚀 Deploying ArgoCD...", "yellow");
    run_command("bin\\deploy_argocd.exe", "Deploying ArgoCD to Kubernetes cluster")?;
    print_status("✅ ArgoCD deployment complete", "green");
    Ok(())
}

fn deploy_stack(_namespace: &str) -> Result<()> {
    print_status("🚀 Deploying observability stack...", "yellow");
    println!("Deploying Grafana, Prometheus, Jaeger, and ClickHouse applications...");
    run_command("kubectl apply -k argocd-apps/", "Applying ArgoCD applications for observability stack")?;
    print_status("✅ Observability stack deployment complete", "green");
    Ok(())
}

fn deploy_sample_apps(namespace: &str) -> Result<()> {
    print_status("🚀 Deploying sample applications...", "yellow");
    let cmd = format!(
        "kubectl apply -f apps/load-generator/ -f apps/sample-app/deployment-basic.yaml -n {}",
        namespace
    );
    run_command(&cmd, "Deploying load generator and sample applications")?;
    print_status("✅ Sample applications deployed", "green");
    Ok(())
}

fn show_status(namespace: &str) -> Result<()> {
    print_status("📊 Cluster Status", "cyan");
    println!("=================");
    
    let pods_cmd = format!("kubectl get pods -n {}", namespace);
    let services_cmd = format!("kubectl get services -n {}", namespace);
    
    let commands = vec![
        ("kubectl get nodes", "Nodes"),
        ("kubectl get namespaces", "Namespaces"),
        ("kubectl get applications -n argocd", "ArgoCD Applications"),
        ("kubectl get pods -n argocd", "ArgoCD Pods"),
        (pods_cmd.as_str(), "Observability Pods"),
        (services_cmd.as_str(), "Services"),
    ];
    
    for (cmd, title) in commands {
        println!("\n{}:", title);
        println!("{}", "─".repeat(title.len() + 1));
        
        let output = Command::new("cmd")
            .args(&["/C", cmd])
            .output();
        
        match output {
            Ok(output) => {
                if output.status.success() {
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                } else {
                    println!("Error: {}", String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => println!("Error: {}", e),
        }
    }
    
    Ok(())
}

fn show_logs(namespace: &str) -> Result<()> {
    print_status("📋 Component Logs", "cyan");
    println!("=================");
    
    let prometheus_logs = format!("kubectl logs -n {} deployment/prometheus-server --tail=20", namespace);
    let grafana_logs = format!("kubectl logs -n {} deployment/grafana --tail=20", namespace);
    let clickhouse_logs = format!("kubectl logs -n {} deployment/clickhouse --tail=20", namespace);
    let jaeger_logs = format!("kubectl logs -n {} deployment/jaeger-query --tail=20", namespace);
    let otel_logs = format!("kubectl logs -n {} deployment/opentelemetry-collector --tail=20", namespace);
    
    let log_commands = vec![
        ("kubectl logs -n argocd deployment/argocd-server --tail=20", "ArgoCD Server"),
        ("kubectl logs -n argocd deployment/argocd-application-controller --tail=20", "ArgoCD Application Controller"),
        (prometheus_logs.as_str(), "Prometheus"),
        (grafana_logs.as_str(), "Grafana"),
        (clickhouse_logs.as_str(), "ClickHouse"),
        (jaeger_logs.as_str(), "Jaeger"),
        (otel_logs.as_str(), "OpenTelemetry Collector"),
    ];
    
    for (cmd, title) in log_commands {
        println!("\n{}:", title);
        println!("{}", "─".repeat(title.len() + 1));
        
        let output = Command::new("cmd")
            .args(&["/C", cmd])
            .output();
        
        match output {
            Ok(output) => {
                if output.status.success() {
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                } else {
                    println!("Error: {}", String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => println!("Error: {}", e),
        }
    }
    
    Ok(())
}

fn port_forward(namespace: &str) -> Result<()> {
    print_status("🔗 Setting up port forwarding...", "cyan");
    println!();
    print_status("🌐 Service URLs & Credentials", "cyan");
    println!("{}", "=".repeat(40));
    println!();
    println!("📊 Grafana Dashboard:");
    println!("   URL: http://localhost:3000");
    println!("   Username: admin");
    println!("   Password: kubectl get secret -n observability grafana-admin -o jsonpath='{{.data.GF_SECURITY_ADMIN_PASSWORD}}' | base64 -d");
    println!();
    println!("🚀 ArgoCD GitOps UI:");
    println!("   URL: http://localhost:8080");
    println!("   Username: admin");
    println!("   Password: kubectl get secret -n argocd argocd-initial-admin-secret -o jsonpath='{{.data.password}}' | base64 -d");
    println!();
    println!("📈 Prometheus Metrics:");
    println!("   URL: http://localhost:9090");
    println!("   No authentication required");
    println!();
    println!("🔍 Jaeger Tracing:");
    println!("   URL: http://localhost:16686");
    println!("   No authentication required");
    println!();
    println!("🗄️ ClickHouse Database:");
    println!("   URL: http://localhost:8123");
    println!("   Username: default");
    println!("   Password: kubectl get secret -n observability clickhouse -o jsonpath='{{.data.password}}' | base64 -d");
    println!();
    println!("{}", "=".repeat(40));
    println!();
    println!("Note: If port-forward fails, check pod status with:");
    println!("  kubectl get pods -n {}", namespace);
    println!("  kubectl describe pod -n {} <pod-name>", namespace);
    println!();
    println!("Press Enter to stop port forwarding...");
    println!();
    
    print_status("🚀 Starting background port-forward jobs...", "cyan");
    
    let pf_commands = vec![
        ("ArgoCD", format!("kubectl port-forward -n argocd svc/argocd-server 8080:80")),
        ("Grafana", format!("kubectl port-forward -n {} svc/grafana 3000:3000", namespace)),
        ("Prometheus", format!("kubectl port-forward -n {} svc/prometheus-server 9090:80", namespace)),
        ("Jaeger", format!("kubectl port-forward -n {} svc/jaeger-query 16686:16686", namespace)),
        ("ClickHouse", format!("kubectl port-forward -n {} svc/clickhouse 8123:8123", namespace)),
    ];
    
    for (name, cmd) in &pf_commands {
        print_status(&format!("[Port-Forward] {}: {}", name, cmd), "cyan");
        
        let mut child = Command::new("cmd")
            .args(&["/C", cmd])
            .spawn()
            .context(format!("Failed to start port-forward for {}", name))?;
        
        // Give it a moment to start
        thread::sleep(Duration::from_millis(500));
        
        // Check if it's still running
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    print_status(&format!("❌ Port-forward for {} failed to start", name), "red");
                }
            }
            Ok(None) => {
                print_status(&format!("✅ {} port-forward started", name), "green");
            }
            Err(e) => {
                print_status(&format!("❌ Error checking {} port-forward: {}", name, e), "red");
            }
        }
    }
    
    print_status("✅ All port-forward jobs started successfully", "green");
    println!("Background jobs running: {} jobs", pf_commands.len());
    println!();
    println!("Press Enter to stop all port-forward jobs...");
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    print_status("🛑 Stopping all port-forward jobs...", "cyan");
    
    // Kill any kubectl port-forward processes
    let _ = Command::new("taskkill")
        .args(&["/F", "/IM", "kubectl.exe"])
        .output();
    
    print_status("✅ All port-forward jobs stopped", "green");
    Ok(())
}

fn get_urls(namespace: &str) -> Result<()> {
    print_status("🌐 Service URLs", "cyan");
    println!("==============");
    
    let services = vec![
        ("argocd-server", "argocd", "ArgoCD UI"),
        ("grafana", namespace, "Grafana"),
        ("prometheus-server", namespace, "Prometheus"),
        ("clickhouse", namespace, "ClickHouse"),
        ("jaeger-query", namespace, "Jaeger UI"),
    ];
    
    for (service, ns, name) in services {
        println!("\n{}:", name);
        
        let cmd = format!("kubectl get svc {} -n {} -o jsonpath='{{.status.loadBalancer.ingress[0].ip}}'", service, ns);
        let output = Command::new("cmd")
            .args(&["/C", &cmd])
            .output();
        
        match output {
            Ok(output) => {
                if output.status.success() && !output.stdout.is_empty() {
                    let url = String::from_utf8_lossy(&output.stdout);
                    println!("{}", url);
                } else {
                    let default_port = match name {
                        "ArgoCD UI" => "8080",
                        "Grafana" => "3000",
                        "Prometheus" => "9090",
                        "ClickHouse" => "8123",
                        "Jaeger UI" => "16686",
                        _ => "80",
                    };
                    println!("http://localhost:{} (use port-forward)", default_port);
                }
            }
            Err(_) => {
                let default_port = match name {
                    "ArgoCD UI" => "8080",
                    "Grafana" => "3000",
                    "Prometheus" => "9090",
                    "ClickHouse" => "8123",
                    "Jaeger UI" => "16686",
                    _ => "80",
                };
                println!("http://localhost:{} (use port-forward)", default_port);
            }
        }
    }
    
    Ok(())
}

fn cleanup(namespace: &str) -> Result<()> {
    print_status("🧹 Cleaning up applications...", "yellow");
    
    let sample_apps_cmd = format!("kubectl delete -f apps/load-generator/ -f apps/sample-app/ -n {} --ignore-not-found=true", namespace);
    let namespace_cmd = format!("kubectl delete namespace {} --ignore-not-found=true", namespace);
    
    let cleanup_commands = vec![
        ("kubectl delete application --all -n argocd", "Removing all ArgoCD applications"),
        (sample_apps_cmd.as_str(), "Removing sample applications"),
        (namespace_cmd.as_str(), "Removing observability namespace"),
    ];
    
    for (cmd, description) in cleanup_commands {
        run_command(cmd, description)?;
    }
    
    print_status("✅ Cleanup complete", "green");
    Ok(())
}

fn clean_all(namespace: &str) -> Result<()> {
    print_status("🧹 Complete cleanup...", "yellow");
    
    cleanup(namespace)?;
    
    let cleanup_commands = vec![
        ("kind delete cluster --name observability-cluster", "Deleting Kind cluster"),
        ("docker system prune -f", "Cleaning Docker system"),
    ];
    
    for (cmd, description) in cleanup_commands {
        run_command(cmd, description)?;
    }
    
    // Clean up temporary files
    if std::path::Path::new("tmp_crds").exists() {
        print_status("🗑️ Removing temporary CRD files...", "cyan");
        std::fs::remove_dir_all("tmp_crds")?;
    }
    
    // Clean binaries
    if std::path::Path::new("bin").exists() {
        print_status("🧹 Cleaning binaries...", "yellow");
        std::fs::remove_dir_all("bin")?;
        print_status("✅ Binaries cleaned", "green");
    }
    
    print_status("✅ Complete cleanup finished", "green");
    Ok(())
}

fn dev_setup() -> Result<()> {
    print_status("🔧 Setting up development environment...", "yellow");
    
    check_prerequisites()?;
    check_binaries()?;
    setup_cluster()?;
    deploy_argocd()?;
    
    print_status("🔧 Development environment ready", "green");
    Ok(())
}

fn quick_start(namespace: &str) -> Result<()> {
    print_status("🎉 Starting complete setup...", "yellow");
    
    setup_cluster()?;
    deploy_argocd()?;
    deploy_stack(namespace)?;
    deploy_sample_apps(namespace)?;
    
    print_status("🎉 Quick start complete! Your observability stack is ready.", "green");
    Ok(())
}

fn show_help() {
    println!("{}", "Kubernetes Observability Stack Management".cyan());
    println!("{}", "==========================================".cyan());
    println!();
    println!("Available commands:");
    println!("  quick-start       - Complete setup from scratch");
    println!("  setup-cluster     - Create and configure Kind cluster");
    println!("  deploy-argocd     - Deploy ArgoCD to the cluster");
    println!("  deploy-stack      - Deploy observability stack via ArgoCD");
    println!("  deploy-sample-apps - Deploy sample applications for testing");
    println!("  status           - Show status of all components");
    println!("  logs             - Show logs for key components");
    println!("  port-forward     - Set up port forwarding for local access");
    println!("  get-urls         - Get service URLs");
    println!("  cleanup          - Remove sample apps and ArgoCD apps");
    println!("  clean-all        - Remove everything including Kind cluster");
    println!("  dev-setup        - Development environment setup");
    println!("  help             - Show this help message");
    println!();
    println!("Components:");
    println!("  📊 Grafana - Metrics visualization");
    println!("  📈 Prometheus - Metrics collection");
    println!("  🔍 Jaeger - Distributed tracing");
    println!("  🗄️ ClickHouse - Data storage");
    println!("  📡 OpenTelemetry Collector - Data collection");
    println!();
    println!("Access URLs (after port-forward):");
    println!("  ArgoCD UI: http://localhost:8080 (admin/admin)");
    println!("  Grafana: http://localhost:3000 (admin/$(kubectl get secret -n observability grafana-admin -o jsonpath='{{.data.GF_SECURITY_ADMIN_PASSWORD}}' | base64 -d))");
    println!("  Prometheus: http://localhost:9090");
    println!("  Jaeger UI: http://localhost:16686");
    println!("  ClickHouse: http://localhost:8123");
    println!();
    println!("Usage: k8s-obs <command> [options]");
    println!("Example: k8s-obs deploy-stack");
} 