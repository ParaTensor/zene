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
use zene_core::agent::AgentClient;
use zene_core::{ExecutionStrategy, RunRequest, RunSnapshot, RunStatus, ZeneEngine};
use zene_worker::Worker;

const EXIT_OK: i32 = 0;
const EXIT_PROTOCOL_ERROR: i32 = 2;
const EXIT_CONFIG_ERROR: i32 = 3;
const EXIT_RUNTIME_ERROR: i32 = 4;

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
    /// Run minimal health checks and print one JSON result
    SelfTest,
    /// Run as a host worker process using NDJSON over stdin/stdout
    Host {
        /// Protocol version name, currently only v1 is supported
        #[arg(long, default_value = "v1")]
        protocol: String,
        /// Exit after handling one request (MVP default)
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        single_request: bool,
        /// Timeout for waiting first/next stdin line
        #[arg(long)]
        stdin_timeout_ms: Option<u64>,
        /// Emit clawbridge-friendly flat response JSON shape
        #[arg(long, default_value_t = false)]
        bridge_compat: bool,
    },
}

fn redact_secret(value: &str) -> String {
    if value.is_empty() {
        return "".to_string();
    }
    if value.len() <= 4 {
        return "****".to_string();
    }
    let suffix = &value[value.len() - 4..];
    format!("****{}", suffix)
}

