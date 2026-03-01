use clap::{Parser, Subcommand};
use dotenv::dotenv;
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
}

#[derive(Subcommand)]
enum Commands {
    /// Execute a single prompt and exit
    Run {
        /// The instruction for the agent
        prompt: String,
    },
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

    if let Some(Commands::Run { prompt }) = cli.command {
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
