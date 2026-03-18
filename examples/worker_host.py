import subprocess
import json
import sys
import os

def run_worker(instruction, strategy="Direct"):
    # 确保二进制文件存在
    binary_path = "./target/debug/zene"
    if not os.path.exists(binary_path):
        print(f"Error: Binary not found at {binary_path}. Please run 'cargo build' first.")
        return

    # 准备请求 JSON
    request = {
        "request_id": "req_123",
        "instruction": instruction,
        "strategy": strategy,
        "context": {
            "env": {
                "LLM_PROVIDER": os.getenv("LLM_PROVIDER", "openai"),
                "OPENAI_API_KEY": os.getenv("OPENAI_API_KEY", ""),
                # 如果使用其他 Provider，请在这里添加对应的 Key
            }
        }
    }

    # 启动 worker 进程
    process = subprocess.Popen(
        [binary_path, "worker"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1
    )

    # 发送请求
    process.stdin.write(json.dumps(request) + "\n")
    process.stdin.close()

    print(f"--- Starting Zene Worker [Strategy: {strategy}] ---")
    
    # 实时解析输出
    for line in process.stdout:
        try:
            msg = json.loads(line)
            msg_type = msg.get("type")
            
            if msg_type == "Snapshot":
                status = msg.get("snapshot", {}).get("status")
                print(f"[STATUS] {status}")
            elif msg_type == "Event":
                content = msg.get("event", {})
                # 打印执行中的关键事件
                if "TaskStarted" in str(content):
                    print(f"[PROCESS] Starting task...")
                elif "ToolCalled" in str(content):
                    tool = content.get("ToolCalled", {}).get("name")
                    print(f"[TOOL] Calling: {tool}")
            elif msg_type == "Log":
                print(f"  > {msg.get('message')}")
                
        except json.JSONDecodeError:
            print(f"RAW: {line.strip()}")

    # 检查错误输出
    stderr_output = process.stderr.read()
    if stderr_output:
        print(f"\n--- Stderr ---\n{stderr_output}")

    process.wait()
    print(f"--- Worker Finished with exit code {process.returncode} ---")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python3 examples/worker_host.py 'Your instruction'")
    else:
        run_worker(sys.argv[1])
