# The Planner

The **Planner** is the strategic brain of Zene. It uses a **Configured LLM** (e.g., DeepSeek, OpenAI) to analyze the user's request and the codebase context to generate a structured execution plan.

## Responsibilities

- **Context Analysis**: Scans the project structure to understand dependencies.
- **Task Decomposition**: Breaks complex requirements (e.g., "Refactor the auth module") into atomic steps.
- **Dependency Management**: Determines the order of operations (e.g., "Create interface before implementation").

## Output

The Planner produces a JSON-structured plan containing a list of tasks, which feeds into the execution loop.
