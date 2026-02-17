use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::engine::plan::Plan;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRequest {
    pub prompt: String,
    pub session_id: String,
    pub env_vars: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResult {
    pub output: String,
    pub session_id: String,
    pub usage: TokenUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AgentEvent {
    PlanningStarted,
    PlanGenerated(Plan),
    TaskStarted { id: usize, description: String },
    ToolCall { name: String, arguments: serde_json::Value },
    ToolResult { name: String, result: String },
    ReflectionStarted,
    ReflectionResult { passed: bool, reason: String },
    Finished(String),
    Error { code: String, message: String },
}
