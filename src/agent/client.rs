use anyhow::Result;
use llm_connector::{
    types::{ChatRequest, ChatResponse, Message, Tool, ToolChoice},
    LlmClient,
};
use crate::config::RoleConfig;

#[derive(Clone)]
pub struct AgentClient {
    client: LlmClient,
    model: String,
}

impl AgentClient {
    /// Initialize the agent client with a specific configuration
    pub fn new(config: &RoleConfig) -> Result<Self> {
        let client = if let Some(base_url) = &config.base_url {
            // If base_url is provided, assume OpenAI compatible protocol for most providers
            // This allows connecting to any service that mimics OpenAI API (DeepSeek, LocalAI, etc.)
            LlmClient::openai_with_base_url(&config.api_key, base_url)?
        } else {
            match config.provider.as_str() {
                "openai" => LlmClient::openai(&config.api_key)?,
                "anthropic" => LlmClient::anthropic(&config.api_key)?,
                "deepseek" => LlmClient::deepseek(&config.api_key)?,
                "google" => LlmClient::google(&config.api_key)?,
                "aliyun" => LlmClient::aliyun(&config.api_key)?,
                "zhipu" => LlmClient::zhipu(&config.api_key)?,
                "volcengine" => LlmClient::volcengine(&config.api_key)?,
                "moonshot" => LlmClient::moonshot(&config.api_key)?,
                "xiaomi" => LlmClient::xiaomi(&config.api_key)?,
                // "minimax" => LlmClient::minimax(&config.api_key)?, 
                "ollama" => LlmClient::ollama()?,
                _ => return Err(anyhow::anyhow!("Unsupported provider: {}", config.provider)),
            }
        };

        Ok(Self {
            client,
            model: config.model.clone(),
        })
    }

    #[allow(dead_code)]
    pub fn set_model(&mut self, model: &str) {
        self.model = model.to_string();
    }

    /// Run a chat completion without tools
    #[allow(dead_code)]
    pub async fn chat(&self, prompt: &str) -> Result<String> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![Message::text(llm_connector::types::Role::User, prompt)],
            ..Default::default()
        };

        let response = self.client.chat(&request).await?;
        Ok(response.content)
    }

    /// Run a chat completion with history and tools
    pub async fn chat_with_history(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<Tool>>,
    ) -> Result<ChatResponse> {
        let mut request = ChatRequest {
            model: self.model.clone(),
            messages,
            ..Default::default()
        };

        if let Some(t) = tools {
            request = request.with_tools(t).with_tool_choice(ToolChoice::auto());
        }

        let response = self.client.chat(&request).await?;
        Ok(response)
    }
}
