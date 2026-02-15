# Zene API Protocol

Zene uses a JSON-RPC 2.0 style protocol over Standard I/O (Stdin/Stdout) for communication.
This allows it to be easily integrated into IDEs, scripts, or other agents.

## 1. Request Format
```json
{
  "jsonrpc": "2.0",
  "method": "METHOD_NAME",
  "params": { ... },
  "id": 1
}
```

## 2. Response Format
**Success:**
```json
{
  "jsonrpc": "2.0",
  "result": { ... },
  "id": 1
}
```

**Error:**
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32600,
    "message": "Invalid Request",
    "data": "..."
  },
  "id": 1
}
```

## 3. Core Methods

### `agent.run`
Execute a coding task.
- **params**:
  - `instruction` (string): The user's prompt.
  - `files` (array<string>, optional): Specific files to focus on.
- **result**:
  - `diff` (string): Unified diff of changes.
  - `message` (string): Explanation of changes.

### `agent.chat`
Conversational turn without immediate side effects.
- **params**:
  - `message` (string)
  - `history` (array<object>)
- **result**:
  - `response` (string)

### `tools.list`
List available capabilities.
- **result**:
  - `tools` (array<object>)

## 4. Interfaces (The "Face" of Zene)
Since Zene is headless, it exposes its capabilities through:
1.  **CLI**: Human-friendly wrapper around the API.
2.  **JSON-RPC Server**: For IDEs and other tools to integrate.
