# Zene Production Backend Execution Tasklist

Status legend:
- [ ] not started
- [~] in progress
- [x] done
- [!] blocked

This document is the execution baseline for replacing the current Copilot CLI login chain with Zene in clawlink/clawops/clawbridge.

## Goals

- Stabilize Zene as a hosted, observable, recoverable backend.
- Deliver a production-safe Host/Worker protocol for clawbridge integration.
- Provide clear rollout gates for Railway deployment.

## Milestones

- M1 (P0): Stable replacement ready (1-2 weeks)
- M2 (P1): Operable and governable service (2-4 weeks)
- M3 (P2): Experience and cost optimization (continuous)

## Epic A: Host/Worker Protocol and Runtime Safety (P0)

### Story A1: Host/Worker command and NDJSON protocol
- [x] Add `zene host --protocol v1` command entry.
- [x] Enforce protocol-only stdout (no log leakage).
- [x] Route all logs and diagnostics to stderr.
- [~] Support request types: run, cancel, ping, session_close.
- [x] Support response types: ack, event, final, error.
- Acceptance:
  - clawbridge can parse 1000+ consecutive responses without corruption.
  - No non-JSON bytes appear on stdout in a 24h soak test.
  - Notes:
    - `cancel` is now supported for active requests via `target_request_id`/`cancel_request_id`; deeper cooperative cancellation in engine internals is still pending.
    - Added integration tests for protocol smoke paths (`ping`, cancel validation).

### Story A2: Request/response schema v1
- [~] Define mandatory fields: protocol_version, type, request_id, session_id, ts_ms.
- [~] Implement run request fields: prompt, metadata, timeout_ms, idempotency_key, stream.
- [x] Implement final response fields: status, text, usage, error, latency_ms.
- [x] Add schema validation and INVALID_REQUEST errors for malformed input.
- Acceptance:
  - Invalid payloads are rejected deterministically with code INVALID_REQUEST.
  - Valid payloads maintain backward compatibility for unknown optional fields.
  - Notes:
    - `session_id` is currently mandatory for `run` but optional for `ping`/`session_close`.
    - Unknown fields are accepted by deserialization and ignored.

### Story A3: Unified error contract
- [x] Add canonical error code enum:
  - TIMEOUT
  - PROVIDER_AUTH
  - PROVIDER_RATE_LIMIT
  - PROVIDER_DOWN
  - INVALID_REQUEST
  - INTERNAL
- [~] Map internal failures to canonical error codes.
- [~] Ensure all top-level failures include code + message + retryable.
- Acceptance:
  - clawbridge can execute retry/degrade policy based only on error code.
  - Zero free-text-only errors at protocol boundary.
  - Notes:
    - Host runtime now uses a canonical `HostErrorCode` enum for protocol boundary error serialization.
    - Engine runtime snapshots now carry structured `error_code` values; host final mapping prefers this typed code and only falls back to message heuristics.
    - Host runtime still keeps heuristic fallback mapping (`PROVIDER_AUTH`, `PROVIDER_RATE_LIMIT`, `PROVIDER_DOWN`, `INTERNAL`) for backward compatibility when typed code is missing.
    - Full provider-subtype coverage remains pending for complete elimination of fallback heuristics.

### Story A4: Per-request timeout and cancellation
- [x] Add request-level timeout enforcement.
- [x] Add cancel message handling by request_id.
- [x] Propagate cancellation into model/tool execution pipeline.
- [~] Ensure canceled/timed-out runs emit single terminal final state.
- Acceptance:
  - Timeout returns final status TIMEOUT and stops execution.
  - Cancel returns final status CANCELED and stops execution.
  - Notes:
    - Host now resolves target requests to engine `run_id` and forwards cancellation through `engine.cancel_run(...)`.
    - Timeout path now also triggers `engine.cancel_run(...)` to stop deep execution instead of only timing out the host wait path.
    - Single-terminal-state hardening remains in progress with additional end-to-end scenario coverage.

