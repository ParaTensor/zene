use crate::engine::error::{Result, ZeneError};
use llm_connector::{
    types::{ChatRequest, ChatResponse, Message, Tool, ToolChoice},
    LlmClient,
};
use crate::config::RoleConfig;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

#[derive(Clone)]
enum ClientBackend {
    Real(LlmClient),
    Mock(Arc<Mutex<VecDeque<ChatResponse>>>),
}

#[derive(Clone)]
pub struct AgentClient {
    backend: ClientBackend,
    model: String,
}

impl AgentClient {
    /// Initialize the agent client with a specific configuration
    pub fn new(config: &RoleConfig) -> Result<Self> {
        let client = if let Some(base_url) = &config.base_url {
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
                "minimax" => LlmClient::openai_with_base_url(
                    &config.api_key, 
                    "https://api.minimaxi.com/v1"
                )?,
                "ollama" => LlmClient::ollama()?,
                _ => return Err(ZeneError::ConfigError(format!("Unsupported provider: {}", config.provider))),
            }
        };

        Ok(Self {
            backend: ClientBackend::Real(client),
            model: config.model.clone(),
        })
    }

    /// Create a mock client that returns the provided sequence of responses
    pub fn mock(responses: Vec<ChatResponse>) -> Self {
        Self {
            backend: ClientBackend::Mock(Arc::new(Mutex::new(VecDeque::from(responses)))),
            model: "mock-model".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn set_model(&mut self, model: &str) {
        self.model = model.to_string();
    }

    /// Run a chat completion without tools
    #[allow(dead_code)]
    pub async fn chat(&self, prompt: &str) -> Result<String> {
        match &self.backend {
            ClientBackend::Real(client) => {
                let request = ChatRequest {
                    model: self.model.clone(),
                    messages: vec![Message::text(llm_connector::types::Role::User, prompt)],
                    ..Default::default()
                };
                let response = client.chat(&request).await?;
                Ok(response.content)
            }
            ClientBackend::Mock(responses) => {
                let mut guard = responses.lock().unwrap();
                if let Some(response) = guard.pop_front() {
                    Ok(response.content)
                } else {
                    Ok("Mock response exhausted".to_string())
                }
            }
        }
    }

    /// Run a chat completion with history and tools
    pub async fn chat_with_history(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<Tool>>,
    ) -> Result<ChatResponse> {
        match &self.backend {
            ClientBackend::Real(client) => {
                let mut request = ChatRequest {
                    model: self.model.clone(),
                    messages,
                    ..Default::default()
                };

                if let Some(t) = tools {
                    request = request.with_tools(t).with_tool_choice(ToolChoice::auto());
                }

                let response = client.chat(&request).await?;
                Ok(response)
            }
            ClientBackend::Mock(responses) => {
                let mut guard = responses.lock().unwrap();
                if let Some(response) = guard.pop_front() {
                    Ok(response)
                } else {
                    Ok(ChatResponse {
                        content: "Mock response exhausted".to_string(),
                        ..Default::default()
                    })
                }
            }
        }
    }
}
