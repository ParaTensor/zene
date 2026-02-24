import os
import sys
import time

# Add the python sdk to sys.path
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "../../python")))

from zene import ZeneClient

# Simple .env loader
def load_env():
    env_path = os.path.join(os.path.dirname(__file__), ".env")
    if os.path.exists(env_path):
        print(f"Loading environment from {env_path}")
        with open(env_path, "r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if not line or line.startswith("#"):
                    continue
                try:
                    key, value = line.split("=", 1)
                    key = key.strip()
                    value = value.strip().strip('"').strip("'")
                    if key and value:
                        os.environ[key] = value
                except ValueError:
                    pass

load_env()

def main():
    # 1. Setup the environment
    work_dir = os.path.join(os.path.dirname(__file__), "workspace")
    os.makedirs(work_dir, exist_ok=True)
    
    # Check if style_profile.md exists
    style_profile_path = os.path.join(work_dir, "style_profile.md")
    if not os.path.exists(style_profile_path):
        print("❌ style_profile.md not found in workspace.")
        print("Checking fallback location...")
        if os.path.exists("style_profile.md"):
             style_profile_path = "style_profile.md"
             print("Found in current directory.")
        else:
             print("Please run run.py first to analyze your style.")
             return

    print(f"Working Directory: {work_dir}")
    print("Reading style profile...")
    with open(style_profile_path, 'r') as f:
        style_profile = f.read()

    # Ask for user input
    article_url = input("请输入要改写的文章链接: ")
    if not article_url:
        print("❌ URL cannot be empty.")
        return

    # 3. Run the task
    task = f"""
    我希望根据我之前分析的写作风格，改写一篇文章。
    
    1. 编写一个 Python 脚本 `scrape_article.py`，爬取链接 '{article_url}' 的内容。
       - 使用 `requests` 和 `BeautifulSoup`。
       - 将文章内容保存为 `workspace/source_article.txt`。
    
    2. 执行该脚本。
    
    3. 读取 `workspace/source_article.txt` 和 `workspace/style_profile.md` (或当前目录下的 style_profile.md)。
    
    4. 根据 `style_profile.md` 中的风格特点，改写 `source_article.txt` 的内容。
       - 保持原文的核心观点和逻辑。
       - 模仿目标风格的语气、用词和句式。
       - 如果原文是枯燥的新闻报道，尝试用更有趣、更生动的语言重写。
       - 如果原文是严肃的学术论文，尝试用更通俗易懂的方式重写（假设风格是科普类）。
    
    5. 将改写后的文章保存为 `workspace/generated_article.md`。
    """

    print("Starting Zene Server for Article Generation Task...")
    session_id = f"zhihu-gen-{int(time.time())}"
    print(f"Session ID: {session_id}")

    # Initialize Zene Client with environment variables
    client = ZeneClient(
        planner_provider=os.environ.get("ZENE_PLANNER_PROVIDER"),
        planner_model=os.environ.get("ZENE_PLANNER_MODEL"),
        planner_api_key=os.environ.get("ZENE_PLANNER_API_KEY"),
        executor_provider=os.environ.get("ZENE_EXECUTOR_PROVIDER"),
        executor_model=os.environ.get("ZENE_EXECUTOR_MODEL"),
        executor_api_key=os.environ.get("ZENE_EXECUTOR_API_KEY"),
        reflector_provider=os.environ.get("ZENE_REFLECTOR_PROVIDER"),
        reflector_model=os.environ.get("ZENE_REFLECTOR_MODEL"),
        reflector_api_key=os.environ.get("ZENE_REFLECTOR_API_KEY")
    )
    client.init(work_dir=os.path.dirname(os.path.abspath(__file__)))

    result = None
    try:
        # Stream events and get result
        events = client.run(task, session_id)
        for event in events:
            event_type = event.get("type")
            
            if event_type == "result":
                result = event.get("content")
                print("\n=== Task Completed ===")
                
            elif event_type == "thought":
                print(f"🤔 {event.get('content', '')}")
            elif event_type == "tool_use":
                print(f"🛠️  Using tool: {event.get('tool', 'unknown')}")
            elif event_type == "command":
                print(f"💻 Running: {event.get('command', '')}")
            elif event_type == "file_edit":
                print(f"📝 Editing: {event.get('path', '')}")

        # Verify output
        if result:
            generated_path = os.path.join(work_dir, "generated_article.md")
            if os.path.exists(generated_path):
                print("\n✅ Article Generation Completed!")
                print(f"Check {generated_path} for the result.")
                with open(generated_path, 'r') as f:
                    print("\n--- Generated Article ---\n")
                    print(f.read())
            else:
                print(f"\n❌ Generation failed or file not created at {generated_path}.")
                if os.path.exists("generated_article.md"):
                    print("Found file in current directory instead.")
        else:
            print("\n❌ Agent failed to return a result.")
            
    except Exception as e:
        print(f"\n❌ Error: {e}")

if __name__ == "__main__":
    main()