### Story A5: Session isolation and persistence policy
- [ ] Treat session_id as first-class key in all execution paths.
- [ ] Define session TTL and cleanup job.
- [ ] Prevent cross-session context contamination.
- [ ] Persist event envelopes with session scoping.
- Acceptance:
  - Multi-session concurrency test shows no cross-talk.
  - Expired sessions are cleaned according to TTL policy.

### Story A6: Idempotency and retry safety
- [x] Add idempotency cache keyed by (session_id, idempotency_key).
- [x] Add dedup window (default 10 min) with bounded capacity.
- [x] Return previous final response when replay is detected.
- [~] Guard tool side effects from duplicate execution.
- Acceptance:
  - Network retry tests do not produce duplicate file writes or commands.
  - Notes:
    - Current implementation uses TTL pruning (10 min) plus hard cap eviction (`50_000`).
    - Host now suppresses duplicate in-flight requests by `(session_id, idempotency_key)` using `DUPLICATE_IN_PROGRESS` ack.

## Epic B: Operability and Governance (P1)

### Story B1: Three-layer observability
- [ ] Business metrics: request count, success rate, P95/P99, timeout rate.
- [ ] Model metrics: provider success rate, token usage, TTFT.
- [ ] Runtime metrics: queue length, active sessions, memory use.
- [ ] Expose metrics endpoint or structured metric stream.
- Acceptance:
  - On-call can identify model/network/executor root cause within 30s.

### Story B2: Structured event stream
- [ ] Standardize event types (PlanningStarted, ToolCall, ReflectionResult, etc.).
- [ ] Guarantee per-request sequence ordering.
- [ ] Emit heartbeat events for long-running requests.
- Acceptance:
  - Upstream can render progress states without waiting final output.

### Story B3: Concurrency governance and backpressure
- [ ] Add global concurrency limit.
- [ ] Enforce single-flight per session.
- [ ] Add bounded queue and queue wait timeout.
- [ ] Return BUSY/429 when overloaded.
- Acceptance:
  - Burst traffic degrades gracefully without runaway memory growth.

### Story B4: Provider routing and fallback
- [ ] Add primary/secondary provider policy.
- [ ] Trigger fallback based on canonical error codes.
- [~] Add template fallback for terminal failure path.
- Acceptance:
  - Single provider instability does not break SLA target.
  - Notes:
    - Host runtime now returns a template fallback text for provider-related terminal failures (`PROVIDER_AUTH`, `PROVIDER_RATE_LIMIT`, `PROVIDER_DOWN`, `TIMEOUT`).

### Story B5: Health probes and self-healing hooks
- [ ] Add healthz (process liveness).
- [ ] Add readyz (dependency readiness).
- [ ] Integrate probe semantics with Railway deployment behavior.
- Acceptance:
  - Railway drains unhealthy instances automatically and recovers.

## Epic C: Quality, Cost, and Security Optimization (P2)

### Story C1: Prompt governance
- [ ] Split system/business/user prompt layers.
- [ ] Add prompt versioning and rollout controls.
- [ ] Enable canary compare and rollback.
- Acceptance:
  - Prompt changes are safely reversible and measurable.

### Story C2: Cost routing policy
- [ ] Route low-complexity tasks to lower-cost models.
- [ ] Route high-complexity tasks to high-capability models.
- [ ] Add cost dashboards by channel and task type.
- Acceptance:
  - Cost decreases without measurable satisfaction regression.

### Story C3: Cache strategy
- [ ] Add short-lived cache for repeated prompts/templates.
- [ ] Define cache key policy and TTL.
- [ ] Add cache hit metrics.
- Acceptance:
  - Hot path latency and provider calls are reduced.

### Story C4: Security and audit controls
- [ ] Redact secrets and PII from logs/events.
- [ ] Add tool-call audit records.
- [ ] Add sensitive keyword and key leakage guardrails.
- Acceptance:
  - Logs remain auditable without exposing secrets.

### Story C5: Regression test framework
- [ ] Build golden dataset for protocol and behavior regression.
- [ ] Add fault injection tests (timeout, provider down, malformed input).
- [ ] Add release gate checks in CI.
- Acceptance:
  - Release pipeline blocks regressions automatically.

