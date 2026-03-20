use clap::{Parser, Subcommand};
use dotenv::dotenv;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{mpsc, Mutex, Semaphore};
use tokio::time::{timeout, Duration};
use tracing::{error, info};

use zene_core::config::AgentConfig;
use zene_core::engine::session::store::FileSessionStore;
use zene_core::{ExecutionStrategy, RunRequest, RunSnapshot, RunStatus, ZeneEngine};
use zene_worker::Worker;

#[derive(Parser)]
#[command(name = "zene")]
#[command(about = "A minimalist, high-performance coding engine.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute a single prompt and exit
    Run {
        /// The instruction for the agent
        prompt: String,
    },
    /// Run one request from stdin and stream JSONL events to stdout
    Worker,
    /// Run as a host worker process using NDJSON over stdin/stdout
    Host {
        /// Protocol version name, currently only v1 is supported
        #[arg(long, default_value = "v1")]
        protocol: String,
    },
}

#[derive(Debug, Deserialize)]
struct HostRequest {
    protocol_version: u8,
    #[serde(rename = "type")]
    request_type: String,
    request_id: String,
    session_id: Option<String>,
    ts_ms: Option<i64>,
    prompt: Option<String>,
    metadata: Option<serde_json::Value>,
    timeout_ms: Option<u64>,
    idempotency_key: Option<String>,
    target_request_id: Option<String>,
    cancel_request_id: Option<String>,
    stream: Option<bool>,
}

struct ActiveRun {
    run_token: String,
    engine_run_id: String,
    handle: tokio::task::JoinHandle<()>,
}

#[derive(Clone)]
struct IdempotencyEntry {
    final_payload: serde_json::Value,
    created_at: std::time::Instant,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum HostErrorCode {
    Timeout,
    ProviderAuth,
    ProviderRateLimit,
    ProviderDown,
    InvalidRequest,
    Internal,
}

impl HostErrorCode {
    fn as_str(self) -> &'static str {
        match self {
            HostErrorCode::Timeout => "TIMEOUT",
            HostErrorCode::ProviderAuth => "PROVIDER_AUTH",
            HostErrorCode::ProviderRateLimit => "PROVIDER_RATE_LIMIT",
            HostErrorCode::ProviderDown => "PROVIDER_DOWN",
            HostErrorCode::InvalidRequest => "INVALID_REQUEST",
            HostErrorCode::Internal => "INTERNAL",
        }
    }

    fn default_retryable(self) -> bool {
        match self {
            HostErrorCode::ProviderAuth => false,
            HostErrorCode::InvalidRequest => false,
            HostErrorCode::Timeout
            | HostErrorCode::ProviderRateLimit
            | HostErrorCode::ProviderDown
            | HostErrorCode::Internal => true,
        }
    }
}

fn now_ts_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn map_error_to_contract(message: &str) -> HostErrorCode {
    let m = message.to_lowercase();
    if m.contains("unauthorized") || m.contains("invalid api key") || m.contains("auth") {
        HostErrorCode::ProviderAuth
    } else if m.contains("rate limit") || m.contains("too many requests") || m.contains("429") {
        HostErrorCode::ProviderRateLimit
    } else if m.contains("service unavailable")
        || m.contains("connection refused")
        || m.contains("provider down")
        || m.contains("dns")
        || m.contains("network")
    {
        HostErrorCode::ProviderDown
    } else {
        HostErrorCode::Internal
    }
}

fn map_snapshot_error_to_contract(error_code: Option<&str>, message: &str) -> HostErrorCode {
    if let Some(code) = error_code {
        return match code {
            "PROVIDER_DOWN" => HostErrorCode::ProviderDown,
            "INVALID_REQUEST" => HostErrorCode::InvalidRequest,
            "PROVIDER_ERROR" => map_error_to_contract(message),
            "INTERNAL" => HostErrorCode::Internal,
            "CANCELED" => HostErrorCode::Internal,
            _ => map_error_to_contract(message),
        };
    }
    map_error_to_contract(message)
}

