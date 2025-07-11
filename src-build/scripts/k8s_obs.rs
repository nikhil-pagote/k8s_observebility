use clap::{Parser, Subcommand};
use colored::*;
use std::process::{Command, Stdio};

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
    /// Setup ingress access for local access
    SetupIngress,
    /// Disable Docker Desktop NGINX ingress controller
    DisableDockerNginx,
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
        Commands::SetupIngress => setup_ingress(&cli.namespace)?,
        Commands::DisableDockerNginx => disable_docker_nginx()?,
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
        let cmd = format!("where {}", tool);
        print_status(&format!("📋 Checking: {}", cmd), "cyan");
        
        let output = Command::new("cmd")
            .args(&["/C", &cmd])
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
    let binaries = vec!["setup_kind_cluster.exe"];
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
    
    // Check if cluster already exists
    let cluster_check = Command::new("kind")
        .args(&["get", "clusters"])
        .output();
    
    match cluster_check {
        Ok(output) => {
            let clusters = String::from_utf8_lossy(&output.stdout);
            if clusters.contains("observability-cluster") {
                print_status("ℹ️  Kind cluster 'observability-cluster' already exists", "yellow");
                print_status("📋 Checking cluster status...", "cyan");
                
                let status_check = Command::new("kubectl")
                    .args(&["cluster-info"])
                    .output();
                
                match status_check {
                    Ok(status_output) => {
                        if status_output.status.success() {
                            print_status("✅ Existing cluster is healthy and ready", "green");
                            return Ok(());
                        } else {
                            print_status("⚠️  Existing cluster may have issues", "yellow");
                        }
                    }
                    Err(_) => {
                        print_status("⚠️  Cannot connect to existing cluster", "yellow");
                    }
                }
            }
        }
        Err(_) => {
            print_status("⚠️  Cannot check existing clusters", "yellow");
        }
    }
    
    run_command("bin\\setup_kind_cluster.exe", "Creating and configuring Kind cluster")?;
    print_status("✅ Kind cluster setup complete", "green");
    Ok(())
}

fn deploy_argocd() -> Result<()> {
    check_binaries()?;
    print_status("🚀 Deploying ArgoCD...", "yellow");
    
    // Check if ArgoCD is already installed
    let argocd_check = Command::new("kubectl")
        .args(&["get", "namespace", "argocd"])
        .output();
    
    match argocd_check {
        Ok(output) => {
            if output.status.success() {
                print_status("ℹ️  ArgoCD namespace already exists", "yellow");
                print_status("📋 Checking ArgoCD deployment status...", "cyan");
                
                let pods_check = Command::new("kubectl")
                    .args(&["get", "pods", "-n", "argocd"])
                    .output();
                
                match pods_check {
                    Ok(pods_output) => {
                        if pods_output.status.success() {
                            let pods = String::from_utf8_lossy(&pods_output.stdout);
                            if pods.contains("Running") {
                                print_status("✅ ArgoCD is already deployed and running", "green");
                                return Ok(());
                            }
                        }
                    }
                    Err(_) => {}
                }
            }
        }
        Err(_) => {}
    }
    
    // Deploy ArgoCD using the separate binary
    run_command("bin\\deploy_argocd.exe", "Deploying ArgoCD to the cluster")?;
    print_status("✅ ArgoCD deployment complete", "green");
    Ok(())
}

