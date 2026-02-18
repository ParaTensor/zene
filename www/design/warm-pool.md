# Design: Zene Warm Pool Orchestration

The "Warm Pool" model strikes a balance between single-process isolation and the performance of a long-running server.

## 1. Context & Architecture

```mermaid
graph TD
    UserA((User A)) --> Gate[Gatekeeper App]
    UserB((User B)) --> Gate
    UserC((User C)) --> Gate
    
    subgraph "Warm Pool"
        Z1[Zene Instance 1 - Idle]
        Z2[Zene Instance 2 - Busy]
        Z3[Zene Instance 3 - Idle]
    end
    
    Gate -- "Assign Task" --> Z1
    Gate -- "Queue Task" --> Redis[(Queue/Redis)]
    
    Z2 -- "Streaming logs" --> Gate
    Gate -- "SSE/WS" --> UserB
```

## 2. Gatekeeper Responsibilities

### A. Lifecycle Management
- **Pre-warming**: Gatekeeper launches $N$ instances of `zene server --stdio`. 
- **Health Checks**: Gatekeeper kills hangs and spawns replacements.
- **Scaling**: Dynamically adjust $N$.

### B. Task Queueing
- Requests are queued when all instances are `Busy`.

### C. State Reset (The "Cleaning" Protocol)
1. **Command**: Gatekeeper sends `system.reset` RPC.
2. **Action**: Zene clears conversation, resets envs, and flushes cache.

## 3. Communication Flow (JSON-RPC over Stdio)

1. **Gatekeeper -> Zene**: `agent.run`
2. **Zene -> Gatekeeper**: Streaming events.
3. **Zene -> Gatekeeper**: Result object.
4. **Gatekeeper -> Zene**: `session.reset`

## 4. Benefits
- **Complexity Shift**: Concurrency handled by Gatekeeper.
- **Zene Simplicity**: Zene remains single-threaded.
- **Performance**: Zero startup delay.

## 5. Security Note: Filesystem Isolation
While the *process* is warm, it still shares the physical filesystem. We recommend using separate directories per user and passing `cwd` to Zene.