fn template_fallback_text(code: HostErrorCode) -> Option<&'static str> {
    match code {
        HostErrorCode::ProviderAuth => Some("Provider authentication failed. Please verify credentials and retry later."),
        HostErrorCode::ProviderRateLimit => Some("Provider is rate limited. Please retry shortly."),
        HostErrorCode::ProviderDown => Some("Provider is temporarily unavailable. Please retry shortly."),
        HostErrorCode::Timeout => Some("Request timed out. Please retry with a simpler prompt or a higher timeout."),
        HostErrorCode::InvalidRequest | HostErrorCode::Internal => None,
    }
}

fn build_error_payload(
    code: HostErrorCode,
    message: &str,
    retryable: Option<bool>,
) -> serde_json::Value {
    json!({
        "code": code.as_str(),
        "message": message,
        "retryable": retryable.unwrap_or_else(|| code.default_retryable())
    })
}

async fn write_protocol_line(
    stdout: &mut tokio::io::Stdout,
    payload: &serde_json::Value,
) -> anyhow::Result<()> {
    let mut line = serde_json::to_vec(payload)?;
    line.push(b'\n');
    stdout.write_all(&line).await?;
    stdout.flush().await?;
    Ok(())
}

async fn send_protocol_line(
    tx: &mpsc::UnboundedSender<serde_json::Value>,
    payload: serde_json::Value,
) -> anyhow::Result<()> {
    tx.send(payload)
        .map_err(|e| anyhow::anyhow!("failed to send protocol line: {}", e))
}

fn build_final_payload(
    request_id: &str,
    session_id: &str,
    status: &str,
    text: &str,
    usage: serde_json::Value,
    error: Option<serde_json::Value>,
    latency_ms: u64,
) -> serde_json::Value {
    json!({
        "protocol_version": 1,
        "type": "final",
        "request_id": request_id,
        "session_id": session_id,
        "ts_ms": now_ts_ms(),
        "status": status,
        "text": text,
        "usage": usage,
        "error": error,
        "latency_ms": latency_ms
    })
}

async fn write_invalid_request(
    tx: &mpsc::UnboundedSender<serde_json::Value>,
    request_id: &str,
    session_id: Option<&str>,
    message: &str,
) -> anyhow::Result<()> {
    let payload = json!({
        "protocol_version": 1,
        "type": "error",
        "request_id": request_id,
        "session_id": session_id,
        "ts_ms": now_ts_ms(),
        "error": build_error_payload(HostErrorCode::InvalidRequest, message, Some(false))
    });
    send_protocol_line(tx, payload).await
}

async fn write_busy_response(
    tx: &mpsc::UnboundedSender<serde_json::Value>,
    request_id: &str,
    session_id: Option<&str>,
) -> anyhow::Result<()> {
    let payload = json!({
        "protocol_version": 1,
        "type": "error",
        "request_id": request_id,
        "session_id": session_id,
        "ts_ms": now_ts_ms(),
        "error": {
            "code": "BUSY",
            "message": "server is overloaded; retry later",
            "retryable": true,
            "http_status": 429
        }
    });
    send_protocol_line(tx, payload).await
}

