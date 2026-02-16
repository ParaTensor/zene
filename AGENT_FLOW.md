## Agent-Engine Interaction

The `agent` module interacts with the `engine` module through the following interfaces:

- **AgentClient**: Used by agents to send requests to the engine.
- **AgentRunner**: Executes tasks within the agent based on the engine's instructions.

### Diagram

```
+------------------+     +------------------+     +------------------+
|     Agent       |     |     Engine       |     |     Agent       |
|   (client)      | --> |   (session)      | --> |   (runner)      |
+------------------+     +------------------+     +------------------+

+------------------+     +------------------+     +------------------+
|     Agent       |     |     Engine       |     |     Agent       |
|   (planner)      | --> |   (plan)        | --> |   (planner)      |
+------------------+     +------------------+     +------------------+

```

## Agent-Config Interaction

The `agent` module reads configuration from the `config` module. The `AgentConfig` struct contains the necessary configuration for an agent to function.

### Diagram

```
+------------------+     +------------------+     
|     Agent       | --> |     Config       |
|   (config)      |
+------------------+     +------------------+     

```

## Engine-Config Interaction

The `engine` module reads configuration from the `config` module. The `EngineConfig` struct contains the necessary configuration for the engine to function.

### Diagram

```
+------------------+     +------------------+     
|     Engine       | --> |     Config       |
|   (config)      |
+------------------+     +------------------+     

```\n
## Agent-Config Flow

The configuration flow from the `config` module to the `agent` and `engine` modules is as follows:

1. The `config` module initializes and provides the configuration.
2. The `agent` and `engine` modules read the configuration from the `config` module.
3. The modules use the configuration to perform their operations.

### Diagram

```
+------------------+     
|     Config       |
|   (module)       |
+------------------+     
    ^                
    |                
+------------------+     +------------------+     
|     Agent        |     |     Engine        |
|   (module)       | --> |   (module)       |
+------------------+     +------------------+     

```
