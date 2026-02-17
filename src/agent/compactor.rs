use anyhow::Result;
use crate::agent::client::AgentClient;
use crate::engine::session::Session;
use llm_connector::types::{Message, Role};
use tracing::info;

pub struct SessionCompactor {
    client: AgentClient,
    threshold: usize,
    keep_recent: usize,
}

impl SessionCompactor {
    pub fn new(client: AgentClient) -> Self {
        Self {
            client,
            threshold: 20, // Default threshold
            keep_recent: 10, // Default recent messages to keep
        }
    }

    pub async fn compact(&self, history: &mut Vec<Message>) -> Result<bool> {
        let history_len = history.len();
        
        if history_len <= self.threshold {
            return Ok(false);
        }

        // We want to keep:
        // 1. The first message (System Prompt usually)
        // 2. The last `keep_recent` messages
        // We summarize everything in between.

        let start_index = 1; // Skip system prompt
        let end_index = history_len.saturating_sub(self.keep_recent);

        if start_index >= end_index {
            return Ok(false); // Nothing to compact
        }

        info!("Compacting session history: summarizing messages {} to {}", start_index, end_index);

        let messages_to_summarize = &history[start_index..end_index];
        let summary_prompt = format!(
            "Please summarize the following conversation history concisely. \
            Focus on key decisions, code changes, and user requirements. \
            Ignore minor chit-chat. \
            History:\n\n{}",
            messages_to_summarize.iter().map(|m| {
                let content = m.content.iter().map(|b| match b {
                    llm_connector::types::MessageBlock::Text { text: t } => t.clone(),
                    _ => "[Non-text content]".to_string(),
                }).collect::<Vec<_>>().join(" ");
                format!("{:?}: {}", m.role, content)
            }).collect::<Vec<_>>().join("\n\n")
        );

        let summary = self.client.chat(&summary_prompt).await?;
        let summary_message = Message::text(Role::System, format!("Older conversation summary: {}", summary));

        // Reconstruct history
        let mut new_history = Vec::new();
        // Keep first message
        if let Some(first) = history.first() {
            new_history.push(first.clone());
        }
        // Add summary
        new_history.push(summary_message);
        // Add recent messages
        new_history.extend_from_slice(&history[end_index..]);

        *history = new_history;
        
        info!("Session compacted. New history length: {}", history.len());

        Ok(true)
    }
}
