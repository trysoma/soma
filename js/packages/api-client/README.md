# @trysoma/api-client

TypeScript/JavaScript client for the [Soma](https://github.com/trysoma/soma) REST API.

## Overview

This package provides a typed API client for interacting with the Soma server. It is auto-generated from the OpenAPI specification and provides full TypeScript support.

**Note:** Most users should use `@trysoma/sdk` instead, which provides a higher-level API for building agents. This package is useful for direct API access or building custom integrations.

## Installation

```bash
npm install @trysoma/api-client
# or
pnpm add @trysoma/api-client
# or
yarn add @trysoma/api-client
```

## Quick Start

```typescript
import { Configuration, TaskApi } from "@trysoma/api-client";

// Create a configuration
const config = new Configuration({
  basePath: "http://localhost:8080",
});

// Create an API instance
const taskApi = new TaskApi(config);

// List tasks
const tasks = await taskApi.listTasks();
console.log(tasks);

// Get a specific task
const task = await taskApi.getTaskById({ taskId: "task-123" });
console.log(task);
```

## Available APIs

- `TaskApi` - Manage tasks and send messages
- `BridgeApi` - Manage provider instances and invoke functions
- `EncryptionApi` - Manage encryption keys
- `SecretApi` - Manage secrets
- `EnvironmentVariableApi` - Manage environment variables
- `A2aApi` - Agent-to-agent protocol endpoints

## Documentation

For comprehensive documentation and guides, visit [https://docs.trysoma.ai/](https://docs.trysoma.ai/)

## Repository

[https://github.com/trysoma/soma](https://github.com/trysoma/soma)
