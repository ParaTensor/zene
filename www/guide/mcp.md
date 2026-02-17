# Model Context Protocol (MCP) Integration

Zene supports the [Model Context Protocol (MCP)](https://modelcontextprotocol.io/), allowing it to connect to external tools and resources seamlessly. This enables Zene to extend its capabilities beyond built-in tools by leveraging the vast ecosystem of MCP servers.

## How it Works

Zene acts as an **MCP Client**. It can connect to multiple **MCP Servers** simultaneously. Each server exposes a set of tools that Zene can use to complete tasks.

## Configuration

To use MCP servers, creating a `zene_config.toml` file in your project root or working directory.

### Example Configuration

```toml
# zene_config.toml

[servers.git]
command = "uvx"
args = ["mcp-server-git"]

[servers.filesystem]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/Users/me/projects"]
```

## Using MCP Tools

Once configured, Zene automatically discovers tools from connected servers. You can use them naturally in your prompts.

### Example

Using the `git` server configured above:

```bash
zene run "Check the git status and commit all changes with message 'Update documentation'"
```

Zene will:
1. Connect to the `git` MCP server.
2. Discover tools like `git_status`, `git_add`, `git_commit`.
3. Plan and execute the necessary steps using these tools.

## Supported Features

- **Dynamic Tool Loading**: Tools are loaded at runtime.
- **Multi-Server Support**: Connect to multiple servers (e.g., Git + Postgres + Filesystem).
- **Stdio Transport**: Currently supports local command-based servers (stdio).

## Troubleshooting

- Ensure the server command (e.g., `uvx`, `npx`) is in your PATH.
- Check logs for connection errors if tools are not found.
- Verify server arguments are correct.
