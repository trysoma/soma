# Insurance Claim Bot Example

This is an example agent built with the Soma Python SDK that processes insurance claims.

Read our [documentation](https://docs.trysoma.ai) to dive deeper into Soma

## Project Structure

```
insurance_claim_bot/
├── agents/
│   └── index.py          # Main agent definition
├── functions/
│   └── approve_claim.py  # Function to approve claims
├── .soma/
│   ├── standalone.py     # Auto-generated server entry point
│   └── bridge.py         # Auto-generated bridge client
├── utils.py              # Utility functions
├── pyproject.toml        # Project configuration
└── README.md             # This file
```

## Getting Started

1. Install dependencies:

```bash
uv sync
```

2. Start the development server:

```bash
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

The agent in `agents/index.py` handles insurance claim processing:

1. **Discover Claim**: Uses the chat pattern to converse with the user and extract claim details
2. **Process Claim**: Uses the workflow pattern to process the extracted claim

### Function

The function in `functions/approve_claim.py` is a simple function that approves claims.
In a real application, this would integrate with your claims processing system.

