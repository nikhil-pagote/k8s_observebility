---
description: Verify all three OTel pillars (metrics → Prometheus, traces → Jaeger, logs → Loki) are flowing end-to-end
allowed-tools:
  - Bash
---

Port-forward each backend, query it, and report PASS/FAIL per pillar.

## Steps

### 1. OTel Collector health

```bash
kubectl get pods -n observability -l app.kubernetes.io/name=opentelemetry-collector
kubectl logs -n observability deployment/opentelemetry-collector --tail=30
```

Look for: receivers started, no ERROR lines.

### 2. Metrics → Prometheus

```bash
kubectl port-forward -n observability svc/prometheus-server 9090:80 &
PF1=$!
sleep 3

curl -s "http://localhost:9090/prometheus/api/v1/targets" | python3 -c "
import sys, json
d = json.load(sys.stdin)
targets = d.get('data', {}).get('activeTargets', [])
otel = [t for t in targets if 'otel' in t.get('labels', {}).get('job', '').lower()]
for t in otel:
    print(f\"OTel target {t['labels']['instance']}: {t['health']}\")
if not otel:
    print('WARNING: no OTel targets found in Prometheus')
"

curl -s "http://localhost:9090/prometheus/api/v1/query?query=otelcol_receiver_accepted_metric_points_total" | \
  python3 -c "import sys,json; d=json.load(sys.stdin); print('Metrics PASS' if d['data']['result'] else 'Metrics FAIL: no data')"

kill $PF1 2>/dev/null
```

### 3. Traces → Jaeger

```bash
kubectl port-forward -n observability svc/jaeger 16686:16686 &
PF2=$!
sleep 3

curl -s "http://localhost:16686/jaeger/api/services" | python3 -c "
import sys, json
d = json.load(sys.stdin)
services = d.get('data', [])
print('Traces PASS — services:', services) if services else print('Traces: no services yet (send traffic first)')
"

kill $PF2 2>/dev/null
```

### 4. Logs → Loki

```bash
kubectl port-forward -n observability svc/loki 3100:3100 &
PF3=$!
sleep 3

curl -s "http://localhost:3100/loki/api/v1/labels" | python3 -c "
import sys, json
d = json.load(sys.stdin)
labels = d.get('data', [])
print('Logs PASS — labels:', labels) if labels else print('Logs FAIL: no labels found (no logs ingested yet)')
"

kill $PF3 2>/dev/null
```

## Summary

Report one line per pillar: `PASS` or `FAIL — <reason>`.