fn run_self_test(config: &AgentConfig) -> serde_json::Value {
    let mut checks = Vec::new();

    let roles = [
        ("planner", &config.planner),
        ("executor", &config.executor),
        ("reflector", &config.reflector),
    ];

    for (role, cfg) in roles {
        let api_key_present = !cfg.api_key.trim().is_empty();
        let provider = cfg.provider.clone();
        let model = cfg.model.clone();

        let client_init = AgentClient::new(cfg);
        let (ok, error_message) = match client_init {
            Ok(_) => (api_key_present, if api_key_present { None } else { Some("api_key is empty".to_string()) }),
            Err(e) => (false, Some(e.to_string())),
        };

        checks.push(json!({
            "role": role,
            "provider": provider,
            "model": model,
            "api_key_present": api_key_present,
            "api_key_hint": redact_secret(&cfg.api_key),
            "ok": ok,
            "error": error_message,
        }));
    }

    let all_ok = checks
        .iter()
        .all(|c| c.get("ok").and_then(|v| v.as_bool()).unwrap_or(false));

    json!({
        "ok": all_ok,
        "status": if all_ok { "PASS" } else { "FAIL" },
        "checks": checks,
    })
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum HostResponseMode {
    Protocol,
    BridgeCompat,
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

fn bridge_compat_from_protocol(payload: &serde_json::Value) -> serde_json::Value {
    let request_id = payload
        .get("request_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let session_id = payload
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let status = payload
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("ERROR");
    let text = payload.get("text").and_then(|v| v.as_str()).unwrap_or("");

    if status == "OK" {
        return json!({
            "ok": true,
            "request_id": request_id,
            "session_id": session_id,
            "text": text,
            "error_code": serde_json::Value::Null,
            "error_message": serde_json::Value::Null,
            "usage": payload.get("usage").cloned().unwrap_or_else(|| json!({}))
        });
    }

    let error_code = payload
        .get("error")
        .and_then(|e| e.get("code"))
        .cloned()
        .unwrap_or_else(|| json!("INTERNAL"));
    let error_message = payload
        .get("error")
        .and_then(|e| e.get("message"))
        .cloned()
        .unwrap_or_else(|| json!("request failed"));

    json!({
        "ok": false,
        "request_id": request_id,
        "session_id": session_id,
        "text": "",
        "error_code": error_code,
        "error_message": error_message,
        "usage": payload.get("usage").cloned().unwrap_or_else(|| json!({
            "prompt_tokens": 0,
            "completion_tokens": 0,
            "total_tokens": 0
        }))
    })
}

fn bridge_compat_error(
    request_id: &str,
    session_id: Option<&str>,
    code: &str,
    message: &str,
) -> serde_json::Value {
    json!({
        "ok": false,
        "request_id": request_id,
        "session_id": session_id.unwrap_or(""),
        "text": "",
        "error_code": code,
        "error_message": message,
        "usage": {
            "prompt_tokens": 0,
            "completion_tokens": 0,
            "total_tokens": 0
        }
    })
}

async fn write_invalid_request(
    tx: &mpsc::UnboundedSender<serde_json::Value>,
    request_id: &str,
    session_id: Option<&str>,
    message: &str,
    response_mode: HostResponseMode,
) -> anyhow::Result<()> {
    if response_mode == HostResponseMode::BridgeCompat {
        let payload = bridge_compat_error(request_id, session_id, "INVALID_REQUEST", message);
        return send_protocol_line(tx, payload).await;
    }

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
    response_mode: HostResponseMode,
) -> anyhow::Result<()> {
    if response_mode == HostResponseMode::BridgeCompat {
        let payload = bridge_compat_error(
            request_id,
            session_id,
            "BUSY",
            "server is overloaded; retry later",
        );
        return send_protocol_line(tx, payload).await;
    }

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
    emit_ack: bool,
    allow_orphan_terminal: bool,
    response_mode: HostResponseMode,
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
                response_mode,
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
                response_mode,
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
                response_mode,
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
                response_mode,
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
            return write_busy_response(&tx, &request_id, Some(&session_id), response_mode).await;
        }
    };

    let request_ts = req.ts_ms.unwrap_or_else(now_ts_ms);
    let timeout_ms = req.timeout_ms.unwrap_or(120_000);
    let stream = req.stream.unwrap_or(false);

    if emit_ack {
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
    }

    let _ = stream;
    let _ = req.metadata;

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
            if allow_orphan_terminal {
                if let Some(run) = active.get(&task_request_id) {
                    if run.run_token == task_run_token {
                        active.remove(&task_request_id);
                    }
                }
                true
            } else {
                match active.get(&task_request_id) {
                    Some(run) if run.run_token == task_run_token => {
                        active.remove(&task_request_id);
                        true
                    }
                    _ => false,
                }
            }
        };

        if !should_emit_terminal {
            return;
        }

        {
            let mut inflight = task_inflight.lock().await;
            inflight.remove(&task_replay_key);
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

        let outbound = if response_mode == HostResponseMode::BridgeCompat {
            bridge_compat_from_protocol(&final_payload)
        } else {
            final_payload
        };
        let _ = send_protocol_line(&task_tx, outbound).await;
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

fn read_stdin_timeout_ms(cli_timeout_ms: Option<u64>) -> u64 {
    cli_timeout_ms
        .or_else(|| {
            std::env::var("ZENE_STDIN_TIMEOUT_MS")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
        })
        .unwrap_or(15_000)
}

async fn run_host(
    engine: Arc<ZeneEngine>,
    protocol: &str,
    single_request: bool,
    stdin_timeout_ms: Option<u64>,
    response_mode: HostResponseMode,
) -> anyhow::Result<()> {
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
    let input_timeout_ms = read_stdin_timeout_ms(stdin_timeout_ms);
    let mut handled_any_request = false;

    loop {
        let next_line = timeout(Duration::from_millis(input_timeout_ms), lines.next_line()).await;
        let line = match next_line {
            Ok(Ok(Some(line))) => line,
            Ok(Ok(None)) => {
                if handled_any_request {
                    break;
                }
                anyhow::bail!("PROTOCOL: stdin EOF before request");
            }
            Ok(Err(e)) => anyhow::bail!("PROTOCOL: stdin read error: {}", e),
            Err(_) => anyhow::bail!("PROTOCOL: stdin read timeout after {}ms", input_timeout_ms),
        };

        if line.trim().is_empty() {
            if single_request {
                anyhow::bail!("PROTOCOL: empty input line");
            }
            continue;
        }

        handled_any_request = true;

        prune_idempotency_cache(&idempotency_store).await;

        let req: HostRequest = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                write_invalid_request(
                    &out_tx,
                    "unknown",
                    None,
                    &format!("invalid JSON payload: {}", e),
                    response_mode,
                )
                .await?;
                anyhow::bail!("PROTOCOL: invalid JSON payload: {}", e);
            }
        };

        if req.protocol_version != 1 {
            write_invalid_request(
                &out_tx,
                &req.request_id,
                req.session_id.as_deref(),
                "unsupported protocol_version (expected 1)",
                response_mode,
            )
            .await?;
            anyhow::bail!("PROTOCOL: unsupported protocol_version");
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
                        response_mode,
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
                            response_mode,
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
                        response_mode,
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
                    !single_request,
                    single_request,
                    response_mode,
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
                    response_mode,
                )
                .await?;
                if single_request {
                    anyhow::bail!("PROTOCOL: unsupported request type");
                }
            }
        }

        if single_request {
            break;
        }
    }

    {
        let mut active = active_runs.lock().await;
        let handles: Vec<_> = active.drain().map(|(_, run)| run.handle).collect();
        drop(active);
        for handle in handles {
            if let Err(e) = handle.await {
                error!("Host run task join error: {}", e);
            }
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

    let cli = Cli::parse();

    if let Some(Commands::SelfTest) = &cli.command {
        let report = run_self_test(&config);
        println!("{}", serde_json::to_string(&report)?);
        let ok = report.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
        if !ok {
            std::process::exit(EXIT_CONFIG_ERROR);
        }
        std::process::exit(EXIT_OK);
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let storage_dir = PathBuf::from(&home).join(".zene/sessions");
    let store = match FileSessionStore::new(storage_dir) {
        Ok(s) => Arc::new(s),
        Err(e) => {
            error!("Failed to initialize session store: {}", e);
            std::process::exit(EXIT_RUNTIME_ERROR);
        }
    };
    let engine = match ZeneEngine::new(config, store).await {
        Ok(e) => Arc::new(e),
        Err(e) => {
            error!("Failed to initialize engine: {}", e);
            std::process::exit(EXIT_RUNTIME_ERROR);
        }
    };

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
                    Err(e) => {
                        error!("Task failed: {}", e);
                        std::process::exit(EXIT_RUNTIME_ERROR);
                    }
                }
            }
            Commands::Worker => {
                if let Err(e) = Worker::run(engine.as_ref()).await {
                    error!("Worker failed: {}", e);
                    std::process::exit(EXIT_RUNTIME_ERROR);
                }
            }
            Commands::SelfTest => unreachable!("self-test handled before engine init"),
            Commands::Host {
                protocol,
                single_request,
                stdin_timeout_ms,
                bridge_compat,
            } => {
                info!("Starting host mode with protocol={}", protocol);
                let response_mode = if bridge_compat {
                    HostResponseMode::BridgeCompat
                } else {
                    HostResponseMode::Protocol
                };
                if let Err(e) =
                    run_host(engine, &protocol, single_request, stdin_timeout_ms, response_mode)
                        .await
                {
                    let message = e.to_string();
                    error!("Host failed: {}", message);
                    if message.starts_with("PROTOCOL:") {
                        std::process::exit(EXIT_PROTOCOL_ERROR);
                    }
                    std::process::exit(EXIT_RUNTIME_ERROR);
                }
            }
        }
    } else {
        use clap::CommandFactory;
        Cli::command().print_help()?;
        std::process::exit(EXIT_OK);
    }

    Ok(())
}
