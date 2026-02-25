# Skills Guide

Skills allow you to define custom automation tasks that the AI can execute.

## Overview

Skills are user-defined prompts that enhance the AI's capabilities for specific tasks.

## Directory Structure

```
skills/
├── my_skill/
│   ├── prompt.md      # Skill definition
│   ├── tools.yaml     # Tool permissions
│   └── config.toml    # Skill configuration
```

## Skill Definition

`prompt.md`:

```markdown
# My Skill

You are an expert code reviewer.

## Instructions

- Review code for security issues
- Check for performance problems
- Suggest improvements

## Output Format

Provide feedback in markdown format.
```

## Tool Permissions

`tools.yaml`:

```yaml
allowed_tools:
  - read
  - glob
  - grep
restricted_tools:
  - bash
  - write
```

## Configuration

`config.toml`:

```toml
[skill.my_skill]
enabled = true
description = "Expert code reviewer"
```

## Loading Skills

```toml
[skills]
enabled = true
directory = "./skills"
```

## Built-in Skills

| Skill | Description |
|-------|-------------|
| code_review | Code review assistant |
| debug | Debugging assistant |
| refactor | Refactoring suggestions |

## Skill Injection

Skills are injected into the prompt based on context:

```
System Prompt
---
[Ski11: code_review]
---
User Message
```

## Best Practices

1. Keep skills focused on single tasks
2. Use clear, specific instructions
3. Define tool permissions carefully
4. Test skills with edge cases
