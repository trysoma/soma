# Soma Python SDK

Python SDK for building AI agents with Soma.

## Packages

This workspace contains the following packages:

- **soma-sdk** (`py/packages/sdk`): High-level Python SDK for building agents
- **soma-api-client** (`py/packages/api-client`): OpenAPI-generated API client
- **soma-sdk-core** (`crates/sdk-py`): Native Rust bindings via PyO3

## Examples

- **insurance-claim-bot** (`py/examples/insurance-claim-bot`): Example insurance claims processing agent

## Development

### Prerequisites

- Python 3.10+
- [uv](https://docs.astral.sh/uv/) package manager
- Rust toolchain (for building native bindings)

### Setup

```bash
# Navigate to the py directory
cd py

# Install dependencies
uv sync

# Build the native module (from repo root)
cd ../crates/sdk-py
maturin develop
```

### Running Examples

```bash
# Navigate to an example
cd py/examples/insurance-claim-bot

# Generate standalone.py and watch for changes
uv run python -m soma_sdk.standalone --watch .

# Or run directly
uv run python soma/standalone.py
```

## Project Structure

```
py/
├── packages/
│   ├── sdk/              # Main SDK package
│   │   └── soma_sdk/
│   │       ├── __init__.py
│   │       ├── agent.py      # Agent creation
│   │       ├── bridge.py     # Function creation
│   │       ├── patterns.py   # Chat/workflow patterns
│   │       └── standalone.py # Server generation
│   └── api-client/       # Generated API client
│       └── soma_api_client/
├── examples/
│   └── insurance-claim-bot/
│       ├── agents/       # Agent definitions
│       ├── functions/    # Function definitions
│       └── .soma/        # Generated files
├── pyproject.toml        # Workspace configuration
└── README.md             # This file
```

## License

MIT
