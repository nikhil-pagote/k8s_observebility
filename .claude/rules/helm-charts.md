---
description: Only official, project-maintained Helm repos may be used in this project
applies-to: helm
---

# Rule: Official Helm Charts Only

Only Helm charts from official, project-maintained repositories are permitted.
Third-party repackagers (e.g. Bitnami, TrueCharts, Artifact Hub mirrors) must not be used.

Each repo must be owned and published by the upstream project itself.

## Approved Repos

| Repo alias | URL | Maintained by |
|---|---|---|
| `argo` | https://argoproj.github.io/argo-helm | Argo Project (CNCF) |
| `prometheus-community` | https://prometheus-community.github.io/helm-charts | Prometheus community (CNCF) |
| `grafana` | https://grafana.github.io/helm-charts | Grafana Labs |
| `jaegertracing` | https://jaegertracing.github.io/helm-charts | Jaeger project (CNCF) |
| `open-telemetry` | https://open-telemetry.github.io/opentelemetry-helm-charts | OpenTelemetry project (CNCF) |
| `traefik` | https://traefik.github.io/charts | Traefik Labs |

## Adding a New Chart

Before adding any chart not in the approved list:
1. Verify the repo is the canonical upstream source for that project.
2. Document the justification in the PR description.
3. Add it to the approved table above.
