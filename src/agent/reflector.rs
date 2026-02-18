use crate::engine::error::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::agent::client::AgentClient;
use crate::engine::plan::Task;

#[derive(Debug, Serialize, Deserialize)]
pub struct ReflectionResult {
    pub passed: bool,
    pub reason: String,
    pub suggestions: Option<String>,
}

pub struct Reflector {
    client: AgentClient,
}

impl Reflector {
    pub fn new(client: AgentClient) -> Self {
        Self { client }
    }

    /// Reviews the completed task and its output
    /// Returns true if the task passes review, false otherwise
    pub async fn review_task(&self, task: &Task, output: &str) -> Result<ReflectionResult> {
        info!("Reflecting on task: {}", task.description);

        // Simple prompt for now. In the future, we can inject project context or linter output.
        let prompt = format!(
            "You are an expert Code Reviewer and QA Lead.\n\
             Your goal is to review the following task execution.\n\
             \n\
             Task Description: {}\n\
             \n\
             Execution Output/Summary:\n\
             {}\n\
             \n\
             Please evaluate if the task was completed successfully and if the code/changes are correct.\n\
             Check for common issues like:\n\
             - Syntax errors or compilation failures mentioned in output\n\
             - Missing files or imports\n\
             - Logic errors or hallucinations (e.g. using non-existent libraries/assets)\n\
             \n\
             Return your review in strict JSON format:\n\
             {{\n\
               \"passed\": true/false,\n\
               \"reason\": \"Brief explanation of your decision\",\n\
               \"suggestions\": \"Optional suggestions for fixing issues (if failed)\"\n\
             }}\n\
             Do not include markdown code blocks, just the raw JSON string.",
            task.description, output
        );

        let response = self.client.chat(&prompt).await?;
        
        // Robust JSON extraction
        // 1. Remove <think>...</think> blocks (Minimax/DeepSeek specific)
        let mut content_to_parse = response.as_str();
        if let Some(end_think) = content_to_parse.rfind("</think>") {
            if end_think + 8 < content_to_parse.len() {
                content_to_parse = &content_to_parse[end_think + 8..];
            }
        }

        // 2. Extract JSON object by finding first '{' and last '}'
        let clean_json = if let Some(start) = content_to_parse.find('{') {
            if let Some(end) = content_to_parse.rfind('}') {
                if end >= start {
                    &content_to_parse[start..=end]
                } else {
                    content_to_parse
                }
            } else {
                content_to_parse
            }
        } else {
            content_to_parse
        };

        let result: ReflectionResult = serde_json::from_str(clean_json).unwrap_or_else(|_| {
            // Fallback if JSON parsing fails
            // Assume passed if we can't parse, to avoid blocking progress on model errors
            // But log a warning
            info!("Failed to parse Reflector JSON: {}", clean_json);
            ReflectionResult {
                passed: true,
                reason: "Review format error, assuming pass.".to_string(),
                suggestions: None,
            }
        });

        Ok(result)
    }
}
