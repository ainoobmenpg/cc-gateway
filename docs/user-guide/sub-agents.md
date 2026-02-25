# Sub-Agents Guide

Sub-agents allow task distribution for parallel processing and complex workflows.

## Overview

Sub-agents enable parallel task execution by delegating work to multiple AI instances.

## Architecture

```
Main Agent
    ↓
┌─────────────────────────────────────────┐
│  Agent Manager                          │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐  │
│  │ Agent 1 │ │ Agent 2 │ │ Agent 3 │  │
│  └────┬────┘ └────┬────┘ └────┬────┘  │
└───────┼──────────┼──────────┼────────┘
        ↓          ↓          ↓
    Results    Results    Results
        ↓          ↓          ↓
    ┌─────────────────────────────────┐
    │  Result Aggregation             │
    └─────────────────────────────────┘
```

## Configuration

```toml
[agents]
enabled = true
max_agents = 5
timeout = 300  # seconds
```

## Agent Types

| Type | Use Case |
|------|----------|
| parallel | Multiple independent tasks |
| sequential | Dependent tasks |
| hierarchical | Main + sub agents |

## Usage

### Parallel Execution

```json
{
    "type": "parallel",
    "tasks": [
        {"prompt": "Task 1"},
       Task 2"},
        {"prompt": {"prompt": " "Task 3"}
    ]
}
```

### Hierarchical Execution

```json
{
    "type": "hierarchical",
    "main": {"prompt": "Main task"},
    "sub": [
        {"prompt": "Subtask 1"},
        {"prompt": "Subtask 2"}
    ]
}
```

## Node Operations

Sub-agents can be managed via the Nodes tool:

```json
{
    "type": "nodes",
    "action": "list"
}
```

```json
{
    "type": "nodes",
    "action": "create",
    "config": {
        "name": "worker-1",
        "capabilities": ["read", "grep"]
    }
}
```

## Resource Management

```toml
[agents.resources]
max_memory_mb = 512
max_cpu_percent = 50
max_concurrent = 3
```

## Monitoring

```bash
# List active agents
cargo run -- agent list

# Agent status
cargo run -- agent status <agent-id>
```

## Best Practices

1. Use appropriate number of sub-agents
2. Define clear task boundaries
3. Monitor resource usage
4. Handle failures gracefully
