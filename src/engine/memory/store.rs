use anyhow::Result;
use usearch::{Index, IndexOptions, MetricKind, ScalarKind};
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use std::fs;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Document {
    pub id: u64,
    pub path: String,
    pub content: String,
    pub metadata: HashMap<String, String>,
}

pub struct VectorStore {
    index: Index,
    documents: HashMap<u64, Document>,
    storage_path: PathBuf,
    next_id: u64,
}

impl VectorStore {
    pub fn new(storage_path: &Path) -> Result<Self> {
        fs::create_dir_all(storage_path)?;
        
        let index_path = storage_path.join("index.usearch");
        let docs_path = storage_path.join("documents.json");

        let options = IndexOptions {
            dimensions: 384, // fastembed AllMiniLML6V2 output dimension
            metric: MetricKind::Cos, // Cosine similarity
            quantization: ScalarKind::F32,
            connectivity: 16,
            expansion_add: 128,
            expansion_search: 64,
            multi: false,
        };

        let index = if index_path.exists() {
            let index = Index::new(&options)?;
            index.load(index_path.to_str().ok_or(anyhow::anyhow!("Invalid path"))?)?;
            index
        } else {
            Index::new(&options)?
        };

        let documents = if docs_path.exists() {
            let content = fs::read_to_string(&docs_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };

        let next_id = documents.keys().max().copied().unwrap_or(0) + 1;

        Ok(Self {
            index,
            documents,
            storage_path: storage_path.to_path_buf(),
            next_id,
        })
    }

    pub fn add(&mut self, embedding: Vec<f32>, path: String, content: String) -> Result<()> {
        if embedding.len() != 384 {
            return Err(anyhow::anyhow!("Embedding dimension mismatch: expected 384, got {}", embedding.len()));
        }

        let id = self.next_id;
        self.next_id += 1;

        self.index.add(id, &embedding)?;

        let doc = Document {
            id,
            path,
            content,
            metadata: HashMap::new(),
        };
        self.documents.insert(id, doc);

        self.save()?;
        Ok(())
    }

    pub fn search(&self, embedding: Vec<f32>, limit: usize) -> Result<Vec<(Document, f32)>> {
        if embedding.len() != 384 {
             return Err(anyhow::anyhow!("Embedding dimension mismatch: expected 384, got {}", embedding.len()));
        }

        let results = self.index.search(&embedding, limit)?;
        
        let mut docs = Vec::new();
        for (id, distance) in results.keys.iter().zip(results.distances.iter()) {
            if let Some(doc) = self.documents.get(id) {
                // usearch returns distance (1 - similarity for Cosine), so lower is better.
                // We might want to return similarity (1 - distance) if needed, but distance is fine.
                docs.push((doc.clone(), *distance));
            }
        }
        
        Ok(docs)
    }

    pub fn save(&self) -> Result<()> {
        let index_path = self.storage_path.join("index.usearch");
        self.index.save(index_path.to_str().ok_or(anyhow::anyhow!("Invalid path"))?)?;

        let docs_path = self.storage_path.join("documents.json");
        let content = serde_json::to_string_pretty(&self.documents)?;
        fs::write(docs_path, content)?;

        Ok(())
    }
}
