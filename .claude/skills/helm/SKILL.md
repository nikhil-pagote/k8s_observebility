---
description: Manage Helm repos and pull charts locally into each app's chart/ directory for ArgoCD to deploy
argument-hint: "<add-repos|pull|update>"
allowed-tools:
  - Bash
---

Pull Helm charts locally into `argocd-apps/<app>/chart/` so ArgoCD can deploy from the Git repo.

> Rule: only official project-maintained repos permitted — see `.claude/rules/helm-charts.md`.

## Add Repos

Register all official Helm repos used by this stack:

```bash
helm repo add argo https://argoproj.github.io/argo-helm
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo add grafana https://grafana.github.io/helm-charts
helm repo add jaegertracing https://jaegertracing.github.io/helm-charts
helm repo add open-telemetry https://open-telemetry.github.io/opentelemetry-helm-charts
helm repo add traefik https://traefik.github.io/charts
helm repo update
helm repo list
```

## Pull

Pull all charts at their pinned versions into each app's `chart/` directory.

```bash
helm pull traefik/traefik                        --version 41.0.0   --untar --untardir argocd-apps/traefik/
helm pull prometheus-community/prometheus         --version 29.12.0  --untar --untardir argocd-apps/prometheus/
helm pull grafana/grafana                        --version 10.5.15  --untar --untardir argocd-apps/grafana/
helm pull jaegertracing/jaeger                   --version 4.11.1   --untar --untardir argocd-apps/jaeger/
helm pull grafana/loki                           --version 6.55.0   --untar --untardir argocd-apps/loki/
helm pull open-telemetry/opentelemetry-collector --version 0.158.2  --untar --untardir argocd-apps/opentelemetry-collector/

echo "Charts pulled:"
for app in traefik prometheus grafana jaeger loki opentelemetry-collector; do
  echo "  $app/chart: $(ls argocd-apps/$app/chart/ 2>/dev/null | head -1 || echo 'empty')"
done
```

> `helm pull --untar` extracts into `<untardir>/<chart-name>/`. Always rename to `chart/` afterwards: `mv argocd-apps/<app>/<chart-name> argocd-apps/<app>/chart`.

## Update

Refresh repo index and re-pull all charts (run when upgrading versions):

```bash
helm repo update

helm pull traefik/traefik                        --version 41.0.0   --untar --untardir argocd-apps/traefik/
helm pull prometheus-community/prometheus         --version 29.12.0  --untar --untardir argocd-apps/prometheus/
helm pull grafana/grafana                        --version 10.5.15  --untar --untardir argocd-apps/grafana/
helm pull jaegertracing/jaeger                   --version 4.11.1   --untar --untardir argocd-apps/jaeger/
helm pull grafana/loki                           --version 6.55.0   --untar --untardir argocd-apps/loki/
helm pull open-telemetry/opentelemetry-collector --version 0.158.2  --untar --untardir argocd-apps/opentelemetry-collector/

echo "Charts updated."
```
