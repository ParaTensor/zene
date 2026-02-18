use anyhow::Result;
use std::env;

pub mod mcp;
use mcp::McpConfig;

#[derive(Debug, Clone)]
pub struct RoleConfig {
    pub provider: String,
    pub model: String,
    pub api_key: String,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub planner: RoleConfig,
    pub executor: RoleConfig,
    pub reflector: RoleConfig,
    pub mcp: McpConfig,
    pub simple_mode: bool,
    pub use_semantic_memory: bool,
    pub xtrace_endpoint: Option<String>,
    pub xtrace_token: Option<String>,
}

impl AgentConfig {
    pub fn from_env() -> Result<Self> {
        // Global defaults
        let default_provider = env::var("LLM_PROVIDER").unwrap_or_else(|_| "openai".to_string());
        let default_model = env::var("LLM_MODEL").unwrap_or_else(|_| "gpt-4o".to_string());
        let default_api_key = env::var("LLM_API_KEY")
            .or_else(|_| env::var("OPENAI_API_KEY"))
            .unwrap_or_default();
        let default_base_url = env::var("LLM_BASE_URL").or_else(|_| env::var("OPENAI_BASE_URL")).ok();
        
        let simple_mode = env::var("ZENE_SIMPLE_MODE")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        let use_semantic_memory = env::var("ZENE_USE_SEMANTIC_MEMORY")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        let planner = Self::load_role_config("PLANNER", &default_provider, &default_model, &default_api_key, &default_base_url);
        let executor = Self::load_role_config("EXECUTOR", &default_provider, &default_model, &default_api_key, &default_base_url);
        let reflector = Self::load_role_config("REFLECTOR", &default_provider, &default_model, &default_api_key, &default_base_url);
        
        // Load MCP config from zene_config.toml if present
        let mcp = if let Ok(content) = std::fs::read_to_string("zene_config.toml") {
             match toml::from_str::<McpConfig>(&content) {
                 Ok(config) => config,
                 Err(e) => {
                     // For now, print error to stderr but don't crash
                     eprintln!("Failed to parse zene_config.toml: {}", e);
                     McpConfig::default()
                 }
             }
        } else {
            McpConfig::default()
        };

        Ok(Self {
            planner,
            executor,
            reflector,
            mcp,
            simple_mode,
            use_semantic_memory,
            xtrace_endpoint: env::var("ZENE_XTRACE_ENDPOINT").ok(),
            xtrace_token: env::var("ZENE_XTRACE_TOKEN").ok(),
        })
    }

    fn load_role_config(
        role: &str,
        default_provider: &str,
        default_model: &str,
        default_api_key: &str,
        default_base_url: &Option<String>,
    ) -> RoleConfig {
        let provider = env::var(format!("ZENE_{}_PROVIDER", role)).unwrap_or_else(|_| default_provider.to_string());
        let model = env::var(format!("ZENE_{}_MODEL", role)).unwrap_or_else(|_| default_model.to_string());
        let api_key = env::var(format!("ZENE_{}_API_KEY", role)).unwrap_or_else(|_| default_api_key.to_string());
        let base_url = env::var(format!("ZENE_{}_BASE_URL", role)).ok().or(default_base_url.clone());

        RoleConfig {
            provider,
            model,
            api_key,
            base_url,
        }
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            planner: RoleConfig::default(),
            executor: RoleConfig::default(),
            reflector: RoleConfig::default(),
            mcp: McpConfig::default(),
            simple_mode: false,
            use_semantic_memory: false,
            xtrace_endpoint: None,
            xtrace_token: None,
        }
    }
}

impl Default for RoleConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            api_key: "".to_string(),
            base_url: None,
        }
    }
}
