# The Reflector

The **Reflector** is the quality assurance lead of Zene. It ensures that the changes made by the Executor actually solve the problem without introducing new issues.

## Responsibilities

- **Outcome Analysis**: Reviews the output of commands (stdout/stderr) and file changes.
- **Verification**: Checks if the acceptance criteria of the plan are met.
- **Self-Healing**: If a task fails or introduces bugs, the Reflector rejects the task and creates a new "Fix" task with specific instructions on what went wrong.

## The Feedback Loop

1.  Executor marks a task as "Complete".
2.  Reflector analyzes the result.
3.  **Pass**: Task is marked finally done. Proceed to next task.
4.  **Fail**: Task is marked failed. A new repair task is inserted into the plan.

This loop guarantees that Zene doesn't just blindly generate code—it verifies functionality before moving on.
