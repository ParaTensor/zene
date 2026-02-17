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
    
    print(f"Working Directory: {work_dir}")

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
    Create a RESTful API for a Todo List application using FastAPI and SQLite.
    The structure should be modular:
    1. `models.py`: SQLAlchemy models for the `Todo` table (id, title, description, completed).
    2. `schemas.py`: Pydantic schemas for request/response validation (TodoCreate, TodoResponse).
    3. `database.py`: Database connection and session management.
    4. `crud.py`: Functions to create, read, update, and delete todos.
    5. `main.py`: The FastAPI application with endpoints for CRUD operations.
    
    Finally, create a test script `test_api.py` using `requests` to verify that we can create and list todos.
    """

    print("Starting Zene Server for Multi-File API Task...")
    print(f"Session ID: {session.session_id}")
    print("\n--- Sending API Development Task ---\n")
    print(task)
    print("\n---\nWaiting for Agent execution...\n")

    try:
        result = await runner.run(task)
        print("\n=== Agent Execution Result ===")
        print(json.dumps(result, indent=2))
        
        # Verify output
        if os.path.exists(os.path.join(work_dir, "main.py")) and \
           os.path.exists(os.path.join(work_dir, "models.py")):
            print("\n✅ API Project Structure Created Successfully!")
            print("Files generated:")
            for file in os.listdir(work_dir):
                if file.endswith(".py"):
                    print(f"  - {file}")
        else:
            print("\n❌ API Project generation incomplete.")
            
    except Exception as e:
        print(f"\n❌ Error: {e}")

if __name__ == "__main__":
    asyncio.run(main())
