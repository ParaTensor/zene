use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

fn start_host() -> (Child, ChildStdin, BufReader<ChildStdout>) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_zene"))
        .arg("host")
        .arg("--protocol")
        .arg("v1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn zene host");

    let stdin = child.stdin.take().expect("missing stdin");
    let stdout = child.stdout.take().expect("missing stdout");
    (child, stdin, BufReader::new(stdout))
}

fn read_json_line(reader: &mut BufReader<ChildStdout>) -> Value {
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .expect("failed to read protocol line");
    serde_json::from_str::<Value>(line.trim()).expect("invalid JSON output")
}

#[test]
fn test_host_ping_ack() {
    let (mut child, mut stdin, mut stdout) = start_host();

    writeln!(
        stdin,
        "{{\"protocol_version\":1,\"type\":\"ping\",\"request_id\":\"p_test\",\"session_id\":\"s_test\"}}"
    )
    .expect("failed to write request");
    stdin.flush().expect("failed to flush request");

    let msg = read_json_line(&mut stdout);

    assert_eq!(msg["type"], "ack");
    assert_eq!(msg["status"], "PONG");
    assert_eq!(msg["request_id"], "p_test");

    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn test_host_cancel_requires_target() {
    let (mut child, mut stdin, mut stdout) = start_host();

    writeln!(
        stdin,
        "{{\"protocol_version\":1,\"type\":\"cancel\",\"request_id\":\"c_test\",\"session_id\":\"s_test\"}}"
    )
    .expect("failed to write request");
    stdin.flush().expect("failed to flush request");

    let msg = read_json_line(&mut stdout);

    assert_eq!(msg["type"], "error");
    assert_eq!(msg["error"]["code"], "INVALID_REQUEST");
    assert_eq!(msg["request_id"], "c_test");

    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn test_host_inflight_idempotency_dedup() {
    let (mut child, mut stdin, mut stdout) = start_host();

    writeln!(
        stdin,
        "{{\"protocol_version\":1,\"type\":\"run\",\"request_id\":\"r_a\",\"session_id\":\"s_test\",\"prompt\":\"hello\",\"timeout_ms\":500,\"idempotency_key\":\"k_test\",\"stream\":false}}"
    )
    .expect("failed to write first run request");
    writeln!(
        stdin,
        "{{\"protocol_version\":1,\"type\":\"run\",\"request_id\":\"r_b\",\"session_id\":\"s_test\",\"prompt\":\"hello\",\"timeout_ms\":500,\"idempotency_key\":\"k_test\",\"stream\":false}}"
    )
    .expect("failed to write second run request");
    stdin.flush().expect("failed to flush run requests");

    let msg1 = read_json_line(&mut stdout);
    let msg2 = read_json_line(&mut stdout);

    assert_eq!(msg1["type"], "ack");
    assert_eq!(msg1["status"], "ACCEPTED");

    assert_eq!(msg2["type"], "ack");
    assert_eq!(msg2["status"], "DUPLICATE_IN_PROGRESS");
    assert_eq!(msg2["existing_request_id"], "r_a");

    let _ = child.kill();
    let _ = child.wait();
}