fn deploy_stack(namespace: &str) -> Result<()> {
    print_status("🚀 Deploying observability stack...", "yellow");
    
    // Create namespace first
    print_status("📋 Creating observability namespace...", "cyan");
    let namespace_cmd = format!("kubectl create namespace {} --dry-run=client -o yaml | kubectl apply -f -", namespace);
    run_command(&namespace_cmd, "Creating observability namespace")?;
    
    println!("Deploying Grafana, Prometheus, Jaeger, and ClickHouse applications...");
    run_command("kubectl apply -k argocd-apps/", "Applying ArgoCD applications for observability stack")?;
    
    // Wait for Traefik to be ready before deploying ingress resources
    print_status("⏳ Waiting for Traefik to be ready...", "yellow");
    let mut attempts = 0;
    while attempts < 60 { // Wait up to 5 minutes
        thread::sleep(Duration::from_secs(5));
        let traefik_check = Command::new("kubectl")
            .args(&["get", "pods", "-n", "traefik", "--no-headers"])
            .output();
        
        if let Ok(output) = traefik_check {
            if output.status.success() {
                let pods = String::from_utf8_lossy(&output.stdout);
                if pods.contains("Running") {
                    print_status("✅ Traefik is ready", "green");
                    break;
                }
            }
        }
        attempts += 1;
        if attempts % 12 == 0 { // Show progress every minute
            print_status(&format!("⏳ Still waiting for Traefik... ({}s)", attempts * 5), "yellow");
        }
    }
    
    // Deploy ingress resources after Traefik is ready
    print_status("🔗 Deploying ingress configuration...", "cyan");
    run_command("kubectl apply -f argocd-apps/observability-ingress.yaml", "Applying ingress configuration")?;
    
    print_status("✅ Observability stack deployment complete", "green");
    Ok(())
}

