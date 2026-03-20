#!/usr/bin/env bash
set -euo pipefail

N="${1:-50}"
BIN="${BIN:-target/debug/zene}"

if [[ ! -x "$BIN" ]]; then
  echo "Building zene binary..." >&2
  cargo build >/dev/null
fi

json_check() {
  local line="$1"
  python3 - <<'PY' "$line"
import json
import sys
json.loads(sys.argv[1])
PY
}

run_one() {
  local idx="$1"
  local req
  req=$(printf '{"protocol_version":1,"type":"run","request_id":"soak_%s","session_id":"s_soak","prompt":"ping","timeout_ms":30000,"idempotency_key":"idem_%s"}' "$idx" "$idx")
  local out
  out=$(printf '%s\n' "$req" | "$BIN" host --protocol v1 --bridge-compat --single-request)
  json_check "$out"
}

echo "[soak] running ${N} one-shot requests" >&2
for ((i=1; i<=N; i++)); do
  run_one "$i"
done

echo "[soak] ${N}/${N} stdout lines are valid JSON" >&2

echo "[recovery] simulating interrupted request" >&2
tmp_in=$(mktemp)
tmp_out=$(mktemp)

"$BIN" host --protocol v1 --bridge-compat --single-request false <"$tmp_in" >"$tmp_out" 2>/dev/null &
host_pid=$!

# Send a request and interrupt quickly.
{
  printf '%s\n' '{"protocol_version":1,"type":"run","request_id":"recovery_a","session_id":"s_recovery","prompt":"long running interrupt","timeout_ms":120000,"idempotency_key":"idem_recovery_a"}'
  sleep 0.05
} >"$tmp_in" &

sleep 0.05
kill "$host_pid" >/dev/null 2>&1 || true
wait "$host_pid" >/dev/null 2>&1 || true

rm -f "$tmp_in" "$tmp_out"

echo "[recovery] verifying next request can still complete" >&2
recovery_req='{"protocol_version":1,"type":"run","request_id":"recovery_b","session_id":"s_recovery","prompt":"after interrupt","timeout_ms":30000,"idempotency_key":"idem_recovery_b"}'
recovery_out=$(printf '%s\n' "$recovery_req" | "$BIN" host --protocol v1 --bridge-compat --single-request)
json_check "$recovery_out"

echo "[recovery] pass: post-interrupt request returned parseable stdout JSON" >&2
