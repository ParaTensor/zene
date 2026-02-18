use anyhow::{Result, Context};
use std::path::Path;
use tracing::info;

pub mod embedder;
pub mod store;

use embedder::Embedder;
use store::{VectorStore, Document};

#[derive(Clone)]
pub struct MemoryManager {
    embedder: Embedder,
    store: VectorStore,
}

impl MemoryManager {
    pub fn new(project_root: &Path) -> Result<Self> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        
        // Use a hash of the project path to create a unique storage directory
        let project_hash = format!("{:x}", md5::compute(project_root.to_string_lossy().as_bytes()));
        let storage_path = Path::new(&home).join(".zene/memory").join(project_hash);

        info!("Initializing MemoryManager at {:?}", storage_path);

        let embedder = Embedder::new().context("Failed to initialize Embedder")?;
        let store = VectorStore::new(&storage_path).context("Failed to initialize VectorStore")?;

        Ok(Self {
            embedder,
            store,
        })
    }
}

use ignore::WalkBuilder;
use std::fs;

impl MemoryManager {
    // ... existing new ...

    pub async fn index_project(&mut self, root: &Path) -> Result<String> {
        let mut count = 0;
        let walker = WalkBuilder::new(root)
            .hidden(false)
            .git_ignore(true)
            .build();

        for result in walker {
            match result {
                Ok(entry) => {
                    if entry.file_type().map_or(false, |ft| ft.is_file()) {
                        let path = entry.path();
                        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                            let ext = ext.to_lowercase();
                            if matches!(ext.as_str(), "rs" | "md" | "toml" | "json" | "py" | "js" | "ts" | "sh" | "html" | "css") {
                                if let Ok(content) = fs::read_to_string(path) {
                                    if content.len() > 0 && content.len() < 100 * 1024 {
                                        let relative_path = path.strip_prefix(root).unwrap_or(path).to_string_lossy().to_string();
                                        if let Err(e) = self.index_text(&content, &relative_path, None).await {
                                            tracing::warn!("Failed to index {}: {}", relative_path, e);
                                        } else {
                                            count += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(err) => tracing::warn!("Error walking directory: {}", err),
            }
        }
        self.store.save().await?;
        Ok(format!("Indexed {} files.", count))
    }

    pub async fn index_text(&mut self, text: &str, path: &str, _metadata: Option<std::collections::HashMap<String, String>>) -> Result<()> {
        let embedding = self.embedder.embed_query(text)?;
        self.store.add(embedding, path.to_string(), text.to_string()).await?;
        Ok(())
    }

    pub async fn search(&mut self, query: &str, limit: usize) -> Result<Vec<(Document, f32)>> {
        let embedding = self.embedder.embed_query(query)?;
        self.store.search(embedding, limit).await
    }
}
