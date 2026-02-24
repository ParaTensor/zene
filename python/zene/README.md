# Zene Python SDK

Python bindings for the [Zene](https://github.com/lipish/zene) autonomous coding engine, powered by Rust and PyO3.

## Installation

```bash
cd python/zene
maturin develop
```

## Usage

```python
import zene

client = zene.ZeneClient(
    planner_provider="deepseek",
    planner_model="deepseek-chat",
    planner_api_key="your-key"
)

client.init(work_dir="./workspace")
events = client.run("Create a hello world script", session_id="test")

for event in events:
    print(event)
```
