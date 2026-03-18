use crate::engine::error::Result;
use llm_connector::types::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use crate::engine::plan::Plan;
pub mod store;
use store::SessionStore;

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
    store: Arc<dyn SessionStore>,
}

impl SessionManager {
    pub async fn new(store: Arc<dyn SessionStore>) -> Result<Self> {
        let mut manager = Self {
            sessions: HashMap::new(),
            store,
        };

        // Load existing sessions
        if let Err(e) = manager.load_all().await {
             info!("Warning: Failed to load sessions: {}", e);
        }

        Ok(manager)
    }

    async fn load_all(&mut self) -> Result<()> {
        let sessions = self.store.load_all().await?;
        for session in sessions {
            self.sessions.insert(session.id.clone(), session);
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
        self.sessions.insert(id.clone(), session);
        self.sessions.get_mut(&id).unwrap()
    }

    pub async fn save_session(&self, session: &Session) -> Result<()> {
        self.store.save(session).await.map_err(crate::engine::error::ZeneError::from)
    }

    pub async fn append_event(&self, session_id: &str, event: &crate::engine::contracts::EventEnvelope) -> Result<()> {
        self.store.append_event(session_id, event).await.map_err(crate::engine::error::ZeneError::from)
    }
}
