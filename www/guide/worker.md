# Worker Protocol

Zene exposes a minimal worker mode for hosts that want process isolation without committing to a long-running daemon.

This mode is designed for the simplest embeddable setup:

1. the host spawns one `zene worker` process
2. the host writes one `RunRequest` JSON object to `stdin`
3. Zene streams JSONL messages to `stdout`
4. the process exits when the run finishes

This gives the host a hard process boundary while keeping the protocol small.

## Why Use Worker Mode

Use worker mode when you want:

- process-level isolation from the host application
- a simple spawn-per-request model
- event streaming without embedding Rust directly
- a clean upgrade path toward a future worker pool

Use the Rust library API instead when you want tighter in-process integration and direct runtime control.

## Input

The worker expects exactly one JSON `RunRequest` on `stdin`.

Example:

```json
{
  "prompt": "Analyze the current Rust project and summarize the main modules.",
  "session_id": "analysis-01",
  "env_vars": {
    "RUST_LOG": "info"
  },
  "strategy": "Planned"
}
```

The shape matches the Rust type exported by Zene.

Fields:

- `prompt`: the instruction to run
- `session_id`: the logical session id
- `env_vars`: optional environment variables scoped to the run
- `strategy`: optional execution strategy such as `Planned` or `Direct`

## Output

The worker writes newline-delimited JSON messages to `stdout`.

Each line is a serialized `WorkerMessage`.

Current message variants:

- `RunStarted`
- `Event`
- `Snapshot`
- `TransportError`

### RunStarted

Sent once after the run has been accepted.

```json
{
  "type": "RunStarted",
  "data": {
    "run_id": "7d72d4f6-5fd5-4f1e-9d9e-e4d7f62fd6c4",
    "session_id": "analysis-01"
  }
}
```

### Event

Wraps a normal `AgentEvent` emitted during the run.

```json
{
  "type": "Event",
  "data": {
    "type": "PlanningStarted"
  }
}
```

Example with payload:

```json
{
  "type": "Event",
  "data": {
    "type": "TaskStarted",
    "data": {
      "id": 1,
      "description": "Scan the repository structure"
    }
  }
}
```

### Snapshot

Sent once at the end of the run.

```json
{
  "type": "Snapshot",
  "data": {
    "run_id": "7d72d4f6-5fd5-4f1e-9d9e-e4d7f62fd6c4",
    "session_id": "analysis-01",
    "status": "Completed",
    "started_at": "2026-03-15T12:00:00Z",
    "updated_at": "2026-03-15T12:00:03Z",
    "finished_at": "2026-03-15T12:00:03Z",
    "output": "Task 1: Completed\n",
    "error_message": null
  }
}
```

### TransportError

Used when the worker cannot finish the protocol cleanly.

```json
{
  "type": "TransportError",
  "data": {
    "message": "run snapshot not found for run_id ..."
  }
}
```

## Lifecycle

The expected lifecycle is:

1. host spawns `zene worker`
2. host writes one request to `stdin`
3. worker emits `RunStarted`
4. worker emits zero or more `Event` lines
5. worker emits one terminal `Snapshot`
6. worker exits

This is intentionally minimal. It does not require a daemon, registry, or background actor system.

## Relationship to Runtime Tracking

Worker mode is a transport over the same run lifecycle used by the embedded runtime.

That means the final `Snapshot` is not just a transport detail. It is a serialized `RunSnapshot` produced by the runtime, including:

- `run_id`
- `session_id`
- `status`
- timestamps
- final output
- terminal error message if the run failed

This keeps the worker protocol aligned with the in-process host API.

## Example Host Flow

```bash
echo '{"prompt":"List the top-level modules","session_id":"demo","env_vars":null,"strategy":"Direct"}' \
  | zene worker
```

The host should treat `stdout` as a JSONL stream and parse each line independently.

## Recommended Host Responsibilities

In worker mode, the host should handle:

- process spawn and timeout
- stdout JSONL parsing
- stderr log capture
- retry policy
- OS-level cancellation and cleanup

Zene handles:

- session-aware execution
- agent events
- run snapshots
- cooperative cancellation inside the runtime

## Current Limits

- worker mode currently handles one request per process
- cancellation is still host-driven at the process level in worker mode
- there is no bidirectional mid-run command channel yet

## Notes

- Worker mode is the recommended first integration step when you want isolation without a daemon.
- The protocol is intentionally one-request-per-process.
- This mode is compatible with a future warm worker pool, but does not require one.