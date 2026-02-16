import subprocess
import json
import time
import os

# Ensure Zene is built
# cargo build --release

ZENE_BIN = "./target/release/zene"
if not os.path.exists(ZENE_BIN):
    # Fallback to debug build if release not found
    ZENE_BIN = "./target/debug/zene"
    if not os.path.exists(ZENE_BIN):
        print("Error: Zene binary not found. Please run 'cargo build' first.")
        exit(1)

def send_request(proc, method, params, req_id):
    req = {
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": req_id
    }
    json_str = json.dumps(req)
    print(f"-> Sending: {json_str}")
    proc.stdin.write(json_str + "\n")
    proc.stdin.flush()

def read_response(proc):
    line = proc.stdout.readline()
    if not line:
        return None
    print(f"<- Received: {line.strip()}")
    return json.loads(line)

def main():
    # Start Zene in Server Mode
    # Note: We need to set RUST_LOG=info to see logs in stderr, but JSON-RPC is on stdout
    env = os.environ.copy()
    env["RUST_LOG"] = "info"
    
    print("Starting Zene Server...")
    proc = subprocess.Popen(
        [ZENE_BIN, "server"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE, # Capture logs separately
        text=True,
        env=env
    )

    try:
        # 1. Start a session and ask a question
        session_id = "demo_session_001"
        
        print("\n--- Step 1: Initialize Context ---")
        send_request(proc, "agent.run", {
            "instruction": "What is the capital of France?",
            "session_id": session_id
        }, 1)
        
        resp1 = read_response(proc)
        if resp1 and "result" in resp1:
            print(f"Agent Answer: {resp1['result']['message']}")

        # 2. Ask a follow-up question using the SAME session_id
        print("\n--- Step 2: Follow-up Question (Memory Test) ---")
        send_request(proc, "agent.run", {
            "instruction": "What is its population?",
            "session_id": session_id
        }, 2)

        resp2 = read_response(proc)
        if resp2 and "result" in resp2:
            print(f"Agent Answer: {resp2['result']['message']}")
            
        print("\n--- Session Persistence Check ---")
        home = os.path.expanduser("~")
        session_file = os.path.join(home, ".zene", "sessions", f"{session_id}.json")
        if os.path.exists(session_file):
            print(f"Success! Session file created at: {session_file}")
            with open(session_file, 'r') as f:
                data = json.load(f)
                print(f"History length: {len(data['history'])} messages")
        else:
            print("Warning: Session file not found.")

    finally:
        proc.terminate()
        # Print any stderr logs
        stderr_output = proc.stderr.read()
        if stderr_output:
            print("\n[Server Logs]:")
            print(stderr_output)

if __name__ == "__main__":
    main()
