use zene::engine::tools::ToolManager;
use zene::config::AgentConfig;
use zene::engine::mcp::manager::McpManager;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting MCP Verification...");

    // 1. Load Config (should read zene_config.toml)
    let config = AgentConfig::from_env()?;
    info!("Config loaded. MCP Servers defined: {:?}", config.mcp.servers.keys());

    // 2. Initialize Manager
    let mcp_manager = std::sync::Arc::new(McpManager::new(config.mcp));
    
    // 3. Connect
    info!("Connecting to servers...");
    mcp_manager.connect_all().await;
    
    // 4. Initialize ToolManager with injected McpManager
    let context_engine = std::sync::Arc::new(tokio::sync::Mutex::new(zene::engine::context::ContextEngine::new(false)?));
    let tool_manager = ToolManager::new(Some(mcp_manager), context_engine);

    // 5. List Tools
    info!("Listing tools...");
    let tools = tool_manager.list_tools().await;
    
    for tool in &tools {
        println!("- Tool: {} ({})", tool.name, tool.description);
    }
    
    // Check if git tools are present
    let git_tools = tools.iter().filter(|t| t.name.starts_with("git__")).count();
    if git_tools > 0 {
        info!("✅ SUCCESS: Found {} git tools!", git_tools);
    } else {
        info!("❌ WARNING: No git tools found. Is uvx/mcp-server-git working?");
        // Just in case, print all tools
    }

    Ok(())
}
