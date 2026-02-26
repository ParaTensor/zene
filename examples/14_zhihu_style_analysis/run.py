import os
import time
from zene import ZeneClient

# Simple .env loader that searches in current and parent directories
def load_env():
    # Search in current directory and its parents (up to 3 levels)
    current_dir = os.path.dirname(os.path.abspath(__file__))
    for _ in range(3):
        env_path = os.path.join(current_dir, ".env")
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
            return
        current_dir = os.path.dirname(current_dir)

load_env()

def main():
    # 1. Setup the environment
    work_dir = os.path.join(os.path.dirname(os.path.abspath(__file__)), "workspace")
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
    我希望对知乎用户的写作风格进行**深度语义画像分析**。
    目前 7 篇核心文章的内容已经准备 in `workspace/articles/` 目录下。
    
    请帮我完成以下任务：
    
    1. **核心任务：深度语义画像分析**：
       读取 `workspace/articles/` 目录下所有 7 篇真实文章内容。
       
       **直接基于你的深度理解撰写报告**，请一次性完成以下维度的分析：
       - **叙事视角与身份构建**：分析其在不同主题（AI vs Rust/Ray）下的身份切换。
       - **修辞艺术与表达偏好**：作者如何使用比喻、对比、设问？有哪些高频出现的专业术语？
       - **逻辑架构与论证风格**：分析文章中对复杂技术概念（如 RAG、Async）的拆解方式。
       - **核心价值主张与底层色彩**：文字背后传递的价值观是什么？
       - **受众定位**：这些文章是写给谁看的？
    
    2. 将分析结果保存为 `workspace/style_profile.md`。
    
    注意：请直接生成最终报告，不需要过多的反思或复杂的工具调用。
    """

    print("Starting Zene Server for Zhihu Style Analysis Task...")
    session_id = f"zhihu-analysis-{int(time.time())}"
    print(f"Session ID: {session_id}")

    # Initialize Zene Client
    client = ZeneClient(
        planner_provider=os.environ.get("ZENE_PLANNER_PROVIDER"),
        planner_model=os.environ.get("ZENE_PLANNER_MODEL"),
        planner_api_key=os.environ.get("ZENE_PLANNER_API_KEY"),
        planner_region=os.environ.get("ZENE_PLANNER_REGION"),
        executor_provider=os.environ.get("ZENE_EXECUTOR_PROVIDER"),
        executor_model=os.environ.get("ZENE_EXECUTOR_MODEL"),
        executor_api_key=os.environ.get("ZENE_EXECUTOR_API_KEY"),
        executor_region=os.environ.get("ZENE_EXECUTOR_REGION"),
        reflector_provider=os.environ.get("ZENE_REFLECTOR_PROVIDER"),
        reflector_model=os.environ.get("ZENE_REFLECTOR_MODEL"),
        reflector_api_key=os.environ.get("ZENE_REFLECTOR_API_KEY"),
        reflector_region=os.environ.get("ZENE_REFLECTOR_REGION")
    )
    client.init(work_dir=os.path.dirname(os.path.abspath(__file__)))

    result = ""
    thought_buffer = []
    
    try:
        # Stream events and get result
        events = client.run(task, session_id)
        for event in events:
            event_type = event.get("type", "")
            
            if event_type == "Finished":
                result = event.get("content")
                print("\n=== Task Completed ===")
                
            elif event_type == "ThoughtDelta":
                content = event.get('content', '')
                thought_buffer.append(content)
                print(content, end='', flush=True)
            elif "ToolCall" in event_type:
                print(f"\n🛠️  Tool Call: {event_type}")
            elif "ToolResult" in event_type:
                res_str = event_type
                if len(res_str) > 100:
                    res_str = res_str[:100] + "..."
                print(f"\n✅ Tool Result: {res_str}")
            elif event_type == "Error":
                print(f"\n❌ Agent Error: {event.get('message')} (Code: {event.get('code')})")
            elif event_type != "PlanningStarted" and event_type != "TaskStarted":
                pass

        # Use thought buffer if result is an error message or empty
        final_content = result
        if not final_content or "error" in final_content.lower():
            final_content = "".join(thought_buffer)

        # Verify output
        if final_content:
            style_profile_path = os.path.join(work_dir, "style_profile.md")
            # Always write the final content to ensure it exists and is correct
            print(f"\n📝 Writing analysis result to {style_profile_path}...")
            with open(style_profile_path, "w", encoding="utf-8") as f:
                f.write(final_content)
            
            print("\n✅ Style Analysis Completed!")
            print(f"Check {style_profile_path} for the result.")
            print("\n--- Style Profile Summary ---\n")
            print(final_content[:1000] + ("..." if len(final_content) > 1000 else ""))
        else:
            print("\n❌ Agent failed to return a result.")
            
    except Exception as e:
        print(f"\n❌ Error: {e}")

if __name__ == "__main__":
    main()
