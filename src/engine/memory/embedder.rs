use anyhow::Result;
use fastembed::{InitOptions, TextEmbedding, EmbeddingModel};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Embedder {
    model: Arc<Mutex<TextEmbedding>>,
}

impl Embedder {
    pub fn new() -> Result<Self> {
        let mut options = InitOptions::default();
        options.model_name = EmbeddingModel::AllMiniLML6V2;
        options.show_download_progress = true;
        let model = TextEmbedding::try_new(options)?;
        Ok(Self {
            model: Arc::new(Mutex::new(model)),
        })
    }

    /// Embed a single string into a vector
    pub fn embed_query(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.model.lock().unwrap().embed(vec![text], None)?;
        embeddings.into_iter().next().ok_or_else(|| anyhow::anyhow!("No embedding generated"))
    }

    /// Embed a batch of strings
    pub fn embed_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        let embeddings = self.model.lock().unwrap().embed(texts, None)?;
        Ok(embeddings)
    }
}
