use std::env;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use clap::Parser;
use colored::*;

#[derive(Parser)]
#[command(name = "deploy_sample_apps")]
#[command(about = "Deploy sample applications to Kind cluster")]
struct Args {
    #[arg(long, default_value = "observability")]
    namespace: String,
}

struct SampleAppDeployer {
    namespace: String,
}

impl SampleAppDeployer {
    fn new(namespace: String) -> Self {
        Self { namespace }
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

    fn create_namespace(&self) -> Result<()> {
        self.print_status(&format!("📦 Creating namespace: {}", self.namespace), "yellow");
        let _ = self.run_command(&format!("kubectl create namespace {} --dry-run=client -o yaml | kubectl apply -f -", self.namespace), false);
        self.print_status("✅ Namespace created", "green");
        Ok(())
    }

    fn deploy_sample_app(&self) -> Result<()> {
        self.print_status("🚀 Deploying sample application...", "yellow");
        
        let sample_app_yaml = r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sample-app
  namespace: observability
spec:
  replicas: 3
  selector:
    matchLabels:
      app: sample-app
  template:
    metadata:
      labels:
        app: sample-app
    spec:
      containers:
      - name: sample-app
        image: nginx:alpine
        ports:
        - containerPort: 80
        env:
        - name: OTEL_SERVICE_NAME
          value: "sample-app"
        - name: OTEL_TRACES_EXPORTER
          value: "otlp"
        - name: OTEL_METRICS_EXPORTER
          value: "otlp"
        - name: OTEL_LOGS_EXPORTER
          value: "otlp"
        - name: OTEL_EXPORTER_OTLP_ENDPOINT
          value: "http://opentelemetry-collector.observability.svc.cluster.local:4318"
        - name: OTEL_RESOURCE_ATTRIBUTES
          value: "service.name=sample-app,service.version=1.0.0"
        resources:
          requests:
            memory: "64Mi"
            cpu: "50m"
          limits:
            memory: "128Mi"
            cpu: "100m"
        livenessProbe:
          httpGet:
            path: /
            port: 80
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /
            port: 80
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: sample-app-service
  namespace: observability
spec:
  selector:
    app: sample-app
  ports:
  - port: 80
    targetPort: 80
    nodePort: 30002
  type: NodePort
---
apiVersion: v1
kind: ServiceMonitor
metadata:
  name: sample-app-monitor
  namespace: observability
  labels:
    app: sample-app
spec:
  selector:
    matchLabels:
      app: sample-app
  endpoints:
  - port: http
    interval: 30s
    path: /metrics
"#;

        std::fs::write("./sample-app.yaml", sample_app_yaml)
            .context("Failed to write sample app YAML")?;
        
        self.run_command("kubectl apply -f ./sample-app.yaml", true)?;
        self.print_status("✅ Sample application deployed", "green");
        
        // Clean up temporary file
        std::fs::remove_file("./sample-app.yaml")
            .context("Failed to remove sample app YAML")?;
        
        Ok(())
    }

    fn deploy_load_generator(&self) -> Result<()> {
        self.print_status("🔄 Deploying load generator...", "yellow");
        
        let load_generator_yaml = r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: load-generator
  namespace: observability
spec:
  replicas: 1
  selector:
    matchLabels:
      app: load-generator
  template:
    metadata:
      labels:
        app: load-generator
    spec:
      containers:
      - name: load-generator
        image: busybox:latest
        command: ["/bin/sh"]
        args:
        - -c
        - |
          while true; do
            wget -q -O- http://sample-app-service:80 || echo "Failed to connect"
            sleep 5
          done
        resources:
          requests:
            memory: "32Mi"
            cpu: "25m"
          limits:
            memory: "64Mi"
            cpu: "50m"
"#;

        std::fs::write("./load-generator.yaml", load_generator_yaml)
            .context("Failed to write load generator YAML")?;
        
        self.run_command("kubectl apply -f ./load-generator.yaml", true)?;
        self.print_status("✅ Load generator deployed", "green");
        
        // Clean up temporary file
        std::fs::remove_file("./load-generator.yaml")
            .context("Failed to remove load generator YAML")?;
        
        Ok(())
    }

    fn verify_deployment(&self) -> Result<()> {
        self.print_status("🔍 Verifying deployment...", "yellow");
        
        self.run_command("kubectl get pods -n observability", true)?;
        self.run_command("kubectl get services -n observability", true)?;
        
        self.print_status("✅ Deployment verified", "green");
        Ok(())
    }

    fn deploy(&self) -> Result<()> {
        self.print_status("🚀 Deploying Sample Applications", "green");
        
        // Set KUBECONFIG
        if std::path::Path::new("./kubeconfig").exists() {
            env::set_var("KUBECONFIG", "./kubeconfig");
        } else {
            self.print_status("❌ Kubeconfig not found. Please run setup_kind_cluster first.", "red");
            return Ok(());
        }
        
        self.create_namespace()?;
        self.deploy_sample_app()?;
        self.deploy_load_generator()?;
        self.verify_deployment()?;
        
        self.print_status("\n🎉 Sample applications deployed successfully!", "green");
        self.print_status("📋 Application Information:", "cyan");
        self.print_status("   Sample App: http://localhost:30002", "white");
        self.print_status("   Load Generator: Running in background", "white");
        self.print_status("", "white");
        self.print_status("🔍 Monitor with:", "cyan");
        self.print_status("   kubectl logs -f deployment/sample-app -n observability", "white");
        self.print_status("   kubectl logs -f deployment/load-generator -n observability", "white");
        
        Ok(())
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    let deployer = SampleAppDeployer::new(args.namespace);
    deployer.deploy()?;
    
    Ok(())
} 