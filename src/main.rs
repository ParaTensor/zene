use clap::{Parser, Subcommand};
use dotenv::dotenv;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info};

use zene::ZeneEngine;
use zene::config::AgentConfig;
use zene::engine::session::store::FileSessionStore;
use zene::RunRequest;

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

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: Option<u64>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
    id: Option<u64>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct JsonRpcError {
    code: i32,
    message: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use zene::engine::observability::init_xtrace;

    let config = AgentConfig::from_env().unwrap_or_else(|e| {
        error!("Failed to load config: {}. Using defaults.", e);
        AgentConfig::default()
    });

    let xtrace_layer = if let (Some(endpoint), Some(token)) = (&config.xtrace_endpoint, &config.xtrace_token) {
        init_xtrace(endpoint, token)
    } else {
        None
    };

    tracing_subscriber::registry()
        .with(xtrace_layer)
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let storage_dir = PathBuf::from(&home).join(".zene/sessions");
    let store = Arc::new(FileSessionStore::new(storage_dir)?);
    let engine = Arc::new(ZeneEngine::new(config, store).await?);

    let cli = Cli::parse();

    if cli.server || matches!(cli.command, Some(Commands::Server)) {
        info!("Starting Zene in Server Mode (Stdio)...");
        run_server(engine).await?;
    } else if let Some(Commands::Run { prompt }) = cli.command {
        info!("Running one-shot task: {}", prompt);
        
        let req = RunRequest {
            prompt,
            session_id: "cli-one-shot".to_string(),
            env_vars: None,
        };

        match engine.run(req).await {
            Ok(result) => println!("{}", result.output),
            Err(e) => error!("Task failed: {}", e),
        }
    } else {
        use clap::CommandFactory;
        Cli::command().print_help()?;
    }

    Ok(())
}

async fn run_server(engine: Arc<ZeneEngine>) -> anyhow::Result<()> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut line = String::new();

    loop {
        line.clear();
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }

        match serde_json::from_str::<JsonRpcRequest>(input) {
            Ok(req) => {
                let response = handle_request(req, engine.clone()).await;
                let json = serde_json::to_string(&response)?;
                println!("{}", json);
                io::stdout().flush()?;
            }
            Err(e) => {
                error!("Failed to parse JSON: {}", e);
                let err_resp = JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    result: None,
                    error: Some(JsonRpcError { code: -32700, message: "Parse error".into() }),
                    id: None,
                };
                println!("{}", serde_json::to_string(&err_resp)?);
            }
        }
    }
    Ok(())
}

async fn handle_request(req: JsonRpcRequest, engine: Arc<ZeneEngine>) -> JsonRpcResponse {
    let id = req.id;
    let result = match req.method.as_str() {
        "agent.run" => {
            let instruction = req.params.get("instruction").and_then(|v| v.as_str()).unwrap_or("No instruction provided");
            let session_id = req.params.get("session_id").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_else(|| "default".to_string());

            let run_req = RunRequest {
                prompt: instruction.to_string(),
                session_id,
                env_vars: None,
            };

            match engine.run(run_req).await {
                Ok(res) => serde_json::json!({
                    "status": "completed",
                    "message": res.output,
                    "session_id": res.session_id
                }),
                Err(e) => serde_json::json!({ "status": "failed", "error": e.to_string() }),
            }
        }
        "tools.list" => {
            let tools = engine.tool_manager.list_tools().await;
            serde_json::json!({ "tools": tools })
        }
        "tools.call" => {
            if let Some(name) = req.params.get("name").and_then(|v| v.as_str()) {
                let args = req.params.get("arguments").unwrap_or(&serde_json::Value::Null);
                match name {
                    "read_file" => {
                        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or_default();
                        match engine.tool_manager.read_file(path) {
                            Ok(content) => serde_json::json!({ "content": content }),
                            Err(e) => serde_json::json!({ "error": e.to_string() }),
                        }
                    }
                    _ => serde_json::json!({ "error": format!("Tool {} not implemented in RPC proxy", name) }),
                }
            } else {
                serde_json::json!({ "error": "Missing name" })
            }
        }
        _ => {
            return JsonRpcResponse {
                jsonrpc: "2.0".into(),
                result: None,
                error: Some(JsonRpcError { code: -32601, message: "Method not found".into() }),
                id,
            };
        }
    };

    JsonRpcResponse {
        jsonrpc: "2.0".into(),
        result: Some(result),
        error: None,
        id,
    }
}
