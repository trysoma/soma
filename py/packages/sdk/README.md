# trysoma_sdk

The official Python SDK for building AI agents with [Soma](https://github.com/trysoma/soma).

## Installation

```bash
pip install trysoma_sdk
# or
uv add trysoma_sdk
```

## Overview

`trysoma_sdk` provides a high-level API for building AI agents that can:

- Define and expose functions as tools for AI models
- Handle agent-to-agent communication via the A2A protocol
- Integrate with external services through provider bridges
- Manage durable workflows with Restate

## Quick Start

```python
from trysoma_sdk import define_agent, define_tool
from pydantic import BaseModel

class GreetInput(BaseModel):
    name: str

@define_tool(
    name="greet",
    description="Greet a user by name",
)
async def greet(input: GreetInput) -> str:
    return f"Hello, {input.name}!"

agent = define_agent(
    name="my-agent",
    tools=[greet],
)
```

## Documentation

For comprehensive documentation and guides, visit [https://docs.trysoma.ai/](https://docs.trysoma.ai/)

## Repository

[https://github.com/trysoma/soma](https://github.com/trysoma/soma)
