use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::fs;
use std::path::PathBuf;
use tokio::sync::Mutex;

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
            std::fs::create_dir_all(&storage_dir)?;
        }
        Ok(Self { storage_dir })
    }
}

#[async_trait]
impl SessionStore for FileSessionStore {
    async fn load(&self, id: &str) -> Result<Option<Session>> {
        let path = self.storage_dir.join(format!("{}.json", id));
        if !fs::try_exists(&path).await? {
            return Ok(None);
        }
        let content = fs::read_to_string(path).await?;
        let session = serde_json::from_str(&content)?;
        Ok(Some(session))
    }

    async fn save(&self, session: &Session) -> Result<()> {
        let path = self.storage_dir.join(format!("{}.json", session.id));
        let content = serde_json::to_string_pretty(session)?;
        fs::write(path, content).await?;
        Ok(())
    }

    async fn load_all(&self) -> Result<Vec<Session>> {
        let mut sessions = Vec::new();
        if !fs::try_exists(&self.storage_dir).await? {
            return Ok(sessions);
        }
        let mut entries = fs::read_dir(&self.storage_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                let content = fs::read_to_string(&path).await?;
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
        let sessions = self.sessions.lock().await;
        Ok(sessions.get(id).cloned())
    }

    async fn save(&self, session: &Session) -> Result<()> {
        let mut sessions = self.sessions.lock().await;
        sessions.insert(session.id.clone(), session.clone());
        Ok(())
    }

    async fn load_all(&self) -> Result<Vec<Session>> {
        let sessions = self.sessions.lock().await;
        Ok(sessions.values().cloned().collect())
    }
}