async fn handle_run_request(
    engine: Arc<ZeneEngine>,
    tx: mpsc::UnboundedSender<serde_json::Value>,
    active_runs: Arc<Mutex<HashMap<String, ActiveRun>>>,
    idempotency_store: Arc<Mutex<HashMap<String, IdempotencyEntry>>>,
    inflight_idempotency: Arc<Mutex<HashMap<String, String>>>,
    global_slots: Arc<Semaphore>,
    req: HostRequest,
) -> anyhow::Result<()> {
    let request_id = req.request_id.clone();

    let session_id = match req.session_id {
        Some(id) if !id.trim().is_empty() => id,
        _ => {
            return write_invalid_request(
                &tx,
                &request_id,
                None,
                "run request requires non-empty session_id",
            )
            .await;
        }
    };

    let prompt = match req.prompt {
        Some(p) if !p.trim().is_empty() => p,
        _ => {
            return write_invalid_request(
                &tx,
                &request_id,
                Some(&session_id),
                "run request requires non-empty prompt",
            )
            .await;
        }
    };

    let idempotency_key = match req.idempotency_key {
        Some(k) if !k.trim().is_empty() => k,
        _ => {
            return write_invalid_request(
                &tx,
                &request_id,
                Some(&session_id),
                "run request requires non-empty idempotency_key",
            )
            .await;
        }
    };

    {
        let active = active_runs.lock().await;
        if active.contains_key(&request_id) {
            return write_invalid_request(
                &tx,
                &request_id,
                Some(&session_id),
                "request_id is already in progress",
            )
            .await;
        }
    }

    let replay_key = format!("{}:{}", session_id, idempotency_key);

    {
        let inflight = inflight_idempotency.lock().await;
        if let Some(existing_request_id) = inflight.get(&replay_key) {
            let payload = json!({
                "protocol_version": 1,
                "type": "ack",
                "request_id": request_id,
                "session_id": session_id,
                "ts_ms": now_ts_ms(),
                "status": "DUPLICATE_IN_PROGRESS",
                "existing_request_id": existing_request_id
            });
            send_protocol_line(&tx, payload).await?;
            return Ok(());
        }
    }

    {
        let store = idempotency_store.lock().await;
        if let Some(entry) = store.get(&replay_key) {
            if entry.created_at.elapsed() <= Duration::from_secs(600) {
                let mut replay = entry.final_payload.clone();
                if let Some(obj) = replay.as_object_mut() {
                    obj.insert("request_id".to_string(), json!(request_id));
                    obj.insert("session_id".to_string(), json!(session_id));
                    obj.insert("ts_ms".to_string(), json!(now_ts_ms()));
                    obj.insert("replayed".to_string(), json!(true));
                }
                send_protocol_line(&tx, replay).await?;
                return Ok(());
            }
        }
    }

    let global_permit = match global_slots.clone().try_acquire_owned() {
        Ok(permit) => permit,
        Err(_) => {
            return write_busy_response(&tx, &request_id, Some(&session_id)).await;
        }
    };

    let request_ts = req.ts_ms.unwrap_or_else(now_ts_ms);
    let timeout_ms = req.timeout_ms.unwrap_or(120_000);
    let stream = req.stream.unwrap_or(true);

    let ack = json!({
        "protocol_version": 1,
        "type": "ack",
        "request_id": request_id,
        "session_id": session_id,
        "ts_ms": now_ts_ms(),
        "status": "ACCEPTED",
        "request_ts_ms": request_ts
    });
    send_protocol_line(&tx, ack).await?;

    if stream {
        let event = json!({
            "protocol_version": 1,
            "type": "event",
            "request_id": request_id,
            "session_id": session_id,
            "ts_ms": now_ts_ms(),
            "event_type": "REQUEST_ACCEPTED",
            "seq": 1,
            "payload": {
                "timeout_ms": timeout_ms,
                "metadata": req.metadata
            }
        });
        send_protocol_line(&tx, event).await?;
    }

    let run_req = RunRequest {
        prompt,
        session_id: session_id.clone(),
        env_vars: None,
        strategy: Some(ExecutionStrategy::Planned),
    };

    let submitted = engine.submit(run_req).await;
    let mut run_handle = match submitted {
        Ok(handle) => handle,
        Err(e) => {
            let message = e.to_string();
            let code = map_error_to_contract(&message);
            let fallback_text = template_fallback_text(code).unwrap_or("");
            let final_payload = build_final_payload(
                &request_id,
                &session_id,
                "ERROR",
                fallback_text,
                json!({
                    "prompt_tokens": 0,
                    "completion_tokens": 0,
                    "total_tokens": 0
                }),
                Some(build_error_payload(code, &message, None)),
                0,
            );
            send_protocol_line(&tx, final_payload).await?;
            return Ok(());
        }
    };

    let engine_run_id = run_handle.run_id.clone();
    let _ = run_handle.events.close();

    let task_tx = tx.clone();
    let task_engine = engine.clone();
    let task_active_runs = active_runs.clone();
    let task_store = idempotency_store.clone();
    let task_inflight = inflight_idempotency.clone();
    let task_request_id = request_id.clone();
    let task_session_id = session_id.clone();
    let task_replay_key = replay_key.clone();
    let task_engine_run_id = engine_run_id.clone();
    let task_run_token = uuid::Uuid::new_v4().to_string();
    let run_token_for_map = task_run_token.clone();

    let handle = tokio::spawn(async move {
        let _global_permit = global_permit;
        let started_at = std::time::Instant::now();
        let terminal_snapshot = timeout(
            Duration::from_millis(timeout_ms),
            async {
                loop {
                    if let Some(snapshot) = task_engine.get_run_snapshot(&task_engine_run_id).await {
                        if matches!(
                            snapshot.status,
                            RunStatus::Completed | RunStatus::Failed | RunStatus::Cancelled
                        ) {
                            break snapshot;
                        }
                    }
                    tokio::time::sleep(Duration::from_millis(25)).await;
                }
            },
        )
        .await;

        let latency_ms = started_at.elapsed().as_millis() as u64;

        let should_emit_terminal = {
            let mut active = task_active_runs.lock().await;
            match active.get(&task_request_id) {
                Some(run) if run.run_token == task_run_token => {
                    active.remove(&task_request_id);
                    true
                }
                _ => false,
            }
        };

        if !should_emit_terminal {
            return;
        }

        {
            let mut inflight = task_inflight.lock().await;
            inflight.remove(&task_replay_key);
        }

        if stream {
            let event = json!({
                "protocol_version": 1,
                "type": "event",
                "request_id": task_request_id,
                "session_id": task_session_id,
                "ts_ms": now_ts_ms(),
                "event_type": "RUN_FINISHED",
                "seq": 2,
                "payload": {}
            });
            let _ = send_protocol_line(&task_tx, event).await;
        }

        let final_payload = match terminal_snapshot {
            Ok(snapshot) => build_final_from_snapshot(
                &task_request_id,
                &task_session_id,
                snapshot,
                latency_ms,
            ),
            Err(_) => {
                let _ = task_engine.cancel_run(&task_engine_run_id).await;
                build_final_payload(
                    &task_request_id,
                    &task_session_id,
                    "TIMEOUT",
                    template_fallback_text(HostErrorCode::Timeout).unwrap_or(""),
                    json!({
                        "prompt_tokens": 0,
                        "completion_tokens": 0,
                        "total_tokens": 0
                    }),
                    Some(build_error_payload(
                        HostErrorCode::Timeout,
                        "request exceeded timeout_ms",
                        Some(true),
                    )),
                    latency_ms,
                )
            }
        };

        {
            let mut store = task_store.lock().await;
            store.insert(
                task_replay_key,
                IdempotencyEntry {
                    final_payload: final_payload.clone(),
                    created_at: std::time::Instant::now(),
                },
            );
        }

        let _ = send_protocol_line(&task_tx, final_payload).await;
    });

    let mut active = active_runs.lock().await;
    active.insert(
        request_id,
        ActiveRun {
            run_token: run_token_for_map,
            engine_run_id,
            handle,
        },
    );

    {
        let mut inflight = inflight_idempotency.lock().await;
        inflight.insert(replay_key, req.request_id);
    }

    Ok(())
}

