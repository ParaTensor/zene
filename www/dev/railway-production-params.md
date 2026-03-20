# Railway Production Parameters (MVP)

This page is the minimum runbook for clawbridge command-provider deployment on Railway.

## Recommended startup mode

Use one-shot command mode:

```bash
zene host --protocol v1 --bridge-compat --single-request
```

## Recommended environment baseline

```bash
ZENE_MAX_CONCURRENCY=8
ZENE_STDIN_TIMEOUT_MS=10000
ZENE_IDEMPOTENCY_TTL_SEC=600
ZENE_IDEMPOTENCY_MAX_KEYS=50000
ZENE_IDEMPOTENCY_REPLAY_MARKER=true
```

Timeout budget recommendation:
- request timeout in payload: `30000`
- clawbridge timeout: `40000`
- keep `zene_timeout_ms < clawbridge_timeout_ms`

Retry budget recommendation:
- `TIMEOUT`: retry at most once if budget remains
- `PROVIDER_DOWN`: retry at most once
- `PROVIDER_RATE_LIMIT`: retry with short backoff once
- `INVALID_REQUEST`, `PROVIDER_AUTH`: no retry

## Fault triage order

When a request fails, inspect in this order:
1. stdout contract (`ok`, `error_code`, `error_message`)
2. stderr logs (runtime metrics and diagnostics)
3. process exit code (`0/2/3/4`)

## Runtime metrics to monitor

Minimum metrics emitted in stderr snapshots:
- timeout rate
- cancel success rate
- average cleanup duration in milliseconds

These are enough to detect timeout pressure, cancellation health, and cleanup regressions.
