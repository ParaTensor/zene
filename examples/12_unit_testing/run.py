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
    
    # Create a Python module to test
    with open(os.path.join(work_dir, "math_utils.py"), "w") as f:
        f.write("""
def calculate_factorial(n):
    if n < 0:
        raise ValueError("Factorial is not defined for negative numbers")
    if n == 0 or n == 1:
        return 1
    return n * calculate_factorial(n - 1)

def is_prime(n):
    if n <= 1:
        return False
    for i in range(2, int(n**0.5) + 1):
        if n % i == 0:
            return False
    return True

def safe_divide(a, b):
    if b == 0:
        return None
    return a / b
""")
    
    print(f"Created math_utils.py at: {work_dir}")

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
    I have a python module `math_utils.py` containing some functions.
    Please create a comprehensive test suite `test_math_utils.py` using `pytest`.
    
    Requirements:
    1. Test `calculate_factorial` with positive integers, 0, and 1.
    2. Test `calculate_factorial` raises ValueError for negative inputs.
    3. Test `is_prime` with prime numbers, composite numbers, 1, 0, and negative numbers.
    4. Test `safe_divide` with normal division and division by zero.
    5. Use parametrized tests where appropriate to cover multiple cases.
    6. Run the tests using `pytest` and confirm they pass.
    """

    print("Starting Zene Server for Unit Testing Task...")
    print(f"Session ID: {session.session_id}")
    print("\n--- Sending Testing Task ---\n")
    print(task)
    print("\n---\nWaiting for Agent execution...\n")

    try:
        result = await runner.run(task)
        print("\n=== Agent Execution Result ===")
        print(json.dumps(result, indent=2))
        
        # Verify output
        if os.path.exists(os.path.join(work_dir, "test_math_utils.py")):
            print("\n✅ Test Suite Created Successfully!")
        else:
            print("\n❌ Test file missing.")
            
    except Exception as e:
        print(f"\n❌ Error: {e}")

if __name__ == "__main__":
    asyncio.run(main())
