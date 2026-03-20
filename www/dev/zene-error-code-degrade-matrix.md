# Zene Error Code to Clawbridge Degrade Matrix (MVP)

This table defines deterministic handling for command-provider integration.

Routing contract:
- clawbridge must route by structured `error_code` only.
- do not rely on free-text `error_message` keyword matching for degrade decisions.

## Mapping Table

| error_code | Meaning | clawbridge action | Retry | Fallback template |
|---|---|---|---|---|
| INVALID_REQUEST | Input schema or required field invalid | Return validation message upstream | No | No |
| TIMEOUT | Request exceeded zene hard timeout | Trigger degrade path immediately | Optional short retry once | Yes |
| PROVIDER_AUTH | Credential/auth issue | Mark provider unavailable for this request | No | Yes |
| PROVIDER_RATE_LIMIT | Upstream rate limit | Retry with backoff if budget allows | Yes | Yes |
| PROVIDER_DOWN | Provider/network unavailable | Trigger degrade path immediately | Optional | Yes |
| INTERNAL | Unexpected runtime failure | Retry once if safe, else degrade | Optional | Yes |

## Degrade Policy (MVP)

Trigger fallback template when:
- `error_code` is `TIMEOUT` or `PROVIDER_DOWN`.

Recommended to also fallback for:
- `PROVIDER_AUTH`
- `PROVIDER_RATE_LIMIT` (after retry budget exhausted)
- `INTERNAL` (after one safe retry)

## Retry Policy (MVP baseline)

- `INVALID_REQUEST`: 0 retries.
- `TIMEOUT`: 0-1 retry if bridge timeout budget remains.
- `PROVIDER_AUTH`: 0 retries.
- `PROVIDER_RATE_LIMIT`: up to 1 retry with short backoff.
- `PROVIDER_DOWN`: 0-1 retry.
- `INTERNAL`: 0-1 retry.

## Timeout Budget Coordination

To avoid bridge timeout before zene finishes:
- enforce `zene_timeout_ms < clawbridge_timeout_ms`.
- keep a response buffer window for transport and fallback rendering.

Suggested baseline:
- `zene_timeout_ms = 30000`
- `clawbridge_timeout_ms = 35000-45000`

## Observability Fields for Routing Decisions

Minimum fields to emit/record per request:
- `request_id`
- `session_id`
- `error_code`
- `latency_ms`
- `status`

These fields are enough to separate provider instability from local execution failures.
