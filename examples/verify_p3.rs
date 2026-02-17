use anyhow::Result;
use zene::agent::runner::AgentRunner;
use zene::config::AgentConfig;
use zene::engine::ui::UserInterface;
use zene::engine::session::SessionManager;
use tracing_subscriber;

struct MockUI;

impl UserInterface for MockUI {
    fn confirm_execution(&self, tool_name: &str, args: &str) -> bool {
        println!("[UI Confirmation]: {} {}", tool_name, args);
        true // Auto-confirm
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing with a subscriber that prints to stdout
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).ok();

    println!("Initializing P3 Verification...");
    
    // Load config from env
    // We assume the user has a valid .env or environment variables set for normal running
    let config = AgentConfig::from_env().unwrap_or_else(|e| {
        eprintln!("Failed to load config from env: {}", e);
        panic!("Please ensure environment variables (like OPENAI_API_KEY) are set.");
    });
    
    let ui = Box::new(MockUI);
    let mut runner = AgentRunner::new(config.clone(), ui)?;
    let mut session_manager = SessionManager::new()?;
    let mut session = session_manager.create_session("p3_test_user".to_string());

    println!("Running AgentRunner with simple task...");
    
    let task = "List the files in the current directory and explain what zene_config.toml contains.";
    
    let result = runner.run(task, session).await?;
    
    println!("\n--- Execution Result ---\n{}", result);
    
    println!("\nHistory Length: {}", session.history.len());
    if session.history.len() > 1 {
        println!("History recorded successfully.");
    } else {
        println!("Warning: History seems empty.");
    }

    Ok(())
}
