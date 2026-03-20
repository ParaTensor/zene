# Clawbridge Command Provider Quickstart (MVP)

This page gives copy-paste settings for replacing Copilot login flow with Zene command mode.

Target chain:
- `QQ -> clawlink -> clawops -> clawbridge(command) -> zene -> clawops -> QQ`

## Recommended Command

Use one-shot bridge-compatible response mode:

```bash
zene host --protocol v1 --bridge-compat --single-request
```

Recommended env:

```bash
ZENE_STDIN_TIMEOUT_MS=10000
ZENE_MAX_CONCURRENCY=8
```

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

- `0`: request handled and structured JSON returned
- `2`: protocol/input class failure
- `4`: runtime class failure

Use stdout JSON as primary contract, exit code as secondary operational signal.
