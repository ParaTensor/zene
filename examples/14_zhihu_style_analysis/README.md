# 知乎写作风格分析与生成 (Zhihu Style Analysis & Generation)

这是一个基于 Zene 的多智能体应用示例，展示了如何利用 Zene 进行复杂的网页爬取、文本分析和风格化写作。

## 功能

1.  **写作风格分析** (`run.py`):
    *   用户输入知乎个人主页链接。
    *   Zene 自动编写并执行爬虫，抓取该用户最近的文章。
    *   分析文章内容，提取写作风格（语气、用词、结构等），生成 `style_profile.md`。

2.  **风格化文章生成** (`run_generate.py`):
    *   用户输入一篇待改写的文章链接。
    *   Zene 抓取该文章内容。
    *   读取 `style_profile.md` 中的风格特征。
    *   将目标文章改写为用户的写作风格，生成 `generated_article.md`。

## 使用方法

### 1. 设置环境变量

确保你已经设置了 Zene 所需的 API Key：

```bash
export ZENE_PLANNER_API_KEY="sk-..."
export ZENE_EXECUTOR_API_KEY="sk-..."
export ZENE_REFLECTOR_API_KEY="sk-..."
```

### 2. 运行风格分析

```bash
python run.py
```
按提示输入知乎个人主页链接（例如 `https://www.zhihu.com/people/excited-vczh/posts`）。

程序运行完成后，会在 `workspace/style_profile.md` 生成风格分析报告。

### 3. 运行风格生成

```bash
python run_generate.py
```
按提示输入想要改写的文章链接。

程序运行完成后，会在 `workspace/generated_article.md` 生成改写后的文章。

## 依赖

Zene 会自动管理 Python 虚拟环境，但为了确保爬虫脚本能正常运行，建议确保环境中安装了以下库（或者 Zene 会尝试安装它们）：

*   `requests`
*   `beautifulsoup4`
*   `fake-useragent` (可选，用于生成 User-Agent)
