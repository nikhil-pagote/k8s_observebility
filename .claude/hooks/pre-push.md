---
description: Validate all Kubernetes manifests with kubectl dry-run before every git push
trigger: PreToolUse
matcher: Bash(git push:*)
allowed-tools:
  - Bash
---

When a `git push` is about to run, dry-run all manifests. Block if any are invalid.

```bash
echo "--- pre-push: manifest dry-run ---"
kubectl apply -k argocd-apps/ --dry-run=client 2>&1 \
  && echo "Manifests OK" \
  || { echo "MANIFEST ERRORS — fix before pushing"; exit 1; }
```

Force pushes (`git push --force`, `git push -f`) are blocked entirely by the deny list in `settings.json`.
