use clap::{Parser, Subcommand};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;
use std::io::{self, BufRead, Write};
use tracing::{error, info, warn};

// Internal modules
mod agent;
mod engine;

use agent::{AgentClient, AgentRunner};
use engine::session::SessionManager;
use engine::tools::ToolManager;
use engine::ui::{AutoUserInterface, CliUserInterface, UserInterface};

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
            let mut session = engine::session::Session::new("cli-one-shot".to_string());
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
    // Try to get API Key from environment variables
    // Priority: DEEPSEEK_API_KEY > OPENAI_API_KEY

    let deepseek_key = env::var("DEEPSEEK_API_KEY").ok();
    let openai_key = env::var("OPENAI_API_KEY").ok();

    let (provider, api_key) = if let Some(k) = deepseek_key {
        ("deepseek", k)
    } else if let Some(k) = openai_key {
        ("openai", k)
    } else {
        warn!("No supported API key found (DEEPSEEK_API_KEY or OPENAI_API_KEY). Agent capabilities will be disabled.");
        return None;
    };

    info!("Initializing agent with provider: {}", provider);

    match AgentClient::new(provider, &api_key) {
        Ok(client) => {
            let ui: Box<dyn UserInterface> = if use_cli_ui {
                Box::new(CliUserInterface::new())
            } else {
                Box::new(AutoUserInterface)
            };
            match AgentRunner::new(client, ui) {
                Ok(runner) => Some(runner),
                Err(e) => {
                    error!("Failed to create AgentRunner: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            error!("Failed to create AgentClient: {}", e);
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

                let session = session_manager.create_session(session_id.clone());

                match runner.run(instruction, session).await {
                    Ok(output) => {
                        // Auto-save session after run
                        if let Err(e) = session_manager.save_session(session) {
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
            let tools = ToolManager::list_tools();
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
