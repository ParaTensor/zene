# Zene Extension Strategy: Model Context Protocol (MCP)

Zene adopts the **Model Context Protocol (MCP)** as its standard for extensibility. This aligns Zene with the broader AI ecosystem, allowing it to leverage existing MCP servers for tools, data, and resources.

## 1. Why MCP?

Instead of inventing a custom JSON-RPC protocol, Zene implements the open standard [MCP Specification](https://modelcontextprotocol.io).

*   **Ecosystem Compatibility**: Zene can immediately use any existing MCP server (e.g., performing Google Searches, querying PostgreSQL, managing Git repositories) without custom plugins.
*   **Standardization**: MCP defines clear schemas for **Tools** (functions), **Resources** (data reading), and **Prompts**.
*   **Safety & Simplicity**: Like our original vision, MCP operates over Stdio/HTTP JSON-RPC, keeping extensions isolated in their own processes.

## 2. Architecture

Zene acts as an **MCP Client**, connecting to one or more **MCP Servers**.

```mermaid
graph LR
    Zene[Zene Core (MCP Client)] -- Stdio/HTTP --> S1[Git MCP Server]
    Zene -- Stdio/HTTP --> S2[Postgres MCP Server]
    Zene -- Stdio/HTTP --> S3[Filesystem MCP Server]
    
    S1 --> Git[Git Binary]
    S2 --> DB[(Database)]
    S3 --> FS[Local Files]
```

## 3. Implementation Roadmap

### Phase 1: Core Client (Current Priority)
*   Implement `McpClient` in Zene.
*   Support connecting to local stdio MCP servers via `zene_config.toml`.
*   Map MCP "Tools" to Zene's internal `ToolDefinition`.

### Phase 2: Zene as a Server
*   Expose Zene's internal capabilities (AST search, Codebase understanding) as an MCP Server (`zene-server`).
*   This allows *other* agents (e.g., Claude Desktop, IDEs) to use Zene's coding brains.

## 4. Configuration Example

Future `zene_config.toml` configuration:

```toml
[mcp_servers]
git = { command = "uvx", args = ["mcp-server-git"] }
postgres = { command = "docker", args = ["run", "-i", "mcp/postgres"] }
```
