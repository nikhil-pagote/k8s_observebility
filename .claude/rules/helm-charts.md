---
description: Only official, project-maintained Helm repos may be used; never modify files inside argocd-apps/**/chart/
globs:
  - "argocd-apps/**/Chart.yaml"
  - "argocd-apps/**/values.yaml"
  - "argocd-apps/**/chart/**"
  - ".claude/rules/helm-charts.md"
---

# Rule: Official Helm Charts Only — Never Modify Vendored Chart Files

Only Helm charts from official, project-maintained repositories are permitted.
Third-party repackagers (e.g. Bitnami, TrueCharts, Artifact Hub mirrors) must not be used.

Each repo must be owned and published by the upstream project itself.

## Do NOT modify files inside `argocd-apps/**/chart/`

Files under `argocd-apps/<app>/chart/` are pulled verbatim from upstream with `helm pull --untar`.
**Never edit templates, helpers, or default values inside `chart/`.** Changes there are silently overwritten on the next chart upgrade.

To work around a chart limitation, use one of these instead:
- **Helm values** (`argocd-apps/<app>/values/values.yaml`) — the correct override mechanism.
- **A separate Kubernetes manifest** — e.g. add a `Sidecar`, `ConfigMap`, or `NetworkPolicy` as a standalone file alongside the chart.
- **ArgoCD `ignoreDifferences`** — suppress fields that upstream modifies at runtime (e.g. webhook caBundle).

If the chart truly does not expose a required value, document that constraint in `docs/troubleshooting_guide.md` and use a standalone manifest.

## Approved Repos

| Repo alias | URL | Maintained by |
|---|---|---|
| `argo` | https://argoproj.github.io/argo-helm | Argo Project (CNCF) |
| `prometheus-community` | https://prometheus-community.github.io/helm-charts | Prometheus community (CNCF) |
| `grafana` | https://grafana.github.io/helm-charts | Grafana Labs |
| `jaegertracing` | https://jaegertracing.github.io/helm-charts | Jaeger project (CNCF) |
| `open-telemetry` | https://open-telemetry.github.io/opentelemetry-helm-charts | OpenTelemetry project (CNCF) |
| `traefik` | https://traefik.github.io/charts | Traefik Labs |
| `victoriametrics` | https://victoriametrics.github.io/helm-charts/ | VictoriaMetrics |
| `istio` | https://istio-release.storage.googleapis.com/charts | Istio Project (CNCF) |
| `kiali` | https://kiali.org/helm-charts | Kiali Project (CNCF) |

## Adding a New Chart

Before adding any chart not in the approved list:
1. Verify the repo is the canonical upstream source for that project.
2. Document the justification in the PR description.
3. Add it to the approved table above.
