use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

struct HostHarness {
    child: Child,
    stdin: ChildStdin,
    rx: Receiver<Value>,
}

impl HostHarness {
    fn send_line(&mut self, line: &str) {
        writeln!(self.stdin, "{}", line).expect("failed to write request");
        self.stdin.flush().expect("failed to flush request");
    }

    fn recv_json(&self, timeout_ms: u64) -> Value {
        self.rx
            .recv_timeout(Duration::from_millis(timeout_ms))
            .expect("timed out waiting for host protocol line")
    }

    fn collect_for(&self, wait_ms: u64) -> Vec<Value> {
        let deadline = Instant::now() + Duration::from_millis(wait_ms);
        let mut out = Vec::new();
        loop {
            let now = Instant::now();
            if now >= deadline {
                break;
            }
            let remain = deadline.saturating_duration_since(now);
            match self.rx.recv_timeout(remain.min(Duration::from_millis(30))) {
                Ok(msg) => out.push(msg),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        out
    }

    fn shutdown(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn start_host_with_max_concurrency(max_concurrency: Option<usize>) -> HostHarness {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_zene"));
    if let Some(limit) = max_concurrency {
        cmd.env("ZENE_MAX_CONCURRENCY", limit.to_string());
    }

    let mut child = cmd
        .arg("host")
        .arg("--protocol")
        .arg("v1")
        .arg("--single-request")
        .arg("false")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn zene host");

    let stdin = child.stdin.take().expect("missing stdin");
    let stdout = child.stdout.take().expect("missing stdout");

    let (tx, rx) = mpsc::channel::<Value>();
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    if let Ok(msg) = serde_json::from_str::<Value>(line.trim()) {
                        if tx.send(msg).is_err() {
                            break;
                        }
                    }
                }
                Err(_) => break,
            }
        }
    });

    HostHarness { child, stdin, rx }
}

fn start_host() -> HostHarness {
    start_host_with_max_concurrency(None)
}

fn count_final_for(messages: &[Value], request_id: &str) -> usize {
    messages
        .iter()
        .filter(|m| m["type"] == "final" && m["request_id"] == request_id)
        .count()
}

#[test]
fn test_host_ping_ack() {
    let mut host = start_host();

    host.send_line(
        "{\"protocol_version\":1,\"type\":\"ping\",\"request_id\":\"p_test\",\"session_id\":\"s_test\"}",
    );

    let msg = host.recv_json(1_000);

    assert_eq!(msg["type"], "ack");
    assert_eq!(msg["status"], "PONG");
    assert_eq!(msg["request_id"], "p_test");

    host.shutdown();
}

#[test]
fn test_host_cancel_requires_target() {
    let mut host = start_host();

    host.send_line(
        "{\"protocol_version\":1,\"type\":\"cancel\",\"request_id\":\"c_test\",\"session_id\":\"s_test\"}",
    );

    let msg = host.recv_json(1_000);

    assert_eq!(msg["type"], "error");
    assert_eq!(msg["error"]["code"], "INVALID_REQUEST");
    assert_eq!(msg["request_id"], "c_test");

    host.shutdown();
}

#[test]
fn test_host_inflight_idempotency_dedup() {
    let mut host = start_host();

    host.send_line(
        "{\"protocol_version\":1,\"type\":\"run\",\"request_id\":\"r_a\",\"session_id\":\"s_test\",\"prompt\":\"hello\",\"timeout_ms\":500,\"idempotency_key\":\"k_test\",\"stream\":false}"
    );
    host.send_line(
        "{\"protocol_version\":1,\"type\":\"run\",\"request_id\":\"r_b\",\"session_id\":\"s_test\",\"prompt\":\"hello\",\"timeout_ms\":500,\"idempotency_key\":\"k_test\",\"stream\":false}"
    );

    let msg1 = host.recv_json(1_500);
    let msg2 = host.recv_json(1_500);

    assert_eq!(msg1["type"], "ack");
    assert_eq!(msg1["status"], "ACCEPTED");

    assert_eq!(msg2["type"], "ack");
    assert_eq!(msg2["status"], "DUPLICATE_IN_PROGRESS");
    assert_eq!(msg2["existing_request_id"], "r_a");

    host.shutdown();
}

