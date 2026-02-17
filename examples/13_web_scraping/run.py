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
    Write a Python script `scrape_quotes.py` to scrape quotes from 'http://quotes.toscrape.com'.
    
    Requirements:
    1. Use `requests` to fetch the page content.
    2. Use `BeautifulSoup` (bs4) to parse the HTML.
    3. Extract the quote text, the author name, and the tags for each quote on the first page.
    4. Store the data as a list of dictionaries.
    5. Save the result to a JSON file named `quotes.json` with indentation for readability.
    6. Print the number of quotes scraped.
    """

    print("Starting Zene Server for Web Scraping Task...")
    print(f"Session ID: {session.session_id}")
    print("\n--- Sending Scraping Task ---\n")
    print(task)
    print("\n---\nWaiting for Agent execution...\n")

    try:
        result = await runner.run(task)
        print("\n=== Agent Execution Result ===")
        print(json.dumps(result, indent=2))
        
        # Verify output
        if os.path.exists(os.path.join(work_dir, "scrape_quotes.py")) and \
           os.path.exists(os.path.join(work_dir, "quotes.json")):
            print("\n✅ Scraping Script Created and Executed Successfully!")
            print("Check quotes.json for results.")
        else:
            print("\n❌ Scraping failed or file not created.")
            
    except Exception as e:
        print(f"\n❌ Error: {e}")

if __name__ == "__main__":
    asyncio.run(main())
