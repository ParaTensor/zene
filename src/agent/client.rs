use anyhow::Result;
use llm_connector::{
    types::{ChatRequest, ChatResponse, Message, Role, Tool, ToolChoice},
    LlmClient,
};

pub struct AgentClient {
    client: LlmClient,
    model: String,
}

impl AgentClient {
    /// Initialize the agent client with a specific provider
    pub fn new(provider: &str, api_key: &str) -> Result<Self> {
        let client = match provider {
            "openai" => LlmClient::openai(api_key)?,
            "anthropic" => LlmClient::anthropic(api_key)?,
            "deepseek" => LlmClient::deepseek(api_key)?,
            "google" => LlmClient::google(api_key)?,
            "aliyun" => LlmClient::aliyun(api_key)?,
            "zhipu" => LlmClient::zhipu(api_key)?,
            // "tencent" => LlmClient::tencent("default_id", api_key)?, // Temporarily disabled due to API change
            "volcengine" => LlmClient::volcengine(api_key)?,
            "moonshot" => LlmClient::moonshot(api_key)?,
            "xiaomi" => LlmClient::xiaomi(api_key)?,
            "ollama" => LlmClient::ollama()?, // Ollama usually doesn't need key, but builder pattern preferred
            _ => return Err(anyhow::anyhow!("Unsupported provider: {}", provider)),
        };

        // Set default models based on provider
        let model = match provider {
            "openai" => "gpt-4".to_string(),
            "anthropic" => "claude-3-opus-20240229".to_string(),
            "deepseek" => "deepseek-coder".to_string(),
            "google" => "gemini-pro".to_string(),
            "aliyun" => "qwen-max".to_string(),
            "zhipu" => "glm-4".to_string(),
            "tencent" => "hunyuan".to_string(),
            "volcengine" => "doubao-pro".to_string(),
            "moonshot" => "moonshot-v1-32k".to_string(),
            "xiaomi" => "yi-34b-chat".to_string(),
            "ollama" => "llama3".to_string(),
            _ => "gpt-4".to_string(),
        };

        Ok(Self { client, model })
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
            messages: vec![Message::text(Role::User, prompt)],
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
