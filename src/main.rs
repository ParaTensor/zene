use clap::{Parser, Subcommand};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};
use tracing::{error, info};

// Internal modules
// Internal modules
// mod agent; // Removed: Use library
// mod engine; // Removed: Use library
// mod config; // Removed: Use library

use zene::agent::runner::AgentRunner;
use zene::config::AgentConfig;
use zene::engine::session::SessionManager;
use zene::engine::tools::{ToolManager, MCP_MANAGER};
use zene::engine::ui::{AutoUserInterface, CliUserInterface, UserInterface};
use zene::engine::mcp::manager::McpManager;
use zene::engine::session::Session;

#[derive(Parser)]
#[command(name = "zene")]
#[command(about = "A minimalist, high-performance coding engine.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Run in server mode (Stdio JSON-RPC)
    #[arg(long, default_value_t = false)]
    server: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute a single prompt and exit
    Run {
        /// The instruction for the agent
        prompt: String,
    },
    /// Start the server (Alias for --server for now)
    Server,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
    id: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcError {
    code: i32,
    message: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    // Initialize logging (stderr only, to keep stdout clean for JSON-RPC)
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    // Initialize MCP Manager
    // We load config separately or extract from AgentConfig if possible, 
    // but AgentConfig currently loads from env.
    // For now, let's just use default or look for a config file later.
    // Since we added `mcp` to `AgentConfig`, we can use that.
    
    if let Ok(config) = AgentConfig::from_env() {
         let mcp_manager = McpManager::new(config.mcp.clone());
         // Connect in background (or await if we want to ensure tools are ready)
         mcp_manager.connect_all().await;
         
         // Set global instance
         if MCP_MANAGER.set(mcp_manager).is_err() {
             error!("Failed to set global MCP Manager");
         }
    }

    let cli = Cli::parse();

    if cli.server || matches!(cli.command, Some(Commands::Server)) {
        info!("Starting Zene in Server Mode (Stdio)...");
        run_server().await?;
    } else if let Some(Commands::Run { prompt }) = cli.command {
        info!("Running one-shot task: {}", prompt);

        // Initialize Agent for one-shot (Use CLI Interface)
        if let Some(runner) = setup_runner(true).await {
            let mut runner = runner;
            // Create a temporary session for one-shot task
            let mut session = Session::new("cli-one-shot".to_string());
            match runner.run(&prompt, &mut session).await {
                Ok(result) => println!("{}", result),
                Err(e) => error!("Task failed: {}", e),
            }
        } else {
            error!("Failed to initialize agent. Check OPENAI_API_KEY.");
        }
    } else {
        // Default to help
        use clap::CommandFactory;
        Cli::command().print_help()?;
    }

    Ok(())
}

async fn setup_runner(use_cli_ui: bool) -> Option<AgentRunner> {
    // Load configuration from environment variables
    match AgentConfig::from_env() {
        Ok(config) => {
            info!("Loaded agent configuration: {:?}", config);
            
            let ui: Box<dyn UserInterface> = if use_cli_ui {
                Box::new(CliUserInterface::new())
            } else {
                Box::new(AutoUserInterface)
            };

            match AgentRunner::new(config, ui) {
                Ok(runner) => Some(runner),
                Err(e) => {
                    error!("Failed to create AgentRunner: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            None
        }
    }
}

async fn run_server() -> anyhow::Result<()> {
    let mut runner = setup_runner(false).await;
    let mut session_manager = SessionManager::new()?;

    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut line = String::new();

    loop {
        line.clear();
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            break; // EOF
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }

        match serde_json::from_str::<JsonRpcRequest>(input) {
            Ok(req) => {
                info!("Received request: {:?}", req.method);
                let response = handle_request(req, &mut runner, &mut session_manager).await;
                let json = serde_json::to_string(&response)?;
                println!("{}", json);
                io::stdout().flush()?;
            }
            Err(e) => {
                error!("Failed to parse JSON: {}", e);
                // Send error response
                let err_resp = JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: "Parse error".into(),
                    }),
                    id: None,
                };
                println!("{}", serde_json::to_string(&err_resp)?);
            }
        }
    }
    Ok(())
}

