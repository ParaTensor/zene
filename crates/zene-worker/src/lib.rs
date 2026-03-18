use zene_core::{ZeneEngine, RunSnapshot, AgentEvent, RunRequest};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use tracing::error;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WorkerMessage {
    RunStarted {
        run_id: String,
        session_id: String,
    },
    Event(AgentEvent),
    Snapshot(RunSnapshot),
    TransportError {
        message: String,
    },
}

/// Worker handles the IPC protocol for Zene, reading requests from stdin
/// and streaming JSONL messages to stdout.
pub struct Worker;

impl Worker {
    /// Start the worker loop. This is a blocking call that consumes stdin until EOF.
    pub async fn run(engine: &ZeneEngine) -> anyhow::Result<()> {
        let request = match Self::read_request() {
            Ok(req) => req,
            Err(e) => {
                // If we can't even read the request, we log to stderr and exit
                error!("Worker failed to read request from stdin: {}", e);
                anyhow::bail!("Invalid worker request: {}", e);
            }
        };

        let handle = engine.submit(request).await?;

        // 1. Signal that the run has started
        Self::write_message(&WorkerMessage::RunStarted {
            run_id: handle.run_id.clone(),
            session_id: handle.session_id.clone(),
        })?;

        let run_id = handle.run_id.clone();
        let mut events = handle.events;

        // 2. Stream events as they happen
        while let Some(event) = events.recv().await {
            Self::write_message(&WorkerMessage::Event(event))?;
        }

        // 3. Final snapshot delivery
        if let Some(snapshot) = engine.get_run_snapshot(&run_id).await {
            Self::write_message(&WorkerMessage::Snapshot(snapshot))?;
            return Ok(());
        }

        Self::write_message(&WorkerMessage::TransportError {
            message: format!("run snapshot not found for run_id {}", run_id),
        })?;

        Ok(())
    }

    fn read_request() -> anyhow::Result<RunRequest> {
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input)?;

        if input.trim().is_empty() {
            anyhow::bail!("worker expected a JSON RunRequest on stdin")
        }

        Ok(serde_json::from_str(input.trim())?)
    }

    fn write_message(message: &WorkerMessage) -> anyhow::Result<()> {
        let mut stdout = std::io::stdout().lock();
        serde_json::to_writer(&mut stdout, message)?;
        stdout.write_all(b"\n")?;
        stdout.flush()?;
        Ok(())
    }
}
