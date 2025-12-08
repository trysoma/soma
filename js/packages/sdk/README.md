# @trysoma/sdk

The official TypeScript/JavaScript SDK for building AI agents with [Soma](https://github.com/trysoma/soma).

## Installation

```bash
npm install @trysoma/sdk
# or
pnpm add @trysoma/sdk
# or
yarn add @trysoma/sdk
```

## Overview

`@trysoma/sdk` provides a high-level API for building AI agents that can:

- Define and expose functions as tools for AI models
- Handle agent-to-agent communication via the A2A protocol
- Integrate with external services through provider bridges
- Manage durable workflows with Restate

## Quick Start

```typescript
import { defineAgent, defineTool } from "@trysoma/sdk";
import { z } from "zod";

const greet = defineTool({
  name: "greet",
  description: "Greet a user by name",
  input: z.object({
    name: z.string(),
  }),
  handler: async ({ input }) => {
    return `Hello, ${input.name}!`;
  },
});

export default defineAgent({
  name: "my-agent",
  tools: [greet],
});
```

## Documentation

For comprehensive documentation and guides, visit [https://docs.trysoma.ai/](https://docs.trysoma.ai/)

## Repository

[https://github.com/trysoma/soma](https://github.com/trysoma/soma)
