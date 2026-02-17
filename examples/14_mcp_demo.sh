#!/bin/bash
# 14_mcp_demo.sh
# Demonstrates how to use Zene with MCP Integration (Model Context Protocol)
set -e

# Ensure we are in project root
ROOT_DIR=$(pwd)
CONFIG_FILE="$ROOT_DIR/zene_config.toml"

echo "🚀 Starting Zene MCP Demo"
echo "--------------------------"

# 1. Create temporary configuration for git MCP server
echo "📝 Creating temporary zene_config.toml..."
cat > "$CONFIG_FILE" <<EOF
[servers.git]
command = "uvx"
args = ["mcp-server-git"]
EOF

echo "✅ Config created. Connecting to 'mcp-server-git' via uvx."

# 2. Verify tools availability (using verify_mcp example logic via CLI if available, or just running task)
# For this demo, we'll ask Zene to use the git tool.
# Note: This requires 'zene' to be built and in PATH or use cargo run.
ZENE_CMD="cargo run --quiet --release --"

echo "🤖 Asking Zene to list git files..."
# We use a simple prompt that should trigger `git__git_ls_files` or similar if available via MCP, 
# or at least `git__git_status`.
$ZENE_CMD run "Use the git MCP tool to check the current git status and tell me if there are modified files."

# 3. Cleanup
echo "🧹 Cleaning up..."
rm "$CONFIG_FILE"

echo "--------------------------"
echo "✨ Demo Complete!"
