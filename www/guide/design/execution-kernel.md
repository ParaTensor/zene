# Execution Kernel vs Strategy

As Zene evolves from a single agent workflow into an embeddable runtime, the most important architectural boundary is this:

- The execution kernel is the stable runtime substrate.
- Decision strategies are replaceable orchestration policies.
- Product policy chooses which strategy to apply for a given use case.

This separation keeps Zene small at the core while allowing multiple orchestration patterns on top.

## Why This Split Matters

When a system grows, there is strong pressure to keep adding modes to a single orchestrator:

- planning mode
- simple mode
- self-healing mode
- multi-agent mode
- routing mode
- evaluation loops

That path creates a large control-flow object that mixes runtime responsibilities with agent-specific reasoning. The result is difficult to embed, difficult to test, and difficult to extend safely.

For Zene, the goal is not to hard-code one agent philosophy into the engine. The goal is to provide a runtime that can host different orchestration styles without rewriting the execution substrate each time.

## The Three Layers

### 1. Execution Kernel

The execution kernel is the durable core of the system. It should remain stable even as agent patterns change.

Responsibilities:

- manage runtime state and session lifecycle
- execute a concrete step or task
- call tools and record results
- emit structured events
- persist checkpoints and outputs
- handle cancellation, timeout, retries, and failure semantics
- enforce permissions and execution policy

The kernel should not care whether the next step came from a planner, a router, a workflow file, or an external host.

Kernel primitives typically look like:

- execute one step
- apply one state transition
- invoke one tool call
- append one event
- save or restore one checkpoint

### 2. Decision Strategy

Decision strategy is the orchestration layer. It decides what to do next based on the current goal and runtime state.

Responsibilities:

- decide whether planning is needed
- choose the next step
- decide when to branch, merge, retry, or stop
- decide whether verification or reflection should run
- decide how to react to failure

This is where orchestration patterns belong:

- prompt chaining
- routing
- parallelization
- orchestrator-worker
- evaluator-optimizer

These are not kernel features. They are policies built on top of kernel primitives.

### 3. Product Policy

Product policy sits above strategy. It decides which strategy should be used for a given product surface or workload.

Examples:

- use direct execution for short requests
- use planned execution for repo-wide refactors
- use parallel workers only when subproblems are truly isolated
- use evaluator loops only for high-risk outputs

Product policy allows Zene to stay opinionated in a product while keeping the runtime general.

## A Practical Rule: Split on Context Boundaries

One of the most common mistakes in agent systems is to split by role name instead of by context boundary.

Bad split:

- planner agent
- implementer agent
- tester agent

This looks clean on paper, but it often causes context handoff, information loss, and redundant prompting. The tester no longer sees the exact reasoning the implementer used. The implementer no longer carries the planner's full decision context.

Better split:

- one feature-level execution context that owns implementation and validation together
- separate agents or workers only when their context is naturally isolated

Examples of good boundaries:

- auth subsystem vs billing subsystem
- backend migration vs frontend cleanup
- three independent file generation tasks

Examples of bad boundaries:

- one agent for planning and another for coding the same change
- one agent for writing tests and another for implementing the same feature without shared context

The rule is simple: split by what an execution context needs to know, not by what role it sounds like it plays.

## What Belongs in Zene's Kernel

If Zene is positioned as an embeddable execution kernel, these capabilities should be first-class runtime features:

- runtime state management
- session persistence and recovery
- structured event streaming
- tool execution and permission checks
- checkpointing and resumability
- cancellation and timeout control
- artifact tracking
- execution isolation and policy enforcement

These capabilities should work regardless of whether the caller uses a plan-driven strategy, a direct ReAct-like loop, or an external workflow orchestrator.

## What Should Stay Out of the Kernel

The following should be strategy-level or product-level concerns rather than kernel truths:

- always plan first
- always reflect after every task
- always insert repair tasks on failure
- always use planner, executor, and reflector as separate roles
- always represent work as a linear task list

Those are valid defaults, but they should remain replaceable.

## Recommended Strategy Shapes for Zene

Zene does not need dozens of built-in strategies. It needs a few clean ones that prove the boundary is real.

### Direct Execution Strategy

Use for short or exploratory requests.

Properties:

- no up-front plan required
- lower latency
- useful as a default for interactive embedding

### Planned Execution Strategy

This is close to Zene's current Plan -> Execute -> Reflect workflow.

Properties:

- good for larger changes
- useful when verification steps are explicit
- should be implemented as a strategy, not as the runtime itself

### Workflow Strategy

Use when a host provides an explicit task graph or step sequence.

Properties:

- the host owns high-level orchestration
- Zene focuses on execution, observability, and recovery

### Host-Driven Strategy

Use when an IDE or platform decides the next step externally.

Properties:

- the host acts as the planner
- Zene acts as task runner and runtime substrate

## Implications for the Current Architecture

In the current codebase, parts of the runtime and strategy are still coupled together.

Areas that are close to kernel responsibilities:

- tool execution
- session handling
- event emission
- task execution loops

Areas that should move toward strategy responsibilities:

- plan creation as a default first move
- reflection as a mandatory phase
- repair-task insertion logic
- assumptions about linear task execution

The architectural direction should be:

1. keep the runtime substrate small and stable
2. move orchestration decisions into explicit strategies
3. let product surfaces choose strategies without rewriting the engine

Recent steps in that direction look like this:

- runs are now tracked explicitly with lifecycle state and snapshots
- worker mode is a thin process transport around one tracked run
- execution strategy is an explicit request input rather than only implicit config
- planned and direct flows are being separated into a strategy layer above reusable orchestrator helpers

## Design Checklist

When evaluating a new agent feature, ask these three questions first:

1. Is this a new execution primitive or only a new orchestration pattern?
2. Does this split follow a real context boundary or only a role label?
3. Is this a runtime capability, a strategy choice, or a product default?

If the answer is strategy or product default, keep it out of the kernel.

## Summary

Zene should aim to be a small, dependable execution kernel that can host multiple agent orchestration patterns.

The kernel should own execution.

The strategy layer should own decision-making.

The product layer should own defaults.

That separation is what makes the system embeddable instead of merely opinionated.