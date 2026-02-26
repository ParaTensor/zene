use crate::engine::error::{Result, ZeneError};
use llm_connector::{
    types::{ChatRequest, ChatResponse, Message, Tool, ToolChoice},
    StreamingResponse, LlmClient,
};
use crate::config::RoleConfig;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use futures::Stream;
use std::pin::Pin;

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
    /// Run a streaming chat completion with history and tools
    pub async fn chat_stream_with_history(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<Tool>>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamingResponse>> + Send>>> {
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

                // Map llm_connector errors to ZeneError
                let stream = client.chat_stream(&request).await?;
                let mapped_stream = futures::stream::StreamExt::map(stream, |res| {
                    res.map_err(|e| ZeneError::ProviderError(e.to_string()))
                });
                
                Ok(Box::pin(mapped_stream))
            }
            ClientBackend::Mock(_responses) => {
                // Mock streaming implementation simplified for release
                // TODO: Implement full mock streaming with correct types
                let stream = futures::stream::empty();
                Ok(Box::pin(stream))
            }
        }
    }
    /// Initialize the agent client with a specific configuration
    pub fn new(config: &RoleConfig) -> Result<Self> {
        let client = if let Some(base_url) = &config.base_url {
            // 用户在配置里显式传了自定义 base_url 时优先用用户的
            LlmClient::openai_with_base_url(&config.api_key, base_url)?
        } else {
            // 1. 先根据 config.provider 去 llm_providers 库里查有没有记录
            let providers_data = llm_providers::get_providers_data();
            
            if let Some(provider_info) = providers_data.get(&config.provider) {
                // 如果在 llm_providers 字典里找到了
                let base_url = if let Some(region) = &config.region {
                    // 尝试几种匹配策略
                    provider_info.endpoints.get(region.as_str())
                        .or_else(|| {
                            // 如果用户填的是 cn，尝试匹配 provider 名字本身 (e.g. "zhipu")
                            if region == "cn" {
                                provider_info.endpoints.get(config.provider.as_str())
                            } else { None }
                        })
                        .or_else(|| {
                            // 如果用户填的是 global，尝试匹配 "provider_global" (e.g. "zhipu_global")
                            if region == "global" {
                                let global_key = format!("{}_global", config.provider);
                                provider_info.endpoints.get(global_key.as_str())
                            } else { None }
                        })
                        .or_else(|| {
                            // 最后尝试匹配 "global" 字面量
                            provider_info.endpoints.get("global")
                        })
                        .map(|e| &e.base_url)
                } else {
                    // 如果没指定 region，优先选 "global"，其次选第一个可用的
                    provider_info.endpoints.get("global")
                        .map(|e| &e.base_url)
                        .or_else(|| {
                            provider_info.endpoints.values().next().map(|e| &e.base_url)
                        })
                };

                if let Some(url) = base_url {
                    LlmClient::openai_with_base_url(&config.api_key, url)?
                } else {
                    // 理论上不会走到这里，除非 endpoints 为空
                    return Err(ZeneError::ConfigError(format!("No endpoints found for provider: {}", config.provider)));
                }
            } else {
                // 2. 如果字典里没找到，回退到老写法兜底
                match config.provider.as_str() {
                    "openai" => LlmClient::openai(&config.api_key)?,
                    "anthropic" => LlmClient::anthropic(&config.api_key)?,
                    "deepseek" => LlmClient::deepseek(&config.api_key)?,
                    "google" => LlmClient::google(&config.api_key)?,
                    "ollama" => LlmClient::ollama()?,
                    _ => return Err(ZeneError::ConfigError(format!("Unsupported provider: {}", config.provider))),
                }
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