fn deploy_sample_apps(namespace: &str) -> Result<()> {
    print_status("🚀 Deploying sample applications...", "yellow");
    
    // Create namespace first if it doesn't exist
    let namespace_cmd = format!("kubectl create namespace {} --dry-run=client -o yaml | kubectl apply -f -", namespace);
    run_command(&namespace_cmd, "Creating observability namespace")?;
    
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
        print_status(&format!("📋 Executing: {}", cmd), "cyan");
        
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
        print_status(&format!("📋 Executing: {}", cmd), "cyan");
        
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

fn setup_ingress(namespace: &str) -> Result<()> {
    print_status("🔗 Setting up Traefik Ingress Controller...", "cyan");
    println!();
    
    // Check if Traefik is running
    print_status("🔍 Checking Traefik status...", "yellow");
    let traefik_check = Command::new("kubectl")
        .args(&["get", "pods", "-n", "traefik", "--no-headers"])
        .output();
    
    match traefik_check {
        Ok(output) => {
            if output.status.success() {
                let pods = String::from_utf8_lossy(&output.stdout);
                if pods.contains("Running") {
                    print_status("✅ Traefik is running", "green");
                } else {
                    print_status("⚠️ Traefik pods are not ready", "yellow");
                    print_status("📋 Waiting for Traefik to be ready...", "cyan");
                    
                    // Wait for Traefik to be ready
                    let mut attempts = 0;
                    while attempts < 30 {
                        thread::sleep(Duration::from_secs(5));
                        let status_check = Command::new("kubectl")
                            .args(&["get", "pods", "-n", "traefik", "--no-headers"])
                            .output();
                        
                        if let Ok(status_output) = status_check {
                            let status_pods = String::from_utf8_lossy(&status_output.stdout);
                            if status_pods.contains("Running") {
                                print_status("✅ Traefik is now ready", "green");
                                break;
                            }
                        }
                        attempts += 1;
                        if attempts % 6 == 0 {
                            print_status(&format!("⏳ Still waiting for Traefik... ({}s)", attempts * 5), "yellow");
                        }
                    }
                }
            } else {
                print_status("❌ Traefik namespace not found", "red");
                print_status("📋 Deploying Traefik first...", "cyan");
                deploy_stack(namespace)?;
            }
        }
        Err(_) => {
            print_status("❌ Cannot check Traefik status", "red");
            print_status("📋 Deploying Traefik first...", "cyan");
            deploy_stack(namespace)?;
        }
    }
    
    // Check if ingress is configured
    print_status("🔍 Checking ingress configuration...", "yellow");
    let ingress_check = Command::new("kubectl")
        .args(&["get", "ingress", "-n", namespace])
        .output();
    
    match ingress_check {
        Ok(output) => {
            if output.status.success() {
                print_status("✅ Ingress resources found", "green");
            } else {
                print_status("⚠️ Ingress resources not found", "yellow");
                print_status("📋 Applying ingress configuration...", "cyan");
                run_command("kubectl apply -f argocd-apps/observability-ingress.yaml", "Applying ingress configuration")?;
            }
        }
        Err(_) => {
            print_status("❌ Cannot check ingress status", "red");
            print_status("📋 Applying ingress configuration...", "cyan");
            run_command("kubectl apply -f argocd-apps/observability-ingress.yaml", "Applying ingress configuration")?;
        }
    }
    
    // Setup hosts file entries
    print_status("📝 Setting up local hosts file entries...", "cyan");
    setup_hosts_file()?;
    
    // Display access information
    print_status("🌐 Ingress Access Information", "cyan");
    println!("{}", "=".repeat(40));
    println!();
    println!("🚀 Traefik Dashboard:");
    println!("   URL: http://localhost:30080/traefik");
    println!("   Username: admin");
    println!("   Password: admin");
    println!();
    println!("📊 Grafana Dashboard:");
    println!("   URL: http://localhost:30080/grafana");
    println!("   Username: admin");
    println!("   Password: admin123");
    println!();
    println!("📈 Prometheus Metrics:");
    println!("   URL: http://localhost:30080/prometheus");
    println!("   No authentication required");
    println!();
    println!("🔍 Jaeger Tracing:");
    println!("   URL: http://localhost:30080/jaeger");
    println!("   No authentication required");
    println!();
    println!("🗄️ ClickHouse Database:");
    println!("   URL: http://localhost:30080/clickhouse");
    println!("   Username: default");
    println!("   Password: clickhouse123");
    println!();
    println!("🎯 ArgoCD UI:");
    println!("   URL: http://localhost:30080/argocd");
    println!("   Username: admin");
    println!("   Password: admin");
    println!();
    println!("{}", "=".repeat(40));
    println!();
    println!("📋 Note: All services are accessible via path-based routing on localhost:30080");
    println!("📋 No port-forwarding required - everything works through Traefik!");
    println!();
    println!("🔧 To check ingress status:");
    println!("   kubectl get ingress -n {}", namespace);
    println!("   kubectl get pods -n traefik");
    println!();
    
    Ok(())
}

fn setup_hosts_file() -> Result<()> {
    print_status("📝 Adding hosts file entries...", "cyan");
    
    let hosts_entries = vec![
        "127.0.0.1 localhost",
    ];
    
    // Check if entries already exist
    let hosts_path = r"C:\Windows\System32\drivers\etc\hosts";
    let hosts_content = std::fs::read_to_string(hosts_path)
        .context("Failed to read hosts file")?;
    
    let mut needs_update = false;
    for entry in &hosts_entries {
        if !hosts_content.contains(entry) {
            needs_update = true;
            break;
        }
    }
    
    if needs_update {
        print_status("📝 Adding new hosts entries...", "yellow");
        
        // Create backup
        let backup_path = format!("{}.backup.{}", hosts_path, chrono::Utc::now().timestamp());
        std::fs::copy(hosts_path, &backup_path)
            .context("Failed to create hosts file backup")?;
        print_status(&format!("✅ Backup created: {}", backup_path), "green");
        
        // Add entries
        let mut new_content = hosts_content.clone();
        new_content.push_str("\n# Kubernetes Observability Stack - Added by k8s-obs\n");
        for entry in &hosts_entries {
            new_content.push_str(&format!("{}\n", entry));
        }
        
        // Write with elevated privileges (this might fail on Windows)
        match std::fs::write(hosts_path, new_content) {
            Ok(_) => {
                print_status("✅ Hosts file updated successfully", "green");
            }
            Err(_e) => {
                print_status("⚠️ Could not update hosts file automatically", "yellow");
                print_status("📋 Please add these entries manually to C:\\Windows\\System32\\drivers\\etc\\hosts:", "cyan");
                println!();
                for entry in hosts_entries {
                    println!("   {}", entry);
                }
                println!();
                print_status("📋 Or run PowerShell as Administrator and execute:", "cyan");
                println!("   Add-Content -Path 'C:\\Windows\\System32\\drivers\\etc\\hosts' -Value '127.0.0.1 localhost'");
            }
        }
    } else {
        print_status("✅ Hosts file entries already exist", "green");
    }
    
    Ok(())
}

fn disable_docker_nginx() -> Result<()> {
    print_status("🔧 Disabling Docker Desktop NGINX Ingress Controller...", "yellow");
    println!();
    
    print_status("🔍 Checking for Docker Desktop NGINX ingress controller...", "cyan");
    
    // Check if we're using Docker Desktop context
    let context_check = Command::new("kubectl")
        .args(&["config", "current-context"])
        .output();
    
    let is_docker_desktop = match context_check {
        Ok(output) => {
            let context = String::from_utf8_lossy(&output.stdout);
            let context_trimmed = context.trim();
            context_trimmed.contains("docker-desktop") || context_trimmed.contains("docker-for-desktop")
        }
        Err(_) => false,
    };
    
    if !is_docker_desktop {
        print_status("ℹ️  Not using Docker Desktop context - this command is for Docker Desktop users", "yellow");
        print_status("📋 If you're using Kind cluster, NGINX conflicts are unlikely", "cyan");
        return Ok(());
    }
    
    print_status("📋 Docker Desktop context detected", "cyan");
    println!();
    
    // Check for NGINX ingress controller resources
    let nginx_resources = vec![
        ("kubectl get namespace ingress-nginx", "ingress-nginx namespace"),
        ("kubectl get deployment -n ingress-nginx", "NGINX deployment"),
        ("kubectl get service -n ingress-nginx", "NGINX services"),
        ("kubectl get ingressclass nginx", "NGINX ingress class"),
    ];
    
    let mut found_nginx = false;
    
    for (cmd, description) in nginx_resources {
        print_status(&format!("🔍 Checking for {}...", description), "cyan");
        
        let output = Command::new("cmd")
            .args(&["/C", cmd])
            .output();
        
        match output {
            Ok(output) => {
                if output.status.success() {
                    let result = String::from_utf8_lossy(&output.stdout);
                    if !result.contains("No resources found") && !result.is_empty() {
                        print_status(&format!("⚠️  Found {}: {}", description, result.lines().next().unwrap_or("")), "yellow");
                        found_nginx = true;
                    } else {
                        print_status(&format!("✅ No {} found", description), "green");
                    }
                } else {
                    print_status(&format!("✅ No {} found (not installed)", description), "green");
                }
            }
            Err(_) => {
                print_status(&format!("✅ No {} found (not accessible)", description), "green");
            }
        }
    }
    
    if !found_nginx {
        print_status("✅ No NGINX ingress controller found - no action needed", "green");
        return Ok(());
    }
    
    println!();
    print_status("⚠️  NGINX ingress controller found! This may conflict with Traefik.", "yellow");
    println!();
    print_status("🔧 Options to resolve:", "cyan");
    println!("   1. Disable NGINX in Docker Desktop settings");
    println!("   2. Remove NGINX resources manually");
    println!("   3. Use Kind cluster instead (recommended)");
    println!();
    
    // Option 1: Docker Desktop Settings
    print_status("📋 Option 1: Disable in Docker Desktop Settings", "cyan");
    println!("   1. Open Docker Desktop");
    println!("   2. Go to Settings → Kubernetes");
    println!("   3. Uncheck 'Enable Kubernetes'");
    println!("   4. Click 'Apply & Restart'");
    println!("   5. Re-enable Kubernetes (this will start fresh)");
    println!();
    
    // Option 2: Manual removal
    print_status("📋 Option 2: Manual Removal (Advanced)", "cyan");
    println!("   Run these commands to remove NGINX resources:");
    println!("   kubectl delete namespace ingress-nginx --ignore-not-found=true");
    println!("   kubectl delete ingressclass nginx --ignore-not-found=true");
    println!("   kubectl delete clusterrolebinding nginx-ingress --ignore-not-found=true");
    println!("   kubectl delete clusterrole nginx-ingress --ignore-not-found=true");
    println!();
    
    // Option 3: Use Kind cluster
    print_status("📋 Option 3: Use Kind Cluster (Recommended)", "cyan");
    println!("   Kind cluster provides a clean environment without Docker Desktop conflicts:");
    println!("   k8s-obs setup-cluster");
    println!();
    
    print_status("🎯 Recommended Action:", "green");
    println!("   Use 'k8s-obs setup-cluster' to create a Kind cluster");
    println!("   This avoids all Docker Desktop conflicts and provides a clean environment");
    println!();
    
    // Check if user wants to proceed with manual removal
    print_status("❓ Do you want to attempt manual removal of NGINX resources? (y/N)", "yellow");
    println!("   This will remove the ingress-nginx namespace and related resources.");
    println!("   Type 'y' to proceed, or any other key to skip:");
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    if input.trim().to_lowercase() == "y" {
        print_status("🗑️  Removing NGINX ingress controller resources...", "yellow");
        
        let removal_commands = vec![
            "kubectl delete namespace ingress-nginx --ignore-not-found=true",
            "kubectl delete ingressclass nginx --ignore-not-found=true",
            "kubectl delete clusterrolebinding nginx-ingress --ignore-not-found=true",
            "kubectl delete clusterrole nginx-ingress --ignore-not-found=true",
            "kubectl delete validatingwebhookconfiguration nginx-ingress-admission --ignore-not-found=true",
        ];
        
        for cmd in removal_commands {
            run_command(cmd, &format!("Removing: {}", cmd))?;
        }
        
        print_status("✅ NGINX ingress controller resources removed", "green");
        println!();
        print_status("📋 Next steps:", "cyan");
        println!("   1. Restart Docker Desktop");
        println!("   2. Run 'k8s-obs deploy-stack' to deploy Traefik");
        println!("   3. Run 'k8s-obs setup-ingress' to configure access");
    } else {
        print_status("⏭️  Skipping manual removal", "yellow");
        println!();
        print_status("📋 To resolve conflicts:", "cyan");
        println!("   - Use Kind cluster: k8s-obs setup-cluster");
        println!("   - Or disable Kubernetes in Docker Desktop settings");
    }
    
    Ok(())
}

fn get_urls(namespace: &str) -> Result<()> {
    print_status("🌐 Service URLs & Access Information", "cyan");
    println!("{}", "=".repeat(40));
    println!();
    println!("📋 Note: Services are accessible via Traefik Ingress Controller");
    println!("📋 Use 'k8s-obs setup-ingress' to configure ingress access");
    println!();
    
    let services = vec![
        ("traefik", "traefik", "Traefik Dashboard", "/traefik", "admin/admin"),
        ("argocd-server", "argocd", "ArgoCD UI", "/argocd", "admin/admin"),
        ("grafana", namespace, "Grafana", "/grafana", "admin/admin123"),
        ("prometheus-server", namespace, "Prometheus", "/prometheus", "No authentication"),
        ("clickhouse", namespace, "ClickHouse", "/clickhouse", "default/clickhouse123"),
        ("jaeger-query", namespace, "Jaeger UI", "/jaeger", "No authentication"),
    ];
    
    for (_service, _ns, name, path, credentials) in services {
        println!("{}:", name);
        println!("  🌐 URL: http://localhost:30080{}", path);
        println!("  🔐 Credentials: {}", credentials);
        println!();
    }
    
    println!("{}", "=".repeat(40));
    println!("🚀 To setup ingress access, run: k8s-obs setup-ingress");
    println!("📊 To check service status, run: k8s-obs status");
    println!("🔧 To check ingress status, run: kubectl get ingress -n {}", namespace);
    
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
    println!("  setup-ingress    - Set up Traefik ingress for local access");
    println!("  disable-docker-nginx - Disable Docker Desktop NGINX ingress controller");
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
    println!("Access URLs (after ingress setup):");
println!("  Traefik Dashboard: http://localhost:30080/traefik (admin/admin)");
println!("  ArgoCD UI: http://localhost:30080/argocd (admin/admin)");
println!("  Grafana: http://localhost:30080/grafana (admin/admin123)");
println!("  Prometheus: http://localhost:30080/prometheus");
println!("  Jaeger UI: http://localhost:30080/jaeger");
println!("  ClickHouse: http://localhost:30080/clickhouse");
    println!();
    println!("Usage: k8s-obs <command> [options]");
    println!("Example: k8s-obs deploy-stack");
} 