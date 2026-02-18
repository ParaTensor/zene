use anyhow::Result;
use zene::engine::context::ContextEngine;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for logs
    tracing_subscriber::fmt::init();

    println!("Initializing Context Engine...");
    let mut context_engine = ContextEngine::new(true)?;

    println!("Indexing current project (this might take a moment)...");
    let root = std::env::current_dir()?;
    let result = context_engine.index_project(&root).await?;
    println!("Indexing Result: {}", result);

    println!("\nPerforming Memory Search for 'memory implementation'...");
    if let Some(memory) = &mut context_engine.memory {
        let results = memory.search("memory implementation", 3).await?;
        for (doc, score) in results {
            println!("\n[Score: {:.4}] File: {}", 1.0 - score, doc.path);
            let snippet: String = doc.content.chars().take(100).collect();
            println!("Content Snippet: {}...", snippet.replace("\n", " "));
        }
    } else {
        println!("Memory module not available.");
    }

    println!("\nPerforming Memory Search for 'agent runner loop'...");
    if let Some(memory) = &mut context_engine.memory {
        let results = memory.search("agent runner loop", 3).await?;
        for (doc, score) in results {
            println!("\n[Score: {:.4}] File: {}", 1.0 - score, doc.path);
            let snippet: String = doc.content.chars().take(100).collect();
            println!("Content Snippet: {}...", snippet.replace("\n", " "));
        }
    }

    Ok(())
}
