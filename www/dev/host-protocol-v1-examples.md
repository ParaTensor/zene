# Zene Host Protocol v1 JSON Examples

This document is the official integration example set for clawbridge -> zene host mode.

Note:
- clawbridge command-provider MVP default path is one-shot bridge-compatible mode (`host + --bridge-compat + --single-request`).
- the stream/event examples below are advanced mode references.

## Transport Rules

- stdin: NDJSON (one JSON object per line)
- stdout: NDJSON (one JSON object per line)
- stdout: protocol data only
- stderr: logs and diagnostics only

## 1) Ping

### Request

```json
{"protocol_version":1,"type":"ping","request_id":"ping-001","session_id":"s-user-1"}
```

### Response

```json
{"protocol_version":1,"type":"ack","request_id":"ping-001","session_id":"s-user-1","ts_ms":1773935000000,"status":"PONG"}
```

## 2) Run (stream=true)

### Request

```json
{"protocol_version":1,"type":"run","request_id":"run-001","session_id":"s-user-1","ts_ms":1773935000100,"prompt":"Analyze project structure","metadata":{"channel":"clawbridge","trace_id":"trace-001"},"timeout_ms":120000,"idempotency_key":"idem-001","stream":true}
```

### Responses (typical sequence)

```json
{"protocol_version":1,"type":"ack","request_id":"run-001","session_id":"s-user-1","ts_ms":1773935000101,"status":"ACCEPTED","request_ts_ms":1773935000100}
{"protocol_version":1,"type":"event","request_id":"run-001","session_id":"s-user-1","ts_ms":1773935000102,"event_type":"REQUEST_ACCEPTED","seq":1,"payload":{"timeout_ms":120000,"metadata":{"channel":"clawbridge","trace_id":"trace-001"}}}
{"protocol_version":1,"type":"event","request_id":"run-001","session_id":"s-user-1","ts_ms":1773935002100,"event_type":"RUN_FINISHED","seq":2,"payload":{}}
{"protocol_version":1,"type":"final","request_id":"run-001","session_id":"s-user-1","ts_ms":1773935002101,"status":"OK","text":"Project has src, tests, and docs.","usage":{"prompt_tokens":120,"completion_tokens":40,"total_tokens":160},"error":null,"latency_ms":2000}
```

## 3) Run Timeout

### Request

```json
{"protocol_version":1,"type":"run","request_id":"run-timeout-001","session_id":"s-user-1","prompt":"Very long task","timeout_ms":1000,"idempotency_key":"idem-timeout-001","stream":false}
```

### Response (terminal)

```json
{"protocol_version":1,"type":"final","request_id":"run-timeout-001","session_id":"s-user-1","ts_ms":1773935005000,"status":"TIMEOUT","text":"Request timed out. Please retry with a simpler prompt or a higher timeout.","usage":{"prompt_tokens":0,"completion_tokens":0,"total_tokens":0},"error":{"code":"TIMEOUT","message":"request exceeded timeout_ms","retryable":true},"latency_ms":1002}
```

## 4) Cancel

### Cancel Request

```json
{"protocol_version":1,"type":"cancel","request_id":"cancel-001","session_id":"s-user-1","target_request_id":"run-001"}
```

### Responses

```json
{"protocol_version":1,"type":"ack","request_id":"cancel-001","session_id":"s-user-1","ts_ms":1773935002200,"status":"CANCEL_ACCEPTED","target_request_id":"run-001"}
{"protocol_version":1,"type":"final","request_id":"run-001","session_id":"s-user-1","ts_ms":1773935002201,"status":"CANCELED","text":"","usage":{"prompt_tokens":0,"completion_tokens":0,"total_tokens":0},"error":null,"latency_ms":0}
```

## 5) Invalid Request

### Request (missing required idempotency_key on run)

```json
{"protocol_version":1,"type":"run","request_id":"bad-001","session_id":"s-user-1","prompt":"hello"}
```

### Response

```json
{"protocol_version":1,"type":"error","request_id":"bad-001","session_id":"s-user-1","ts_ms":1773935003000,"error":{"code":"INVALID_REQUEST","message":"run request requires non-empty idempotency_key","retryable":false}}
```

## 6) Idempotency Replay

### First Request

```json
{"protocol_version":1,"type":"run","request_id":"run-a","session_id":"s-user-1","prompt":"hello","timeout_ms":120000,"idempotency_key":"idem-replay-1","stream":false}
```

### Retry with same (session_id, idempotency_key)

```json
{"protocol_version":1,"type":"run","request_id":"run-b","session_id":"s-user-1","prompt":"hello","timeout_ms":120000,"idempotency_key":"idem-replay-1","stream":false}
```

### Replay Response Example

```json
{"protocol_version":1,"type":"final","request_id":"run-b","session_id":"s-user-1","ts_ms":1773935008000,"status":"OK","text":"...","usage":{"prompt_tokens":12,"completion_tokens":5,"total_tokens":17},"error":null,"latency_ms":321,"replayed":true}
```

## 7) In-flight Idempotency Suppression

If a duplicate request arrives while the first one is still running:

```json
{"protocol_version":1,"type":"ack","request_id":"run-b","session_id":"s-user-1","ts_ms":1773935007000,"status":"DUPLICATE_IN_PROGRESS","existing_request_id":"run-a"}
```

## Error Code Reference

- TIMEOUT
- PROVIDER_AUTH
- PROVIDER_RATE_LIMIT
- PROVIDER_DOWN
- INVALID_REQUEST
- INTERNAL
