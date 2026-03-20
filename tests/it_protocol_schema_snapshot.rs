use serde_json::Value;

fn parse(line: &str) -> Value {
    serde_json::from_str(line).expect("snapshot json must be valid")
}

fn assert_bridge_response_shape(v: &Value) {
    assert!(v.is_object(), "response must be an object");
    assert!(v.get("ok").map(|x| x.is_boolean()).unwrap_or(false));
    assert!(v.get("request_id").map(|x| x.is_string()).unwrap_or(false));
    assert!(v.get("session_id").map(|x| x.is_string()).unwrap_or(false));
    assert!(v.get("text").map(|x| x.is_string()).unwrap_or(false));
    assert!(
        v.get("error_code")
            .map(|x| x.is_null() || x.is_string())
            .unwrap_or(false)
    );
    assert!(
        v.get("error_message")
            .map(|x| x.is_null() || x.is_string())
            .unwrap_or(false)
    );

    let usage = v.get("usage").expect("usage field is required");
    assert!(usage.is_object(), "usage must be object");
    assert!(
        usage
            .get("prompt_tokens")
            .map(|x| x.is_number())
            .unwrap_or(false)
    );
    assert!(
        usage
            .get("completion_tokens")
            .map(|x| x.is_number())
            .unwrap_or(false)
    );
    assert!(
        usage
            .get("total_tokens")
            .map(|x| x.is_number())
            .unwrap_or(false)
    );
}

#[test]
fn snapshot_success_bridge_payload_schema_v1() {
    let req = parse(
        r#"{"protocol_version":1,"type":"run","request_id":"req_success_001","session_id":"qq_user_42","prompt":"Summarize this issue.","timeout_ms":30000,"idempotency_key":"idem_success_001"}"#,
    );
    assert_eq!(req["protocol_version"], 1);
    assert_eq!(req["type"], "run");
    assert!(req.get("request_id").map(|x| x.is_string()).unwrap_or(false));
    assert!(req.get("session_id").map(|x| x.is_string()).unwrap_or(false));
    assert!(req.get("prompt").map(|x| x.is_string()).unwrap_or(false));
    assert!(req.get("timeout_ms").map(|x| x.is_number()).unwrap_or(false));
    assert!(
        req.get("idempotency_key")
            .map(|x| x.is_string())
            .unwrap_or(false)
    );

    let resp = parse(
        r#"{"ok":true,"request_id":"req_success_001","session_id":"qq_user_42","text":"All systems normal.","error_code":null,"error_message":null,"usage":{"prompt_tokens":12,"completion_tokens":8,"total_tokens":20}}"#,
    );
    assert_bridge_response_shape(&resp);
    assert_eq!(resp["ok"], true);
    assert!(resp["error_code"].is_null());
    assert!(resp["error_message"].is_null());
}

#[test]
fn snapshot_timeout_bridge_payload_schema_v1() {
    let resp = parse(
        r#"{"ok":false,"request_id":"req_timeout_001","session_id":"qq_user_42","text":"","error_code":"TIMEOUT","error_message":"request exceeded timeout_ms","usage":{"prompt_tokens":0,"completion_tokens":0,"total_tokens":0}}"#,
    );
    assert_bridge_response_shape(&resp);
    assert_eq!(resp["ok"], false);
    assert_eq!(resp["error_code"], "TIMEOUT");
}

#[test]
fn snapshot_provider_auth_bridge_payload_schema_v1() {
    let resp = parse(
        r#"{"ok":false,"request_id":"req_auth_001","session_id":"qq_user_42","text":"","error_code":"PROVIDER_AUTH","error_message":"provider authentication failed","usage":{"prompt_tokens":0,"completion_tokens":0,"total_tokens":0}}"#,
    );
    assert_bridge_response_shape(&resp);
    assert_eq!(resp["ok"], false);
    assert_eq!(resp["error_code"], "PROVIDER_AUTH");
}

#[test]
fn snapshot_invalid_request_protocol_payload_schema_v1() {
    let req = parse(
        r#"{"protocol_version":1,"type":"run","request_id":"req_bad_001","session_id":"qq_user_42","prompt":"hello"}"#,
    );
    assert_eq!(req["protocol_version"], 1);
    assert_eq!(req["type"], "run");
    assert!(
        req.get("idempotency_key").is_none(),
        "invalid-request baseline should intentionally miss idempotency_key"
    );

    let protocol_error_resp = parse(
        r#"{"protocol_version":1,"type":"error","request_id":"req_bad_001","session_id":"qq_user_42","error":{"code":"INVALID_REQUEST","message":"run request requires non-empty idempotency_key","retryable":false}}"#,
    );
    assert_eq!(protocol_error_resp["protocol_version"], 1);
    assert_eq!(protocol_error_resp["type"], "error");
    assert_eq!(protocol_error_resp["error"]["code"], "INVALID_REQUEST");
    assert!(protocol_error_resp["error"]["message"].is_string());
    assert!(protocol_error_resp["error"]["retryable"].is_boolean());
}
