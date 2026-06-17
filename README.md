# Kubernetes Observability Stack

A Kubernetes observability POC using **OpenTelemetry** as the unified collection layer, deployed via GitOps (ArgoCD), and exposed through a single Traefik ingress on path-based routing.

## Architecture

### Three pillars

| Pillar | Tool | Role |
|---|---|---|
| **Metrics** | Prometheus + Grafana | Collect, store, and visualize metrics |
| **Traces** | Jaeger | Distributed trace storage and query UI |
| **Logs** | ClickHouse | High-performance log storage and querying |

### Data flow

```
flowchart LR
    User["Browser"]
    Ingress["Traefik :30080
    ────────────────
    /grafana
    /prometheus
    /jaeger
    /clickhouse"]

    subgraph Collectors["OTel Collectors"]
        CD["Deployment (metrics + traces)"]
        DS["DaemonSet (logs per node)"]
    end

    Prometheus["Prometheus"]
    CH["ClickHouse"]
    Jaeger["Jaeger"]
    Grafana["Grafana"]

    User --> Ingress
    Ingress --> Grafana
    Ingress --> Prometheus
    Ingress --> Jaeger
    App["App (OTLP)"] --> CD
    CD --> Prometheus
    CD --> Jaeger
    DS --> CH
    Prometheus --> Grafana
    CH --> Grafana
    Jaeger --> Grafana
```

### Components

| Component | Namespace | Purpose |
|---|---|---|
| **Traefik** | traefik | Path-based ingress on NodePort 30080 |
| **OTel Operator** | opentelemetry-operator-system | Manages Collector CRDs and admission webhooks |
| **OTel Collector (Deployment)** | observability | Receives OTLP; fans out metrics → Prometheus, traces → Jaeger |
| **OTel Collector (DaemonSet)** | observability | Tails `/var/log/pods` per node; ships to ClickHouse |
| **Prometheus** | observability | Scrapes and stores metrics |
| **ClickHouse** | observability | Stores and indexes log data |
| **Jaeger** | observability | Stores distributed traces; trace search and dependency graphs |
| **Grafana** | observability | Unified dashboards — correlates traces ↔ logs ↔ metrics |
| **ArgoCD** | argocd | GitOps reconciler for all stack components |

### Ingress URL map

| Path | Service | Port |
|---|---|---|
| `/grafana` | Grafana | 80 |
| `/prometheus` | Prometheus | 9090 |
| `/jaeger` | Jaeger Query UI | 16686 |
| `/clickhouse` | ClickHouse HTTP | 8123 |
| `/traefik` | Traefik Dashboard | 9000 |
| `/argocd` | ArgoCD Server | 80 |

---

## Quick Start

### Prerequisites

- **Podman** (rootless) with the user socket running — Docker is not used
- `kubectl`
- `helm` v3+
- `kind`

```bash
# Verify Podman socket is active
systemctl --user enable --now podman.socket

# Load env vars (or use direnv to auto-load)
source .envrc
```

### 1. Create the Kind cluster (via Podman)

```bash
kind create cluster --name observability-cluster --config kind-config.yaml
# Creates: 1 control-plane + 3 workers
```

### 2. Deploy the stack

```bash
# Install ArgoCD
kubectl create namespace argocd
kubectl apply -n argocd -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml
kubectl wait --for=condition=Available deployment/argocd-server -n argocd --timeout=120s

# Deploy all observability apps via ArgoCD
kubectl apply -k argocd-apps/
```

### 3. Access the UIs

| UI | URL | Credentials |
|---|---|---|
| Grafana | http://localhost:30080/grafana | admin / admin123 |
| Prometheus | http://localhost:30080/prometheus | — |
| Jaeger | http://localhost:30080/jaeger | — |
| ClickHouse | http://localhost:30080/clickhouse | default / clickhouse123 |
| Traefik | http://localhost:30080/traefik | admin / admin |
| ArgoCD | http://localhost:30080/argocd | admin / admin |

---

## Setup Guide (Manual Steps)

### Step 1 — Install cert-manager

Required by the OTel Operator for admission webhook TLS:

```bash
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.20.2/cert-manager.yaml
kubectl wait --for=condition=Ready pods --all -n cert-manager --timeout=120s
```

### Step 2 — Install the OTel Operator

```bash
helm repo add open-telemetry https://open-telemetry.github.io/opentelemetry-helm-charts
helm repo update

helm install opentelemetry-operator open-telemetry/opentelemetry-operator \
  --namespace opentelemetry-operator-system \
  --create-namespace \
  --set "manager.collectorImage.repository=otel/opentelemetry-collector-contrib"
```

### Step 3 — Install Traefik

```bash
helm repo add traefik https://traefik.github.io/charts
helm repo update

helm install traefik traefik/traefik \
  --namespace traefik \
  --create-namespace \
  --set ports.web.exposedPort=80 \
  --set ports.websecure.exposedPort=443
```

### Step 4 — Install observability backends

