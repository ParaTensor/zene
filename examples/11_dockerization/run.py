import os
import sys
import asyncio
import json

# Add the project root to sys.path so we can import zene
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "../..")))

from zene.engine.session import Session
from zene.config import AgentConfig, RoleConfig
from zene.agent.runner import AgentRunner
from zene.agent.planner import Planner
from zene.agent.executor import Executor
from zene.agent.reflector import Reflector

async def main():
    # 1. Setup the environment
    work_dir = os.path.join(os.path.dirname(__file__), "workspace")
    os.makedirs(work_dir, exist_ok=True)
    
    # Create a simple Flask app to dockerize
    with open(os.path.join(work_dir, "app.py"), "w") as f:
        f.write("""
from flask import Flask
import os

app = Flask(__name__)

@app.route('/')
def hello():
    return f"Hello from Docker! Host: {os.uname().nodename}"

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
""")
    
    with open(os.path.join(work_dir, "requirements.txt"), "w") as f:
        f.write("Flask==2.0.1\nWerkzeug==2.0.1\n")

    print(f"Created Flask app at: {work_dir}")

    # 2. Initialize Zene components
    config = AgentConfig(
        planner=RoleConfig(provider="deepseek", model="deepseek-chat", api_key=os.environ.get("ZENE_PLANNER_API_KEY")),
        executor=RoleConfig(provider="zhipu", model="glm-4-flash", api_key=os.environ.get("ZENE_EXECUTOR_API_KEY")),
        reflector=RoleConfig(provider="minimax", model="MiniMax-M2.5", api_key=os.environ.get("ZENE_REFLECTOR_API_KEY"))
    )

    session = Session(work_dir=work_dir)
    planner = Planner(config.planner)
    executor = Executor(config.executor, work_dir=work_dir)
    reflector = Reflector(config.reflector)
    
    runner = AgentRunner(session, planner, executor, reflector)

    # 3. Run the task
    task = """
    I have a Python Flask application in `app.py` with `requirements.txt`.
    Please help me containerize it:
    1. Create a `Dockerfile` that uses python:3.9-slim, installs dependencies, exposes port 5000, and runs the app.
    2. Create a `docker-compose.yml` file to run the service, mapping port 5000:5000.
    3. Add a `.dockerignore` file to exclude unnecessary files (like __pycache__, .git, venv).
    """

    print("Starting Zene Server for Dockerization Task...")
    print(f"Session ID: {session.session_id}")
    print("\n--- Sending Dockerization Task ---\n")
    print(task)
    print("\n---\nWaiting for Agent execution...\n")

    try:
        result = await runner.run(task)
        print("\n=== Agent Execution Result ===")
        print(json.dumps(result, indent=2))
        
        # Verify output
        if os.path.exists(os.path.join(work_dir, "Dockerfile")) and \
           os.path.exists(os.path.join(work_dir, "docker-compose.yml")):
            print("\n✅ Docker Configuration Created Successfully!")
        else:
            print("\n❌ Docker files missing.")
            
    except Exception as e:
        print(f"\n❌ Error: {e}")

if __name__ == "__main__":
    asyncio.run(main())
