# Zene Examples

This directory contains example scripts demonstrating how to use `zene` for various tasks.

## Prerequisites

1. Build `zene`:
   ```bash
   cargo build --release
   ```
2. Set your API Key:
   ```bash
   export OPENAI_API_KEY="sk-..."
   # OR
   export DEEPSEEK_API_KEY="sk-..."
   ```
3. Add `zene` to your PATH or use `cargo run --` instead of `zene`.

## Running Examples

You can run the examples directly as shell scripts:

```bash
sh examples/01_hello_world.sh
```

Or copy the prompt inside them and run it manually.