#[test]
fn test_host_cancel_emits_single_terminal_final() {
    let mut host = start_host();

    host.send_line(
        "{\"protocol_version\":1,\"type\":\"run\",\"request_id\":\"r_cancel\",\"session_id\":\"s_cancel\",\"prompt\":\"long running cancel test\",\"timeout_ms\":120000,\"idempotency_key\":\"k_cancel\",\"stream\":false}",
    );

    let ack = host.recv_json(2_000);
    assert_eq!(ack["type"], "ack");
    assert_eq!(ack["status"], "ACCEPTED");

    host.send_line(
        "{\"protocol_version\":1,\"type\":\"cancel\",\"request_id\":\"c_cancel\",\"session_id\":\"s_cancel\",\"target_request_id\":\"r_cancel\"}",
    );

    let mut observed = vec![host.recv_json(2_000), host.recv_json(2_000)];
    observed.extend(host.collect_for(500));

    let maybe_cancel_ack = observed
        .iter()
        .find(|m| m["type"] == "ack" && m["request_id"] == "c_cancel");

    let final_count = count_final_for(&observed, "r_cancel");
    assert!(final_count <= 1, "expected at most one final, got {}", final_count);

    let target_final = observed
        .iter()
        .find(|m| m["type"] == "final" && m["request_id"] == "r_cancel")
        .expect("missing target final");

    if let Some(cancel_ack) = maybe_cancel_ack {
        assert_eq!(cancel_ack["status"], "CANCEL_ACCEPTED");
        assert_eq!(target_final["status"], "CANCELED");
    } else {
        let cancel_error = observed
            .iter()
            .find(|m| m["type"] == "error" && m["request_id"] == "c_cancel")
            .expect("missing cancel response");
        assert_eq!(cancel_error["error"]["code"], "INVALID_REQUEST");
    }

    host.shutdown();
}

#[test]
fn test_host_timeout_emits_single_terminal_final() {
    let mut host = start_host();

    host.send_line(
        "{\"protocol_version\":1,\"type\":\"run\",\"request_id\":\"r_timeout\",\"session_id\":\"s_timeout\",\"prompt\":\"force timeout\",\"timeout_ms\":1,\"idempotency_key\":\"k_timeout\",\"stream\":false}",
    );

    let mut observed = Vec::new();
    observed.push(host.recv_json(2_000));

    loop {
        let msg = host.recv_json(3_000);
        let is_target_final = msg["type"] == "final" && msg["request_id"] == "r_timeout";
        observed.push(msg);
        if is_target_final {
            break;
        }
    }

    observed.extend(host.collect_for(500));

    let timeout_final = observed
        .iter()
        .find(|m| m["type"] == "final" && m["request_id"] == "r_timeout")
        .expect("missing timeout final");
    assert_eq!(timeout_final["status"], "TIMEOUT");

    assert_eq!(count_final_for(&observed, "r_timeout"), 1);
    host.shutdown();
}

#[test]
fn test_host_returns_busy_when_global_limit_reached() {
    let mut host = start_host_with_max_concurrency(Some(1));

    host.send_line(
        "{\"protocol_version\":1,\"type\":\"run\",\"request_id\":\"r_busy_a\",\"session_id\":\"s_busy\",\"prompt\":\"hold capacity\",\"timeout_ms\":120000,\"idempotency_key\":\"k_busy_a\",\"stream\":false}",
    );
    host.send_line(
        "{\"protocol_version\":1,\"type\":\"run\",\"request_id\":\"r_busy_b\",\"session_id\":\"s_busy\",\"prompt\":\"should be rejected\",\"timeout_ms\":120000,\"idempotency_key\":\"k_busy_b\",\"stream\":false}",
    );

    let mut observed = vec![host.recv_json(2_000), host.recv_json(2_000)];
    observed.extend(host.collect_for(500));

    let busy_error = observed
        .iter()
        .find(|m| m["type"] == "error" && m["request_id"] == "r_busy_b")
        .expect("missing busy rejection for second request");
    assert_eq!(busy_error["error"]["code"], "BUSY");
    assert_eq!(busy_error["error"]["http_status"], 429);

    host.shutdown();
}

#[test]
fn test_host_single_request_mode_emits_one_json_line() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_zene"))
        .arg("host")
        .arg("--protocol")
        .arg("v1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn single-request host");

    {
        let mut stdin = child.stdin.take().expect("missing stdin");
        writeln!(
            stdin,
            "{{\"protocol_version\":1,\"type\":\"run\",\"request_id\":\"r_single\",\"session_id\":\"s_single\",\"prompt\":\"single response mode\",\"timeout_ms\":2000,\"idempotency_key\":\"k_single\"}}"
        )
        .expect("failed to write single request");
        stdin.flush().expect("failed to flush single request");
    }

    let output = child
        .wait_with_output()
        .expect("failed waiting for single-request host output");
    assert!(output.status.success(), "single-request host exited with non-zero status");

    let stdout_text = String::from_utf8(output.stdout).expect("stdout should be valid utf8");
    let lines: Vec<&str> = stdout_text.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(lines.len(), 1, "expected exactly one stdout JSON line");

    let msg: Value = serde_json::from_str(lines[0]).expect("stdout line must be valid json");
    assert_eq!(msg["type"], "final");
}
