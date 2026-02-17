use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::engine::session::Session;

#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn load(&self, id: &str) -> Result<Option<Session>>;
    async fn save(&self, session: &Session) -> Result<()>;
    async fn load_all(&self) -> Result<Vec<Session>>;
}

pub struct FileSessionStore {
    storage_dir: PathBuf,
}

impl FileSessionStore {
    pub fn new(storage_dir: PathBuf) -> Result<Self> {
        if !storage_dir.exists() {
            fs::create_dir_all(&storage_dir)?;
        }
        Ok(Self { storage_dir })
    }
}

#[async_trait]
impl SessionStore for FileSessionStore {
    async fn load(&self, id: &str) -> Result<Option<Session>> {
        let path = self.storage_dir.join(format!("{}.json", id));
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(path)?;
        let session = serde_json::from_str(&content)?;
        Ok(Some(session))
    }

    async fn save(&self, session: &Session) -> Result<()> {
        let path = self.storage_dir.join(format!("{}.json", session.id));
        let content = serde_json::to_string_pretty(session)?;
        fs::write(path, content)?;
        Ok(())
    }

    async fn load_all(&self) -> Result<Vec<Session>> {
        let mut sessions = Vec::new();
        if !self.storage_dir.exists() {
            return Ok(sessions);
        }
        for entry in fs::read_dir(&self.storage_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                let content = fs::read_to_string(&path)?;
                if let Ok(session) = serde_json::from_str::<Session>(&content) {
                    sessions.push(session);
                }
            }
        }
        Ok(sessions)
    }
}

pub struct InMemorySessionStore {
    sessions: Mutex<HashMap<String, Session>>,
}

impl Default for InMemorySessionStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemorySessionStore {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl SessionStore for InMemorySessionStore {
    async fn load(&self, id: &str) -> Result<Option<Session>> {
        let sessions = self.sessions.lock().map_err(|e| anyhow::anyhow!("Lock failed: {}", e))?;
        Ok(sessions.get(id).cloned())
    }

    async fn save(&self, session: &Session) -> Result<()> {
        let mut sessions = self.sessions.lock().map_err(|e| anyhow::anyhow!("Lock failed: {}", e))?;
        sessions.insert(session.id.clone(), session.clone());
        Ok(())
    }

    async fn load_all(&self) -> Result<Vec<Session>> {
        let sessions = self.sessions.lock().map_err(|e| anyhow::anyhow!("Lock failed: {}", e))?;
        Ok(sessions.values().cloned().collect())
    }
}
