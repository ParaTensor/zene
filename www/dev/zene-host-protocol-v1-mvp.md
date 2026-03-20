# Zene Host Protocol v1 (MVP)

This spec is the minimal contract for command-provider integration.

Primary target flow:
- `QQ -> clawlink -> clawops -> clawbridge(command) -> zene -> clawops -> QQ`

Design goals:
- One request JSON in via stdin.
- One response JSON out via stdout.
- Process may exit after response.
- No event stream in MVP.

For clawbridge command-provider integration, use one-shot main path by default:
- `zene host --protocol v1 --bridge-compat --single-request`
- stream/event protocol should be treated as advanced mode only

## Transport Rules

- stdin: one JSON object per line (NDJSON input).
- stdout: one JSON object per line (NDJSON output).
- stderr: logs and diagnostics only.
- stdout must never contain plain text logs.

## Request Schema (MVP)

Required fields:
- `request_id` (string)
- `session_id` (string)
- `prompt` (string)

Optional fields:
- `channel_id` (string)
- `agent` (string)
- `timeout_ms` (number)
- `idempotency_key` (string)

Example request:

```json
{"protocol_version":1,"type":"run","request_id":"req_001","session_id":"qq_user_42","channel_id":"qq_group_7","agent":"default","prompt":"Summarize this issue in one paragraph.","timeout_ms":30000,"idempotency_key":"msg_9981"}
```

## Response Schema (MVP)

Response is always a single structured JSON object.

Required fields:
- `ok` (boolean)
- `request_id` (string)
- `session_id` (string)
- `text` (string, can be empty on error)
- `error_code` (string or null)
- `error_message` (string or null)
- `usage` (object)

Recommended usage fields:
- `usage.prompt_tokens` (number)
- `usage.completion_tokens` (number)
- `usage.total_tokens` (number)

Successful response example:

```json
{"ok":true,"request_id":"req_001","session_id":"qq_user_42","text":"Here is a one-paragraph summary...","error_code":null,"error_message":null,"usage":{"prompt_tokens":120,"completion_tokens":86,"total_tokens":206}}
```

Error response example:

```json
{"ok":false,"request_id":"req_001","session_id":"qq_user_42","text":"","error_code":"TIMEOUT","error_message":"request exceeded timeout_ms","usage":{"prompt_tokens":0,"completion_tokens":0,"total_tokens":0}}
```

## Validation Rules

Reject as `INVALID_REQUEST` when:
- required field is missing;
- required field is empty after trim;
- JSON cannot be parsed;
- payload exceeds configured input size limit.

The process must not panic on bad input.

## Error Code Set (minimum)

- `INVALID_REQUEST`
- `TIMEOUT`
- `PROVIDER_AUTH`
- `PROVIDER_RATE_LIMIT`
- `PROVIDER_DOWN`
- `INTERNAL`

## Timeouts and Lifecycle

- Apply hard request timeout (suggestion: 30000 ms or 45000 ms).
- On timeout, return one structured response with `error_code=TIMEOUT`.
- Ensure underlying tasks are cancelled and resources released.

## Exit Code Semantics (recommended)

- `0`: structured response completed (including structured business error response).
- `2`: protocol/input error before request execution.
- `3`: preflight/config/provider initialization failure.
- `4`: runtime failure where host/engine cannot finish normal structured handling.

Note:
- clawbridge should trust stdout JSON first; exit code is secondary signal for operations.
