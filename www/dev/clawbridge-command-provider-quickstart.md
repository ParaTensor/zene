# Clawbridge Command Provider Quickstart (MVP)

This page gives copy-paste settings for replacing Copilot login flow with Zene command mode.

Target chain:
- `QQ -> clawlink -> clawops -> clawbridge(command) -> zene -> clawops -> QQ`

Default integration mode for clawbridge is one-shot only:
- `host` + `--bridge-compat` + `--single-request`
- one stdin JSON line in, one stdout JSON line out
- do not use stream/event flow unless explicitly enabling advanced mode

## Recommended Command

Use one-shot bridge-compatible response mode:

```bash
zene host --protocol v1 --bridge-compat --single-request
```

Recommended env:

```bash
ZENE_STDIN_TIMEOUT_MS=10000
ZENE_MAX_CONCURRENCY=8
ZENE_IDEMPOTENCY_TTL_SEC=600
ZENE_IDEMPOTENCY_MAX_KEYS=50000
ZENE_IDEMPOTENCY_REPLAY_MARKER=true
```

## Preflight Check

Run self-test before wiring to clawbridge:

```bash
zene self-test
```

Expected behavior:
- stdout prints one JSON report.
- exit code `0` means config/provider client initialization checks passed.
- exit code `3` means config/provider check failed (for example missing API key).
- self-test JSON includes `missing_required_env` and `provider_probe_results` for CI diagnostics.

## Request Payload (stdin)

Send one JSON line per invocation:

```json
{"protocol_version":1,"type":"run","request_id":"req_1001","session_id":"qq_user_42","channel_id":"qq_group_7","agent":"default","prompt":"请用一句话总结今天的状态。","timeout_ms":30000,"idempotency_key":"msg_1001"}
```

Required fields for MVP:
- `request_id`
- `session_id`
- `prompt`
- `idempotency_key`

Idempotency replay behavior:
- replay hit reuses cached terminal response body.
- if `ZENE_IDEMPOTENCY_REPLAY_MARKER=true` (default), response includes `replayed=true`.
- tune replay window and cache cap with `ZENE_IDEMPOTENCY_TTL_SEC` and `ZENE_IDEMPOTENCY_MAX_KEYS`.

## Response Payload (stdout)

Always one JSON line.

Success example:

```json
{"ok":true,"request_id":"req_1001","session_id":"qq_user_42","text":"今天整体进度正常，核心功能已完成联调。","error_code":null,"error_message":null,"usage":{"prompt_tokens":123,"completion_tokens":48,"total_tokens":171}}
```

Failure example:

```json
{"ok":false,"request_id":"req_1001","session_id":"qq_user_42","text":"","error_code":"TIMEOUT","error_message":"request exceeded timeout_ms","usage":{"prompt_tokens":0,"completion_tokens":0,"total_tokens":0}}
```

## Timeout Coordination

Set time budgets to keep fallback window:
- `zene_timeout_ms < clawbridge_timeout_ms`
- suggestion: `zene_timeout_ms=30000`, `clawbridge_timeout_ms=40000`

## Degrade Trigger Rules

Minimum fallback triggers in clawbridge:
- `TIMEOUT`
- `PROVIDER_DOWN`

Recommended fallback after retry budget:
- `PROVIDER_RATE_LIMIT`
- `INTERNAL`

No retry:
- `INVALID_REQUEST`
- `PROVIDER_AUTH`

## Exit Code Semantics

- `0`: structured JSON response written to stdout (success or business error)
- `2`: protocol/input class failure before request execution
- `3`: preflight/config/provider initialization failure (`zene self-test` fail)
- `4`: runtime class failure where host/engine cannot finish normal structured handling

Use stdout JSON as primary contract, exit code as secondary operational signal.

Advanced mode:
- stream/event protocol examples are documented in `host-protocol-v1-examples.md` and are not the default clawbridge command-provider path.

Runtime observability baseline:
- stderr logs include host runtime metrics snapshot fields:
	- timeout rate
	- cancel success rate
	- average cleanup duration (ms)

Stability baseline check:
- run `examples/15_host_soak_and_recovery.sh 50` to verify continuous parseability and post-interruption recovery.
