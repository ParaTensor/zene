use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::agent::client::AgentClient;
use crate::engine::plan::{Plan, Task, TaskStatus};

pub struct Planner {
    client: AgentClient,
}

#[derive(Serialize, Deserialize)]
struct PlanOutput {
    tasks: Vec<String>,
}

impl Planner {
    pub fn new(client: AgentClient) -> Self {
        Self { client }
    }

    /// Generate a plan for the given goal and context
    pub async fn create_plan(&self, goal: &str, project_context: &str) -> Result<Plan> {
        let system_prompt = r#"You are a Senior Software Architect and Project Planner.
Your goal is to break down a complex user request into a series of clear, sequential, and actionable tasks for a coding agent.

**Guidelines:**
1. **Understand the Goal**: Read the user's request carefully.
2. **Analyze Context**: Use the provided project structure to understand existing files.
3. **Break Down**: Split the work into logical steps (e.g., "Analyze X", "Implement Y", "Refactor Z", "Verify").
4. **Be Specific**: Each task should be a clear instruction (e.g., "Create file src/foo.rs with basic struct" rather than "Do coding").
5. **Dependencies**: Ensure tasks are in the correct execution order.
6. **Verification**: Include a final task to verify the implementation (e.g., "Run tests" or "Check compilation").

**Output Format:**
Return ONLY a JSON object with a single field `tasks`, which is an array of strings.
Example:
{
  "tasks": [
    "Scan src/main.rs to understand current logic",
    "Create src/utils.rs with helper functions",
    "Update src/main.rs to import utils",
    "Run cargo test to verify"
  ]
}
"#;

        let user_prompt = format!(
            "Goal: {}\n\nProject Structure Context:\n{}",
            goal, project_context
        );

        // Call the LLM (Planner Model)
        // Note: In a real implementation, we should use `json_mode` if supported by the provider.
        // For now, we rely on the prompt to enforce JSON.
        let response = self.client.chat(&format!("{}\n\n{}", system_prompt, user_prompt)).await?;
        
        // Parse the JSON response
        let json_str = if let Some(start) = response.find("```json") {
            let after_start = &response[start + 7..];
            if let Some(end) = after_start.find("```") {
                &after_start[..end]
            } else {
                after_start
            }
        } else if let Some(start) = response.find("```") {
             // Sometimes users just use ``` without language
             let after_start = &response[start + 3..];
             if let Some(end) = after_start.find("```") {
                 &after_start[..end]
             } else {
                 after_start
             }
        } else if let Some(start) = response.find("{") {
             // Simple heuristic to extract JSON object
             let end = response.rfind("}").unwrap_or(response.len() - 1) + 1;
             if end > start {
                &response[start..end]
             } else {
                &response
             }
        } else {
            &response
        };

        let plan_output: PlanOutput = serde_json::from_str(json_str.trim())
            .map_err(|e| anyhow::anyhow!("Failed to parse plan JSON: {}. Raw: {}", e, response))?;

        // Convert strings to Task objects
        let tasks = plan_output.tasks.into_iter().enumerate().map(|(i, desc)| {
            Task {
                id: format!("{}", i + 1),
                description: desc,
                status: TaskStatus::Pending,
                dependencies: if i > 0 { vec![format!("{}", i)] } else { vec![] },
            }
        }).collect();

        Ok(Plan {
            goal: goal.to_string(),
            tasks,
            current_task_index: None,
        })
    }
}
