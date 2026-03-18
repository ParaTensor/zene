use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::engine::contracts::AgentEvent;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RunStatus {
    Pending,
    Running,
    WaitingTool,
    WaitingApproval,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSnapshot {
    pub run_id: String,
    pub session_id: String,
    pub status: RunStatus,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub output: Option<String>,
    pub error_message: Option<String>,
}

impl RunSnapshot {
    pub fn new(run_id: String, session_id: String) -> Self {
        let now = Utc::now();
        Self {
            run_id,
            session_id,
            status: RunStatus::Pending,
            started_at: now,
            updated_at: now,
            finished_at: None,
            output: None,
            error_message: None,
        }
    }

    pub fn mark_status(&mut self, status: RunStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    pub fn mark_completed(&mut self, output: String) {
        let now = Utc::now();
        self.status = RunStatus::Completed;
        self.updated_at = now;
        self.finished_at = Some(now);
        self.output = Some(output);
        self.error_message = None;
    }

    pub fn mark_failed(&mut self, message: String) {
        let now = Utc::now();
        self.status = RunStatus::Failed;
        self.updated_at = now;
        self.finished_at = Some(now);
        self.error_message = Some(message);
    }

    pub fn mark_cancelled(&mut self) {
        let now = Utc::now();
        self.status = RunStatus::Cancelled;
        self.updated_at = now;
        self.finished_at = Some(now);
    }
}

#[derive(Clone, Default)]
pub struct CancellationToken {
    inner: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn cancel(&self) {
        self.inner.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.inner.load(Ordering::SeqCst)
    }
}

pub struct RunHandle {
    pub run_id: String,
    pub session_id: String,
    pub events: mpsc::UnboundedReceiver<AgentEvent>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_snapshot_lifecycle() {
        let mut snapshot = RunSnapshot::new("run-1".to_string(), "session-1".to_string());

        assert_eq!(snapshot.status, RunStatus::Pending);
        assert!(snapshot.finished_at.is_none());

        snapshot.mark_status(RunStatus::Running);
        assert_eq!(snapshot.status, RunStatus::Running);

        snapshot.mark_completed("done".to_string());
        assert_eq!(snapshot.status, RunStatus::Completed);
        assert_eq!(snapshot.output.as_deref(), Some("done"));
        assert!(snapshot.finished_at.is_some());
    }

    #[test]
    fn test_cancellation_token() {
        let token = CancellationToken::default();
        assert!(!token.is_cancelled());

        token.cancel();
        assert!(token.is_cancelled());
    }
}
