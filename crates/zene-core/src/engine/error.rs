use thiserror::Error;

#[derive(Error, Debug)]
pub enum ZeneError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Model/LLM error: {0}")]
    ModelError(String),

    #[error("Tool execution error: {0}")]
    ToolError(String),

    #[error("Session error: {0}")]
    SessionError(String),

    #[error("MCP error: {0}")]
    McpError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Anyhow error: {0}")]
    AnyhowError(String),

    #[error("Grep error: {0}")]
    GrepError(String),

    #[error("Tree-sitter error: {0}")]
    TreeSitterError(String),
    #[error("Language error: {0}")]
    LanguageError(String),
    #[error("LLM Connector error: {0}")]
    LlmConnectorError(String),
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("MCP SDK error: {0}")]
    McpSdkError(String),
    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Run cancelled: {0}")]
    Cancelled(String),
}

pub type Result<T> = std::result::Result<T, ZeneError>;

impl From<anyhow::Error> for ZeneError {
    fn from(err: anyhow::Error) -> Self {
        ZeneError::AnyhowError(err.to_string())
    }
}

impl From<tree_sitter::QueryError> for ZeneError {
    fn from(err: tree_sitter::QueryError) -> Self {
        ZeneError::TreeSitterError(err.to_string())
    }
}

impl From<tree_sitter::LanguageError> for ZeneError {
    fn from(err: tree_sitter::LanguageError) -> Self {
        ZeneError::LanguageError(err.to_string())
    }
}

impl From<llm_connector::error::LlmConnectorError> for ZeneError {
    fn from(err: llm_connector::error::LlmConnectorError) -> Self {
        ZeneError::LlmConnectorError(err.to_string())
    }
}

impl From<grep_regex::Error> for ZeneError {
    fn from(err: grep_regex::Error) -> Self {
        ZeneError::GrepError(err.to_string())
    }
}

impl From<zene_mcp::McpError> for ZeneError {
    fn from(err: zene_mcp::McpError) -> Self {
        ZeneError::McpSdkError(err.to_string())
    }
}
