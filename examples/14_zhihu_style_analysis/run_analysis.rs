use dotenv::dotenv;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use zene::agent::client::AgentClient;
use zene::agent::orchestrator::Orchestrator;
use zene::config::{AgentConfig, RoleConfig};
use zene::engine::context::ContextEngine;
use zene::engine::session::store::InMemorySessionStore;
use zene::engine::session::SessionManager;
use zene::engine::tools::ToolManager;
use zene::engine::ui::UserInterface;
use zene::engine::contracts::AgentEvent;
use zene::ExecutionStrategy;
use tokio::sync::mpsc;
use std::fs::{File, OpenOptions};
use std::io::Write;
use chrono::Local;

#[path = "analyzer.rs"]
mod analyzer;

struct LoggingUI {
    log_file: Arc<Mutex<File>>,
}

impl LoggingUI {
    fn new(path: &str) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .expect("Failed to open log file");
        Self {
            log_file: Arc::new(Mutex::new(file)),
        }
    }

    fn log(&self, msg: &str) {
        if let Ok(mut file) = self.log_file.try_lock() {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
            writeln!(file, "[{}] {}", timestamp, msg).ok();
        }
    }
}

impl UserInterface for LoggingUI {
    fn confirm_execution(&self, tool_name: &str, args: &str) -> bool {
        let msg = format!("🤖 Tool Call: {} {}", tool_name, args);
        self.log(&msg);
        true
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    println!("Loading environment...");

    // --- Rust Style Analysis ---
    println!("Running native Rust style analysis...");
    let mut style_analyzer = analyzer::StyleAnalyzer::new("articles", "workspace/index.json");
    if let Ok(count) = style_analyzer.load_articles() {
        if count > 0 {
            let report = style_analyzer.analyze();
            if let Ok(json) = serde_json::to_string_pretty(&report) {
                std::fs::create_dir_all("workspace").ok();
                if std::fs::write("workspace/style_report.json", json).is_ok() {
                    println!("Style analysis completed. Report saved to workspace/style_report.json");
                }
            }
        } else {
            println!("No articles found in 'articles/' directory. Skipping analysis.");
        }
    } else {
        println!("Failed to load articles. Skipping analysis.");
    }
    // ---------------------------

    let mut config = AgentConfig::default();

    // Configure Planner (DeepSeek)
    config.planner = RoleConfig {
        provider: env::var("ZENE_PLANNER_PROVIDER").unwrap_or("deepseek".to_string()),
        model: env::var("ZENE_PLANNER_MODEL").unwrap_or("deepseek-reasoner".to_string()),
        api_key: env::var("ZENE_PLANNER_API_KEY").expect("ZENE_PLANNER_API_KEY must be set"),
        base_url: None, 
        region: None,
    };
    println!("Planner: {} ({})", config.planner.provider, config.planner.model);

    // Configure Executor (Zhipu Global)
    config.executor = RoleConfig {
        provider: env::var("ZENE_EXECUTOR_PROVIDER").unwrap_or("zhipu".to_string()),
        model: env::var("ZENE_EXECUTOR_MODEL").unwrap_or("glm-4-flash".to_string()),
        api_key: env::var("ZENE_EXECUTOR_API_KEY").expect("ZENE_EXECUTOR_API_KEY must be set"),
        base_url: None,
        region: Some(env::var("ZENE_EXECUTOR_REGION").unwrap_or("global".to_string())),
    };
    println!("Executor: {} ({}, region: {:?})", config.executor.provider, config.executor.model, config.executor.region);

    // Configure Reflector (Minimax)
    config.reflector = RoleConfig {
        provider: env::var("ZENE_REFLECTOR_PROVIDER").unwrap_or("minimax".to_string()),
        model: env::var("ZENE_REFLECTOR_MODEL").unwrap_or("abab6.5s-chat".to_string()),
        api_key: env::var("ZENE_REFLECTOR_API_KEY").expect("ZENE_REFLECTOR_API_KEY must be set"),
        base_url: None,
        region: None, 
    };
    println!("Reflector: {} ({})", config.reflector.provider, config.reflector.model);

    // Initialize Clients
    let planner_client = AgentClient::new(&config.planner)?;
    let executor_client = AgentClient::new(&config.executor)?;
    let reflector_client = AgentClient::new(&config.reflector)?;

    // Initialize Engine Components
    let context_engine = Arc::new(Mutex::new(ContextEngine::new(false)?));
    let tool_manager = Arc::new(ToolManager::new(None, context_engine.clone()));
    
    // Create logs directory
    std::fs::create_dir_all("workspace/logs").ok();
    let log_path = "workspace/logs/interaction.log";
    let ui = Box::new(LoggingUI::new(log_path));
    
    // Also create a separate file for structured event dumping
    let event_log_path = "workspace/logs/events.jsonl";
    let event_file = Arc::new(Mutex::new(OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(event_log_path)?));

    let (tx, mut rx) = mpsc::unbounded_channel();
    
    let mut orchestrator = Orchestrator::new(
        config.clone(),
        planner_client,
        executor_client,
        reflector_client,
        tool_manager,
        context_engine,
        ui,
    ).with_event_sender(tx);

    // Session
    let session_store = Arc::new(InMemorySessionStore::new());
    let mut session_manager = SessionManager::new(session_store).await?;
    let mut session = session_manager.create_session("rust-analysis-demo".to_string());

    // Enhanced Task Goal: Focus on style extraction and verification
    let goal = r#"
    Project: Deep Style Analysis & Replication System

    Objective: 
    Not just to analyze the writing style, but to *extract a reusable Style Prompt* and *verify it by generating new content*.
    We want to create a "Virtual Author" that writes exactly like the author of the articles in 'articles/'.

    Workflow:
    1. **Data Gathering**: Read 'workspace/style_report.json' to get statistical data. (Already generated by native analyzer)
    2. **Deep Reading**: Read 2-3 representative articles from 'articles/' (e.g., long technical ones) to understand the tone, logic flow, and rhetorical devices.
    3. **Style Extraction**: 
       - Synthesize a "Style System Prompt" (Instruction). This prompt should teach an AI how to mimic this author.
       - Focus on: "Serious Tone", "Negative Sentence Structure", "Example-Driven Explanation", "Short Paragraphs".
    4. **Verification (The "Turing Test")**: 
       - Use the extracted Style Prompt to write a new short article about "Why Rust is Memory Safe" (or any technical topic).
       - The generated article MUST follow the style constraints strictly.
    5. **Reflection & Iteration**: 
       - Compare the generated article with the original style.
       - If it feels "stiff" or "robotic", refine the Style Prompt to make it more natural.
    6. **Final Output**: 
       - Save the *Final Style Prompt* and the *Generated Article* to 'workspace/style_guide_and_demo.md'.
    "#;

    println!("\n🚀 Starting Orchestrator with Enhanced Logging...");
    println!("Logs will be saved to: {}", log_path);
    println!("Event dump: {}", event_log_path);
    
    // Spawn event listener
    let event_file_clone = event_file.clone();
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            // Log to JSONL
            {
                let mut f = event_file_clone.lock().await;
                if let Ok(json) = serde_json::to_string(&event) {
                    writeln!(f, "{}", json).ok();
                }
            }

            // Console Output
            match event {
                AgentEvent::PlanningStarted => println!("\n[🧠 PLANNER] Thinking..."),
                AgentEvent::PlanGenerated(plan) => println!("\n[🧠 PLANNER] Plan Generated: {} steps", plan.tasks.len()),
                AgentEvent::TaskStarted { id, description } => println!("\n[🔨 EXECUTOR] Task {}: {}", id, description),
                AgentEvent::ToolCall { name, arguments } => println!("\n  > 🛠️  Tool: {} {}", name, arguments),
                AgentEvent::ToolOutputDelta(delta) => {
                    print!("{}", delta);
                    std::io::stdout().flush().ok();
                }
                AgentEvent::ToolResult { name, result } => println!("\n  > 📄 Result ({}): {} chars", name, result.len()),
                AgentEvent::ReflectionStarted => println!("\n[🧐 REFLECTOR] Reviewing..."),
                AgentEvent::ReflectionResult { passed, reason } => {
                    let icon = if passed { "✅" } else { "❌" };
                    println!("[🧐 REFLECTOR] {} {}", icon, reason);
                }
                AgentEvent::Finished(out) => println!("\n[🏁 FINISHED] Result length: {} chars", out.len()),
                AgentEvent::Error { code, message } => eprintln!("\n[❌ ERROR] {} - {}", code, message),
                _ => {}
            }
        }
    });

    match orchestrator.run(goal, &mut session, ExecutionStrategy::Planned).await {
        Ok((_res, _)) => println!("\nOrchestrator completed successfully."),
        Err(e) => eprintln!("\nOrchestrator failed: {}", e),
    }

    Ok(())
}