```bash
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo update

# Prometheus + Grafana
helm install kube-prometheus-stack prometheus-community/kube-prometheus-stack \
  --namespace monitoring \
  --create-namespace \
  --set prometheus.prometheusSpec.serviceMonitorSelectorNilUsesHelmValues=false \
  --set "grafana.grafana\.ini.server.serve_from_sub_path=true" \
  --set "grafana.grafana\.ini.server.root_url=http://localhost:30080/grafana"

# ClickHouse (log store)
helm install clickhouse bitnami/clickhouse \
  --namespace monitoring \
  --set auth.password=clickhouse123

# Jaeger Operator
kubectl create namespace observability
kubectl apply -n observability \
  -f https://github.com/jaegertracing/jaeger-operator/releases/download/v1.65.0/jaeger-operator.yaml
```

### Step 5 — Deploy Jaeger instance

```yaml
# jaeger.yaml
apiVersion: jaegertracing.io/v1
kind: Jaeger
metadata:
  name: jaeger
  namespace: monitoring
spec:
  strategy: allInOne
  allInOne:
    options:
      query:
        base-path: /jaeger
  storage:
    type: memory
    options:
      memory:
        max-traces: 100000
```

```bash
kubectl apply -f jaeger.yaml
```

### Step 6 — Create the IngressRoute

```yaml
# observability-ingressroute.yaml
apiVersion: traefik.io/v1alpha1
kind: IngressRoute
metadata:
  name: observability-ingressroute
  namespace: monitoring
spec:
  entryPoints:
    - web
  routes:
    - match: PathPrefix(`/grafana`)
      kind: Rule
      services:
        - name: kube-prometheus-stack-grafana
          port: 80
    - match: PathPrefix(`/prometheus`)
      kind: Rule
      services:
        - name: kube-prometheus-stack-prometheus
          port: 9090
    - match: PathPrefix(`/jaeger`)
      kind: Rule
      services:
        - name: jaeger-query
          port: 16686
```

### Step 7 — Deploy the Metrics + Traces Collector

```yaml
# otel-collector.yaml
apiVersion: opentelemetry.io/v1alpha1
kind: OpenTelemetryCollector
metadata:
  name: otel-collector
  namespace: monitoring
spec:
  mode: deployment
  config: |
    receivers:
      otlp:
        protocols:
          grpc:
            endpoint: 0.0.0.0:4317
          http:
            endpoint: 0.0.0.0:4318

    processors:
      batch:
        timeout: 10s
      memory_limiter:
        limit_mib: 400
        spike_limit_mib: 100
        check_interval: 5s

    exporters:
      prometheus:
        endpoint: "0.0.0.0:8889"
      otlp/jaeger:
        endpoint: jaeger-collector.monitoring.svc.cluster.local:4317
        tls:
          insecure: true
      debug:
        verbosity: basic

    service:
      pipelines:
        metrics:
          receivers: [otlp]
          processors: [memory_limiter, batch]
          exporters: [prometheus, debug]
        traces:
          receivers: [otlp]
          processors: [memory_limiter, batch]
          exporters: [otlp/jaeger, debug]
```

### Step 8 — Deploy the Log Collector (DaemonSet)

Apply RBAC first:

```yaml
# otel-log-collector-rbac.yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: otel-log-collector
  namespace: monitoring
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: otel-log-collector
rules:
  - apiGroups: [""]
    resources: ["pods", "namespaces", "nodes"]
    verbs: ["get", "list", "watch"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: otel-log-collector
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: otel-log-collector
subjects:
  - kind: ServiceAccount
    name: otel-log-collector
    namespace: monitoring
```

Then the DaemonSet collector (ships logs to ClickHouse):

```yaml
# otel-log-collector.yaml
apiVersion: opentelemetry.io/v1alpha1
kind: OpenTelemetryCollector
metadata:
  name: otel-log-collector
  namespace: monitoring
spec:
  mode: daemonset
  serviceAccount: otel-log-collector
  volumeMounts:
    - name: varlogpods
      mountPath: /var/log/pods
      readOnly: true
  volumes:
    - name: varlogpods
      hostPath:
        path: /var/log/pods
  config: |
    receivers:
      filelog:
        include:
          - /var/log/pods/*/*/*.log
        start_at: beginning
        include_file_path: true
        include_file_name: false
        operators:
          - type: container
            id: container-parser

    processors:
      batch:
        timeout: 5s
      k8sattributes:
        auth_type: serviceAccount
        passthrough: false
        extract:
          metadata:
            - k8s.namespace.name
            - k8s.pod.name
            - k8s.pod.uid
            - k8s.container.name
            - k8s.node.name
        pod_association:
          - sources:
              - from: resource_attribute
                name: k8s.pod.name
              - from: resource_attribute
                name: k8s.namespace.name

    exporters:
      clickhouse:
        endpoint: tcp://clickhouse.monitoring.svc.cluster.local:9000
        database: default
        username: default
        password: clickhouse123
        logs_table_name: otel_logs
        ttl_days: 3

    service:
      pipelines:
        logs:
          receivers: [filelog]
          processors: [k8sattributes, batch]
          exporters: [clickhouse]
```