fn build_final_from_snapshot(
    request_id: &str,
    session_id: &str,
    snapshot: RunSnapshot,
    latency_ms: u64,
) -> serde_json::Value {
    match snapshot.status {
        RunStatus::Completed => build_final_payload(
            request_id,
            session_id,
            "OK",
            snapshot.output.as_deref().unwrap_or(""),
            json!({
                "prompt_tokens": 0,
                "completion_tokens": 0,
                "total_tokens": 0
            }),
            None,
            latency_ms,
        ),
        RunStatus::Cancelled => build_final_payload(
            request_id,
            session_id,
            "CANCELED",
            "",
            json!({
                "prompt_tokens": 0,
                "completion_tokens": 0,
                "total_tokens": 0
            }),
            None,
            latency_ms,
        ),
        RunStatus::Failed => {
            let message = snapshot
                .error_message
                .clone()
                .unwrap_or_else(|| "run failed".to_string());
            let code = map_snapshot_error_to_contract(snapshot.error_code.as_deref(), &message);
            build_final_payload(
                request_id,
                session_id,
                "ERROR",
                template_fallback_text(code).unwrap_or(""),
                json!({
                    "prompt_tokens": 0,
                    "completion_tokens": 0,
                    "total_tokens": 0
                }),
                Some(build_error_payload(code, &message, None)),
                latency_ms,
            )
        }
        _ => build_final_payload(
            request_id,
            session_id,
            "ERROR",
            "",
            json!({
                "prompt_tokens": 0,
                "completion_tokens": 0,
                "total_tokens": 0
            }),
            Some(build_error_payload(
                HostErrorCode::Internal,
                "run finished with unexpected status",
                Some(true),
            )),
            latency_ms,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_error_code_mapping_takes_precedence() {
        let code = map_snapshot_error_to_contract(
            Some("INVALID_REQUEST"),
            "provider unavailable: upstream unavailable",
        );
        assert_eq!(code, HostErrorCode::InvalidRequest);
    }

    #[test]
    fn test_build_final_from_snapshot_failed_uses_explicit_code() {
        let mut snapshot = RunSnapshot::new("r1".to_string(), "s1".to_string());
        snapshot.status = RunStatus::Failed;
        snapshot.error_message = Some("downstream unavailable".to_string());
        snapshot.error_code = Some("PROVIDER_DOWN".to_string());

        let payload = build_final_from_snapshot("req_1", "s1", snapshot, 123);
        assert_eq!(payload["type"], "final");
        assert_eq!(payload["status"], "ERROR");
        assert_eq!(payload["error"]["code"], "PROVIDER_DOWN");
    }

    #[test]
    fn test_build_final_from_snapshot_cancelled_status() {
        let mut snapshot = RunSnapshot::new("r2".to_string(), "s2".to_string());
        snapshot.status = RunStatus::Cancelled;

        let payload = build_final_from_snapshot("req_2", "s2", snapshot, 5);
        assert_eq!(payload["type"], "final");
        assert_eq!(payload["status"], "CANCELED");
        assert!(payload.get("error").is_none() || payload["error"].is_null());
    }
}

async fn prune_idempotency_cache(store: &Arc<Mutex<HashMap<String, IdempotencyEntry>>>) {
    let ttl = Duration::from_secs(600);
    let max_entries = 50_000usize;
    let mut guard = store.lock().await;
    guard.retain(|_, entry| entry.created_at.elapsed() <= ttl);

    while guard.len() > max_entries {
        if let Some(oldest_key) = guard
            .iter()
            .min_by_key(|(_, entry)| entry.created_at)
            .map(|(k, _)| k.clone())
        {
            guard.remove(&oldest_key);
        } else {
            break;
        }
    }
}

fn read_max_concurrency() -> usize {
    std::env::var("ZENE_MAX_CONCURRENCY")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(8)
}

async fn run_host(engine: Arc<ZeneEngine>, protocol: &str) -> anyhow::Result<()> {
    if protocol != "v1" {
        anyhow::bail!("unsupported protocol: {}", protocol);
    }

    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<serde_json::Value>();

    let writer = tokio::spawn(async move {
        let mut stdout = tokio::io::stdout();
        while let Some(payload) = out_rx.recv().await {
            if let Err(e) = write_protocol_line(&mut stdout, &payload).await {
                error!("Host writer failed: {}", e);
                break;
            }
        }
    });

    let active_runs: Arc<Mutex<HashMap<String, ActiveRun>>> = Arc::new(Mutex::new(HashMap::new()));
    let idempotency_store: Arc<Mutex<HashMap<String, IdempotencyEntry>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let inflight_idempotency: Arc<Mutex<HashMap<String, String>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let global_slots = Arc::new(Semaphore::new(read_max_concurrency()));

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }

        prune_idempotency_cache(&idempotency_store).await;

        let req: HostRequest = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                write_invalid_request(
                    &out_tx,
                    "unknown",
                    None,
                    &format!("invalid JSON payload: {}", e),
                )
                .await?;
                continue;
            }
        };

        if req.protocol_version != 1 {
            write_invalid_request(
                &out_tx,
                &req.request_id,
                req.session_id.as_deref(),
                "unsupported protocol_version (expected 1)",
            )
            .await?;
            continue;
        }

        match req.request_type.as_str() {
            "ping" => {
                let payload = json!({
                    "protocol_version": 1,
                    "type": "ack",
                    "request_id": req.request_id,
                    "session_id": req.session_id,
                    "ts_ms": now_ts_ms(),
                    "status": "PONG"
                });
                send_protocol_line(&out_tx, payload).await?;
            }
            "cancel" => {
                let target = req
                    .target_request_id
                    .or(req.cancel_request_id)
                    .filter(|v| !v.trim().is_empty());

                let Some(target_request_id) = target else {
                    write_invalid_request(
                        &out_tx,
                        &req.request_id,
                        req.session_id.as_deref(),
                        "cancel request requires target_request_id or cancel_request_id",
                    )
                    .await?;
                    continue;
                };

                let engine_run_id = {
                    let active = active_runs.lock().await;
                    active
                        .get(&target_request_id)
                        .map(|run| run.engine_run_id.clone())
                };

                if let Some(run_id) = engine_run_id {
                    let cancelled = engine.cancel_run(&run_id).await;
                    if !cancelled {
                        write_invalid_request(
                            &out_tx,
                            &req.request_id,
                            req.session_id.as_deref(),
                            "target_request_id is not cancelable",
                        )
                        .await?;
                        continue;
                    }

                    let ack = json!({
                        "protocol_version": 1,
                        "type": "ack",
                        "request_id": req.request_id,
                        "session_id": req.session_id,
                        "ts_ms": now_ts_ms(),
                        "status": "CANCEL_ACCEPTED",
                        "target_request_id": target_request_id
                    });
                    send_protocol_line(&out_tx, ack).await?;
                } else {
                    write_invalid_request(
                        &out_tx,
                        &req.request_id,
                        req.session_id.as_deref(),
                        "target_request_id is not running",
                    )
                    .await?;
                }
            }
            "session_close" => {
                let payload = json!({
                    "protocol_version": 1,
                    "type": "ack",
                    "request_id": req.request_id,
                    "session_id": req.session_id,
                    "ts_ms": now_ts_ms(),
                    "status": "SESSION_CLOSED"
                });
                send_protocol_line(&out_tx, payload).await?;
            }
            "run" => {
                if let Err(e) = handle_run_request(
                    engine.clone(),
                    out_tx.clone(),
                    active_runs.clone(),
                    idempotency_store.clone(),
                    inflight_idempotency.clone(),
                    global_slots.clone(),
                    req,
                )
                .await
                {
                    let payload = json!({
                        "protocol_version": 1,
                        "type": "error",
                        "request_id": "unknown",
                        "session_id": serde_json::Value::Null,
                        "ts_ms": now_ts_ms(),
                        "error": build_error_payload(HostErrorCode::Internal, &e.to_string(), Some(true))
                    });
                    send_protocol_line(&out_tx, payload).await?;
                }
            }
            _ => {
                write_invalid_request(
                    &out_tx,
                    &req.request_id,
                    req.session_id.as_deref(),
                    "unsupported request type",
                )
                .await?;
            }
        }
    }

    {
        let mut active = active_runs.lock().await;
        for (_, run) in active.drain() {
            run.handle.abort();
        }
    }

    drop(out_tx);
    if let Err(e) = writer.await {
        error!("Host writer join error: {}", e);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use zene_core::engine::observability::init_xtrace;

    let config = AgentConfig::from_env().unwrap_or_else(|e| {
        error!("Failed to load config: {}. Using defaults.", e);
        AgentConfig::default()
    });

    let xtrace_layer =
        if let (Some(endpoint), Some(token)) = (&config.xtrace_endpoint, &config.xtrace_token) {
            init_xtrace(endpoint, token)
        } else {
            None
        };

    tracing_subscriber::registry()
        .with(xtrace_layer)
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let storage_dir = PathBuf::from(&home).join(".zene/sessions");
    let store = Arc::new(FileSessionStore::new(storage_dir)?);
    let engine = Arc::new(ZeneEngine::new(config, store).await?);

    let cli = Cli::parse();

    if let Some(command) = cli.command {
        match command {
            Commands::Run { prompt } => {
                info!("Running one-shot task: {}", prompt);

                let req = RunRequest {
                    prompt,
                    session_id: "cli-one-shot".to_string(),
                    env_vars: None,
                    strategy: Some(ExecutionStrategy::Planned),
                };

                match engine.run(req).await {
                    Ok(result) => println!("{}", result.output),
                    Err(e) => error!("Task failed: {}", e),
                }
            }
            Commands::Worker => {
                Worker::run(engine.as_ref()).await?;
            }
            Commands::Host { protocol } => {
                info!("Starting host mode with protocol={}", protocol);
                run_host(engine, &protocol).await?;
            }
        }
    } else {
        use clap::CommandFactory;
        Cli::command().print_help()?;
    }

    Ok(())
}
