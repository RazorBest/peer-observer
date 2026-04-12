#!/bin/sh

set -x

# Envinronment variables that can be overwritten by the calling parent
END="${END:-$(date +%s)}"
START="${START:-$(( ${END} - 345600 ))}" # 4 days ago
PROMETHEUS_URL="${PROMETHEUS_URL:-http://127.0.0.1:9090}"
RULE_FILES="${RULE_FILES:-/etc/prometheus/rules/monotonic.yml /etc/prometheus/rules/anomalies.yml}"

promtool tsdb create-blocks-from rules --start "${START}" --end "${END}" --url "${PROMETHEUS_URL}" ${RULE_FILES}

set +x
