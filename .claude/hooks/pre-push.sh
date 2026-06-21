#!/usr/bin/env bash
# Dry-run all manifests before git push. Force pushes are blocked by the deny list.
cmd=$(python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('tool_input',{}).get('command',''))" 2>/dev/null)
echo "$cmd" | grep -qE '^git push' || exit 0
echo "$cmd" | grep -qE '\-\-force|-f' && exit 0

echo "--- pre-push: manifest dry-run ---"
kubectl apply -k argocd-apps/ --dry-run=client 2>&1 \
  && echo "Manifests OK" \
  || { echo "MANIFEST ERRORS — fix before pushing"; exit 1; }