async fn handle_request(
    req: JsonRpcRequest,
    runner: &mut Option<AgentRunner>,
    session_manager: &mut SessionManager,
) -> JsonRpcResponse {
    let result = match req.method.as_str() {
        "agent.run" => {
            if let Some(runner) = runner {
                let instruction = req
                    .params
                    .get("instruction")
                    .and_then(|v| v.as_str())
                    .unwrap_or("No instruction provided");

                let session_id = req
                    .params
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "default".to_string());

                // Get session first (mutable borrow)
                let session = session_manager.create_session(session_id.clone());

                // Run the task
                let run_result = runner.run(instruction, session).await;
                
                // Clone needed data for saving and responding, to release the borrow
                let session_clone = session.clone();
                
                match run_result {
                    Ok(output) => {
                        // Now we can use session_manager again because `session` borrow is ended (if we don't use `session` anymore)
                        // Actually, to end the borrow, we must not use `session` after this point.
                        // But `session` is still in scope?
                        // Rust NLL (Non-Lexical Lifetimes) should handle this if we don't touch `session`.
                        
                        // However, `session_clone` is a clone of the data. We can pass THAT to save_session.
                        // But wait, save_session is a method on SessionManager.
                        // We need to call session_manager.save_session(&session_clone).
                        // This requires &self (immutable borrow of manager) and &Session.
                        
                        // If we drop `session` (the mutable borrow of manager), we can borrow manager immutably.
                        // Yes!
                        
                        if let Err(e) = session_manager.save_session(&session_clone) {
                            error!("Failed to save session: {}", e);
                        }
                        
                        serde_json::json!({
                            "status": "completed",
                            "message": output,
                            "session_id": session_id
                        })
                    }
                    Err(e) => serde_json::json!({
                        "status": "failed",
                        "error": e.to_string()
                    }),
                }
            } else {
                serde_json::json!({
                    "status": "error",
                    "error": "Agent not initialized (Missing API Key)"
                })
            }
        }
        "tools.list" => {
            let tools = ToolManager::list_tools().await;
            serde_json::json!({ "tools": tools })
        }
        "tools.call" => {
            if let Some(name) = req.params.get("name").and_then(|v| v.as_str()) {
                let args = req
                    .params
                    .get("arguments")
                    .unwrap_or(&serde_json::Value::Null);

                let tool_result = match name {
                    "read_file" => {
                        if let Some(path) = args.get("path").and_then(|v| v.as_str()) {
                            match ToolManager::read_file(path) {
                                Ok(content) => serde_json::json!({ "content": content }),
                                Err(e) => serde_json::json!({ "error": e.to_string() }),
                            }
                        } else {
                            serde_json::json!({ "error": "Missing 'path' argument" })
                        }
                    }
                    "write_file" => {
                        let path = args.get("path").and_then(|v| v.as_str());
                        let content = args.get("content").and_then(|v| v.as_str());

                        if let (Some(p), Some(c)) = (path, content) {
                            match ToolManager::write_file(p, c) {
                                Ok(_) => serde_json::json!({ "status": "success" }),
                                Err(e) => serde_json::json!({ "error": e.to_string() }),
                            }
                        } else {
                            serde_json::json!({ "error": "Missing 'path' or 'content' argument" })
                        }
                    }
                    _ => serde_json::json!({ "error": format!("Unknown tool: {}", name) }),
                };
                tool_result
            } else {
                serde_json::json!({ "error": "Missing 'name' parameter" })
            }
        }
        _ => {
            return JsonRpcResponse {
                jsonrpc: "2.0".into(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: "Method not found".into(),
                }),
                id: req.id,
            };
        }
    };

    JsonRpcResponse {
        jsonrpc: "2.0".into(),
        result: Some(result),
        error: None,
        id: req.id,
    }
}
