# Insurance Claim Bot Example

This is an example agent built with the Soma TypeScript SDK that processes insurance claims.

Read our [documentation](https://docs.trysoma.ai) to dive deeper into Soma

## Project Structure

```
insurance-claim-bot/
├── agents/
│   └── index.ts          # Main agent definition
├── functions/
│   └── approveClaim.ts   # Function to approve claims
├── soma/
│   ├── standalone.ts     # Auto-generated server entry point
│   └── bridge.ts         # Auto-generated bridge client
├── utils.ts              # Utility functions
├── package.json          # Project configuration
└── README.md             # This file
```

## Getting Started

1. Install dependencies:

```bash
pnpm install
```

2. Start the development server:

```bash
# Generate and watch for changes
soma dev --clean
```

3. In a seperate terminal, configure OPENAI_API_KEY

```bash
soma enc-key add local --file-name local.bin
soma secret set OPENAI_API_KEY xxxx
```

4. Enable the approveClaim function: navigate to `http://localhost:3000` > Bridge > Enable functions
5. start a chat: navigate to `http://localhost:3000` > A2A > Chat

## How It Works

### Agent

The agent in `agents/index.ts` handles insurance claim processing using two patterns:

1. **Discover Claim**: Uses the `chat` pattern to converse with the user and extract claim details (date, category, reason, amount, email)
2. **Process Claim**: Uses the `workflow` pattern to process the extracted claim

The agent uses the Vercel AI SDK with OpenAI's GPT-4o model and includes Restate's durable execution middleware for reliability.

### Function

The function in `functions/approveClaim.ts` is a simple function that approves claims.
In a real application, this would integrate with your claims processing system.

