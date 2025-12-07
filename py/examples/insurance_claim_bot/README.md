# Insurance Claim Bot Example

This is an example agent built with the Soma Python SDK that processes insurance claims.

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
# Generate and watch for changes
uv run python -m soma_sdk.standalone --watch .

# Or run the standalone server directly
uv run python soma/standalone.py
```

## How It Works

### Agent

The agent in `agents/index.py` handles insurance claim processing:

1. **Discover Claim**: Uses the chat pattern to converse with the user and extract claim details
2. **Process Claim**: Uses the workflow pattern to process the extracted claim

### Function

The function in `functions/approve_claim.py` is a simple function that approves claims.
In a real application, this would integrate with your claims processing system.

## Development

To regenerate the standalone.py file after modifying agents or functions:

```bash
uv run python -m soma_sdk.standalone .
```

To watch for changes and auto-regenerate:

```bash
uv run python -m soma_sdk.standalone --watch .
```
