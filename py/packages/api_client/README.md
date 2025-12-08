# trysoma_api_client

Python client for the [Soma](https://github.com/trysoma/soma) REST API.

## Overview

This package provides a typed API client for interacting with the Soma server. It is auto-generated from the OpenAPI specification and provides full type hints support.

**Note:** Most users should use `trysoma_sdk` instead, which provides a higher-level API for building agents. This package is useful for direct API access or building custom integrations.

## Installation

```bash
pip install trysoma_api_client
# or
uv add trysoma_api_client
```

## Quick Start

```python
import trysoma_api_client
from trysoma_api_client.rest import ApiException

# Create a configuration
configuration = trysoma_api_client.Configuration(
    host="http://localhost:8080"
)

# Create an API client
with trysoma_api_client.ApiClient(configuration) as api_client:
    # Create an API instance
    task_api = trysoma_api_client.TaskApi(api_client)

    # List tasks
    tasks = task_api.list_tasks()
    print(tasks)

    # Get a specific task
    task = task_api.get_task_by_id(task_id="task-123")
    print(task)
```

## Available APIs

- `TaskApi` - Manage tasks and send messages
- `BridgeApi` - Manage provider instances and invoke functions
- `EncryptionApi` - Manage encryption keys
- `SecretApi` - Manage secrets
- `EnvironmentVariableApi` - Manage environment variables
- `A2aApi` - Agent-to-agent protocol endpoints

## Requirements

Python 3.9+

## Documentation

For comprehensive documentation and guides, visit [https://docs.trysoma.ai/](https://docs.trysoma.ai/)

## Repository

[https://github.com/trysoma/soma](https://github.com/trysoma/soma)
