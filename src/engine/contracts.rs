use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::engine::plan::Plan;
use chrono::{DateTime, Utc};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub run_id: String,
    pub session_id: String,
    pub seq: u64,
    pub ts: DateTime<Utc>,
    pub event_type: String,
    pub payload: Value,
}

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
pub struct FileChange {
    pub path: String,
    pub change_type: String, // "created", "modified", "deleted"
    pub diff: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AgentEvent {
    PlanningStarted,
    PlanGenerated(Plan),
    TaskStarted { id: usize, description: String },
    ThoughtDelta(String),
    ToolCall { name: String, arguments: serde_json::Value },
    ToolOutputDelta(String),
    ToolResult { name: String, result: String },
    FileStateChanged(FileChange),
    ReflectionStarted,
    ReflectionResult { passed: bool, reason: String },
    Finished(String),
    Error { code: String, message: String },
}
