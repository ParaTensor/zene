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
    
    print(f"Working Directory: {work_dir}")

    # Ask for user input
    zhihu_url = os.environ.get("ZHIHU_USER_URL")
    if not zhihu_url:
        zhihu_url = input("请输入你的知乎个人主页链接 (默认: https://www.zhihu.com/people/excited-vczh/posts): ")
        if not zhihu_url:
            zhihu_url = "https://www.zhihu.com/people/excited-vczh/posts"
    else:
        print(f"Using Zhihu URL from environment: {zhihu_url}")

    # 3. Run the task
    task = f"""
    我希望分析知乎用户的写作风格。
    请帮我完成以下任务：
    
    1. 编写一个 Python 脚本 `scrape_zhihu.py`，爬取链接 '{zhihu_url}' 下最近的 5 篇文章。
       - 使用 `requests` 和 `BeautifulSoup`。
       - 需要添加 User-Agent 头以避免被拦截。
       - 将每篇文章的内容保存为单独的文本文件，放在 `workspace/articles/` 目录下，文件名使用文章标题。
       - 创建一个 `workspace/index.json` 文件，记录文章标题和原始链接的映射关系。
       - 打印出成功爬取的文章数量。
    
    2. 执行该脚本。
    
    3. 读取 `workspace/articles/` 目录下的所有文章内容。
    
    4. 分析这些文章的写作风格，包括：
       - 语气（Tone）：如幽默、严肃、讽刺等。
       - 常用词汇或句式。
       - 段落结构特点。
       - 情感色彩。
    
    5. 将分析结果保存为 `workspace/style_profile.md`。
    """

    print("Starting Zene Server for Zhihu Style Analysis Task...")
    session_id = f"zhihu-analysis-{int(time.time())}"
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
            style_profile_path = os.path.join(work_dir, "style_profile.md")
            if os.path.exists(style_profile_path):
                print("\n✅ Style Analysis Completed!")
                print(f"Check {style_profile_path} for the result.")
                with open(style_profile_path, 'r') as f:
                    print("\n--- Style Profile ---\n")
                    print(f.read())
            else:
                print(f"\n❌ Analysis failed or file not created at {style_profile_path}.")
                # Check if it was created in current dir instead of workspace
                alt_path = "style_profile.md"
                if os.path.exists(alt_path):
                    print(f"⚠️ Found file at {alt_path} instead.")
        else:
            print("\n❌ Agent failed to return a result.")
            
    except Exception as e:
        print(f"\n❌ Error: {e}")

if __name__ == "__main__":
    main()