### Step 9 — Instrument Your Application

**Auto-instrumentation** (zero code change, requires OTel Operator):

```yaml
# instrumentation.yaml
apiVersion: opentelemetry.io/v1alpha1
kind: Instrumentation
metadata:
  name: my-instrumentation
  namespace: default
spec:
  exporter:
    endpoint: http://otel-collector-collector.monitoring.svc.cluster.local:4318
  propagators:
    - tracecontext
    - baggage
  sampler:
    type: parentbased_traceidratio
    argument: "1"
  python:
    image: ghcr.io/open-telemetry/opentelemetry-operator/autoinstrumentation-python:latest
  java:
    image: ghcr.io/open-telemetry/opentelemetry-operator/autoinstrumentation-java:latest
  nodejs:
    image: ghcr.io/open-telemetry/opentelemetry-operator/autoinstrumentation-nodejs:latest
```

Then annotate your Deployment:

```yaml
annotations:
  instrumentation.opentelemetry.io/inject-python: "true"
```

**Manual SDK** — set via environment variables:

```yaml
env:
  - name: OTEL_EXPORTER_OTLP_ENDPOINT
    value: "http://otel-collector-collector.monitoring.svc.cluster.local:4317"
  - name: OTEL_SERVICE_NAME
    value: "my-service"
```

---

## Grafana Setup

### Default credentials

```bash
# Grafana admin password
kubectl get secret kube-prometheus-stack-grafana -n monitoring \
  -o jsonpath="{.data.admin-password}" | base64 --decode
```

### Add ClickHouse data source

1. **Connections → Plugins** → search "ClickHouse" → install
2. **Connections → Data Sources → Add → ClickHouse**
3. URL: `http://clickhouse.monitoring.svc.cluster.local:8123`
4. Database: `default`, Username: `default`, Password: `clickhouse123`

### Add Jaeger data source

1. **Connections → Data Sources → Add → Jaeger**
2. URL: `http://jaeger-query.monitoring.svc.cluster.local:16686`

### Import dashboards

| Dashboard | ID | Data source |
|---|---|---|
| OTel Collector | 15983 | Prometheus |
| Kubernetes Cluster | 7249 | Prometheus |
| Node Exporter | 1860 | Prometheus |

---

## Troubleshooting

| Symptom | Likely Cause | Fix |
|---|---|---|
| `/grafana` returns 404 | IngressRoute not picked up | `kubectl describe ingressroute -n monitoring`; check Traefik logs |
| Grafana CSS/JS broken | `serve_from_sub_path` not set | Helm upgrade with `serve_from_sub_path=true` and correct `root_url` |
| Jaeger UI redirects to `/` | `base-path` not set | Check `spec.allInOne.options.query.base-path: /jaeger` |
| Collector `CrashLoopBackOff` | Bad config YAML | `kubectl logs <pod> -n monitoring` |
| Prometheus target DOWN | ServiceMonitor label mismatch | Check `release:` label matches Helm release name |
| No logs in ClickHouse | DaemonSet can't reach CH | Check collector logs; verify ClickHouse pod running |
| Logs missing K8s labels | k8sattributes RBAC missing | Verify `ClusterRoleBinding` is applied |
| No traces in Jaeger | Collector can't reach Jaeger | Verify `jaeger-collector` service; check `insecure: true` |
| Auto-instrumentation not injecting | Annotation typo | `kubectl describe pod` — check mutating webhook events |
| cert-manager webhook timeout | cert-manager not ready | `kubectl get pods -n cert-manager` and wait |

### Useful commands

```bash
kubectl get pods --all-namespaces
kubectl get applications -n argocd
kubectl logs -n observability deployment/opentelemetry-collector
kubectl get svc -n monitoring
kubectl get secret kube-prometheus-stack-grafana -n monitoring \
  -o jsonpath="{.data.admin-password}" | base64 --decode
```

---

## Resource Requirements

### Minimum

- CPU: 4 cores
- Memory: 8 GB RAM
- Storage: 20 GB

### Per component

| Component | Memory | CPU |
|---|---|---|
| Grafana | 512 MB | 500m |
| Prometheus | 2 GB | 1 core |
| Jaeger | 512 MB | 500m |
| ClickHouse | 1 GB | 1 core |
| OTel Collector | 512 MB | 500m |

---

## Production Considerations

This stack is a **development/POC** setup. For production:

- **Jaeger storage**: Switch from in-memory to production strategy backed by ClickHouse or Cassandra
- **ClickHouse**: Deploy as a distributed cluster; enable replication
- **Prometheus**: Add persistent storage (PVC with `storageSpec`)
- **Security**: Enable TLS, RBAC, and network policies throughout
- **High availability**: Multiple replicas for all components
- **Auto-instrumentation images**: Pin to specific versions (not `:latest`)
