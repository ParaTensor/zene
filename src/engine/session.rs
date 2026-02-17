use anyhow::Result;
use llm_connector::types::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::info;
use crate::engine::plan::Plan;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Session {
    pub id: String,
    pub history: Vec<Message>,
    pub plan: Option<Plan>, // Store the execution plan
    pub env_vars: HashMap<String, String>, // Session-scoped environment variables
    pub created_at: u64,
    pub last_updated_at: u64,
}

impl Session {
    pub fn new(id: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            id,
            history: Vec::new(),
            plan: None,
            env_vars: HashMap::new(),
            created_at: now,
            last_updated_at: now,
        }
    }
}

pub struct SessionManager {
    sessions: HashMap<String, Session>,
    storage_dir: PathBuf,
}

impl SessionManager {
    pub fn new() -> Result<Self> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let storage_dir = PathBuf::from(&home).join(".zene/sessions");

        if !storage_dir.exists() {
            fs::create_dir_all(&storage_dir)?;
        }

        let mut manager = Self {
            sessions: HashMap::new(),
            storage_dir,
        };

        // Load existing sessions
        if let Err(e) = manager.load_all() {
             info!("Warning: Failed to load sessions: {}", e);
        }

        Ok(manager)
    }

    fn load_all(&mut self) -> Result<()> {
        if !self.storage_dir.exists() {
             return Ok(());
        }
        for entry in fs::read_dir(&self.storage_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                let content = fs::read_to_string(&path)?;
                if let Ok(session) = serde_json::from_str::<Session>(&content) {
                    self.sessions.insert(session.id.clone(), session);
                }
            }
        }
        info!("Loaded {} sessions", self.sessions.len());
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_session(&mut self, id: &str) -> Option<&mut Session> {
        self.sessions.get_mut(id)
    }

    pub fn create_session(&mut self, id: String) -> &mut Session {
        if self.sessions.contains_key(&id) {
            return self.sessions.get_mut(&id).unwrap();
        }

        let session = Session::new(id.clone());
        // Don't auto-save on creation to avoid empty files, wait for first save
        self.sessions.insert(id.clone(), session);
        self.sessions.get_mut(&id).unwrap()
    }

    pub fn save_session(&self, session: &Session) -> Result<()> {
        let path = self.storage_dir.join(format!("{}.json", session.id));
        let content = serde_json::to_string_pretty(session)?;
        fs::write(path, content)?;
        Ok(())
    }
}
