import subprocess
import time
import json
import os
import sys
import shutil

# Configuration
ZENE_BIN = os.path.abspath("./target/release/zene")
WORKSPACE_DIR = os.path.abspath("./temp_workspace/refactoring_legacy")

# Sample Messy Python Script (The "Legacy Code")
LEGACY_CODE = """
def process_data(data):
    # This function processes data in a very messy way
    # No type hints, no docstrings, mixed logic
    result = []
    for item in data:
        if item['age'] > 18:
            if item['status'] == 'active':
                full_name = item['first_name'] + " " + item['last_name']
                email = item['email'].lower()
                # Simulate heavy processing
                score = item['score'] * 1.5
                result.append({'name': full_name, 'email': email, 'score': score})
    
    # Calculate average score
    total = 0
    for r in result:
        total += r['score']
    
    avg = total / len(result) if len(result) > 0 else 0
    
    print("Processed " + str(len(result)) + " items.")
    print("Average score: " + str(avg))
    
    return result

def main():
    # Sample data
    users = [
        {'first_name': 'John', 'last_name': 'Doe', 'age': 25, 'status': 'active', 'email': 'JOHN@EXAMPLE.COM', 'score': 80},
        {'first_name': 'Jane', 'last_name': 'Smith', 'age': 17, 'status': 'active', 'email': 'jane@example.com', 'score': 90},
        {'first_name': 'Bob', 'last_name': 'Brown', 'age': 30, 'status': 'inactive', 'email': 'bob@example.com', 'score': 70},
        {'first_name': 'Alice', 'last_name': 'White', 'age': 22, 'status': 'active', 'email': 'alice@example.com', 'score': 85}
    ]
    
    output = process_data(users)
    
    # Save to file (hardcoded path, bad practice)
    with open('output.txt', 'w') as f:
        f.write(str(output))

if __name__ == "__main__":
    main()
"""

def setup_workspace():
    if os.path.exists(WORKSPACE_DIR):
        print(f"Cleaning existing workspace: {WORKSPACE_DIR}")
        shutil.rmtree(WORKSPACE_DIR)
    os.makedirs(WORKSPACE_DIR)
    
    # Write the legacy script
    with open(os.path.join(WORKSPACE_DIR, "legacy_script.py"), "w") as f:
        f.write(LEGACY_CODE)
    
    print(f"Created messy script at: {os.path.join(WORKSPACE_DIR, 'legacy_script.py')}")

def main():
    setup_workspace()

    env = os.environ.copy()
    env["RUST_LOG"] = "info"
    
    print("Starting Zene Server for Refactoring Task...")
    print(f"Working Directory: {WORKSPACE_DIR}")

    proc = subprocess.Popen(
        [ZENE_BIN, "server"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=sys.stderr, 
        text=True,
        env=env,
        cwd=WORKSPACE_DIR
    )

    try:
        session_id = f"demo_refactor_{int(time.time())}"
        print(f"Session ID: {session_id}")
        
        # Task: Refactor Legacy Code
        instruction = """
I have a messy Python script `legacy_script.py`. Please refactor it to follow modern Python best practices.

Requirements:
1. Analyze the existing code to understand its logic.
2. Create a new module `src/processor.py` for the data processing logic.
3. Create a new module `src/models.py` using Pydantic or dataclasses for type safety.
4. Rewrite the logic to be clean, with type hints and docstrings.
5. Create a `main.py` entry point that uses these modules.
6. Verify the refactored code by running it and ensuring it produces the same output as the original.
        """
        
        request = {
            "jsonrpc": "2.0",
            "method": "agent.run",
            "params": {
                "instruction": instruction.strip(),
                "session_id": session_id
            },
            "id": 1
        }

        print(f"\n--- Sending Refactoring Task ---\n{instruction}\n---")
        
        json_req = json.dumps(request)
        proc.stdin.write(json_req + "\n")
        proc.stdin.flush()

        print("Waiting for Agent execution...")
        
        while True:
            line = proc.stdout.readline()
            if not line:
                break
            
            try:
                resp = json.loads(line)
                if "result" in resp:
                    print("\n=== Agent Execution Result ===")
                    print(json.dumps(resp["result"], indent=2))
                    
                    if resp["result"]["status"] == "completed":
                        print("\n✅ Refactoring Completed!")
                        print(f"Check the new code in: {WORKSPACE_DIR}")
                    else:
                        print("\n❌ Task Failed.")
                    break
                elif "error" in resp:
                    print(f"Error: {resp['error']}")
                    break
            except json.JSONDecodeError:
                print(f"[Server Log] {line.strip()}")

    except KeyboardInterrupt:
        print("\nStopping...")
    finally:
        proc.terminate()

if __name__ == "__main__":
    main()
