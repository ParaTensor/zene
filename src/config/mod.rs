use anyhow::Result;
use std::env;

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

        let planner = Self::load_role_config("PLANNER", &default_provider, &default_model, &default_api_key, &default_base_url);
        let executor = Self::load_role_config("EXECUTOR", &default_provider, &default_model, &default_api_key, &default_base_url);
        let reflector = Self::load_role_config("REFLECTOR", &default_provider, &default_model, &default_api_key, &default_base_url);

        Ok(Self {
            planner,
            executor,
            reflector,
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
