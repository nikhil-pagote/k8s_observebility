---
description: Dry-run all YAML manifests in the repo to catch schema and syntax errors before applying to the cluster
argument-hint: "[--strict]"
allowed-tools:
  - Bash
  - Read
---

Run `kubectl apply --dry-run=client` on all manifests and report any errors.

## Steps

1. Validate ArgoCD kustomize overlay:
```bash
kubectl apply -k argocd-apps/ --dry-run=client
```

2. Validate app manifests:
```bash
kubectl apply -f apps/load-generator/ --dry-run=client --namespace=observability
kubectl apply -f apps/sample-app/deployment-basic.yaml --dry-run=client --namespace=observability
```

3. If `--strict` argument is passed, also run yamllint:
```bash
find . -name "*.yaml" -not -path "./.git/*" -not -path "./src-build/*" | xargs yamllint -d relaxed
```

4. Report: list each file validated, flag any errors. If all pass, print "All manifests valid."

Note: `--dry-run=client` validates locally without a running cluster.
