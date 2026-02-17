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
    
    # Create a dirty CSV file for analysis
    dirty_csv_path = os.path.join(work_dir, "sales_data.csv")
    with open(dirty_csv_path, "w") as f:
        f.write("date,product,amount,region\n")
        f.write("2023-01-01,Widget A,100,North\n")
        f.write("2023-01-02,Widget B,200,South\n")
        f.write("2023-01-03,Widget A,,North\n")  # Missing amount
        f.write("2023-01-04,Widget C,150,East\n")
        f.write("invalid_date,Widget B,300,West\n") # Invalid date
        f.write("2023-01-05,Widget A,120,North\n")

    print(f"Created dirty CSV data at: {dirty_csv_path}")

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
    I have a CSV file `sales_data.csv` with some messy data.
    Please help me:
    1. Load the data using pandas.
    2. Clean the data: remove rows with missing values or invalid dates.
    3. Convert the 'date' column to datetime objects.
    4. Calculate the total sales amount per product.
    5. Generate a bar chart of total sales per product using matplotlib and save it as `sales_chart.png`.
    6. Save the cleaned data to `cleaned_sales_data.csv`.
    """

    print("Starting Zene Server for Data Analysis Task...")
    print(f"Working Directory: {work_dir}")
    print(f"Session ID: {session.session_id}")
    print("\n--- Sending Analysis Task ---\n")
    print(task)
    print("\n---\nWaiting for Agent execution...\n")

    try:
        result = await runner.run(task)
        print("\n=== Agent Execution Result ===")
        print(json.dumps(result, indent=2))
        
        # Verify output
        if os.path.exists(os.path.join(work_dir, "sales_chart.png")):
            print("\n✅ Analysis Completed! Chart generated successfully.")
        else:
            print("\n❌ Chart generation failed.")
            
    except Exception as e:
        print(f"\n❌ Error: {e}")

if __name__ == "__main__":
    asyncio.run(main())