## Execution Order (strict)

1. A1 Host/Worker command and NDJSON protocol
2. A2 Request/response schema v1
3. A3 Unified error contract
4. A4 Per-request timeout and cancellation
5. A6 Idempotency and retry safety
6. A5 Session isolation and TTL cleanup
7. B3 Concurrency governance and backpressure
8. B1 Three-layer observability
9. B2 Structured event stream hardening
10. B4 Provider routing and fallback
11. B5 Health probes and Railway readiness
12. C-series optimization stories

## Railway Baseline Config (initial)

- ZENE_MAX_CONCURRENCY=8
- ZENE_SESSION_MAX_CONCURRENCY=1
- ZENE_QUEUE_MAX=200
- ZENE_QUEUE_WAIT_TIMEOUT_MS=5000
- ZENE_REQUEST_TIMEOUT_MS=120000
- ZENE_PROVIDER_TIMEOUT_MS=45000
- ZENE_TOOL_TIMEOUT_MS=60000
- ZENE_CANCEL_GRACE_MS=2000
- ZENE_SESSION_TTL_SEC=86400
- ZENE_SESSION_CLEANUP_INTERVAL_SEC=300
- ZENE_IDEMPOTENCY_TTL_SEC=600
- ZENE_IDEMPOTENCY_MAX_KEYS=50000

## Weekly Tracking Template

### Week N
- Planned:
  - 
- Done:
  - 
- Blockers:
  - 
- Risks:
  - 
- Decisions:
  - 

## Immediate Next Sprint (recommended)

- [~] Implement A1 and A2 end-to-end with integration test harness.
- [x] Define the official `v1` JSON examples for clawbridge team.
- [ ] Add temporary compatibility adapter if old run mode is still needed.

## Execution Log

### 2026-03-19
- Added host mode command: `zene host --protocol v1`.
- Added NDJSON stdin/stdout protocol loop in CLI entry.
- Implemented request routing for `run`, `ping`, `cancel`, `session_close`.
- Implemented structured protocol responses: `ack`, `event`, `final`, `error`.
- Added per-request timeout handling for host `run` requests.
- Added async active-run registry and request-level cancel handling (`target_request_id`/`cancel_request_id`).
- Added idempotency replay cache keyed by `(session_id, idempotency_key)` with 10-minute TTL pruning.
- Verified compilation with `cargo check`.
- Verified host smoke test for `ping` request returns `ack/PONG`.
- Verified `cancel` on unknown target returns structured `INVALID_REQUEST`.
- Added host-side terminal dedup guard using per-run token to avoid duplicate final emissions.
- Added idempotency cache hard cap (`50_000`) with oldest-entry eviction.
- Added canonical host error mapping heuristics for provider/auth/rate-limit/down/internal classes.
- Added in-flight idempotency suppression to prevent duplicate concurrent execution before first terminal state.
- Verified duplicate run requests return `DUPLICATE_IN_PROGRESS` for the second request.
- Added host integration test file `tests/it_host_protocol.rs`.
- Verified `cargo test --test it_host_protocol` passes (3/3).
- Added canonical host error enum and centralized error payload builder at protocol boundary.
- Rebasing and push completed successfully to `origin/main` after conflict resolution.
- Added host terminal template fallback text for provider-related failures.
- Added official `Host Protocol v1 JSON Examples` doc for clawbridge integration.
- Refactored host `run` path to use `engine.submit(...)` and terminal-state polling via engine run snapshots.
- Added engine-level cancellation propagation for both explicit `cancel` requests and timeout-triggered termination.
- Extended `RunSnapshot` with structured `error_code` and updated engine failure recording to persist typed failure category.
- Updated host final error mapping to prioritize snapshot `error_code` over message heuristics.
- Added unit tests for snapshot-based error mapping and final-state shaping in `src/main.rs`.
- Re-verified with `cargo check`, `cargo test --bin zene`, and `cargo test --test it_host_protocol`.
