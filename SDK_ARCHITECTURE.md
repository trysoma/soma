# SDK Architecture

This document describes the architecture for building multi-language SDKs (Node.js and Python) that expose a gRPC server for function invocation.

## Overview

The SDK is structured in three main layers:

1. **sdk-core**: Core Rust library with the gRPC server implementation over Unix sockets
2. **sdk-js** (soma-js): Node.js native module using napi-rs
3. **sdk-py**: Python extension module using PyO3

All communication happens over Unix domain sockets for efficient inter-process communication.

## Architecture

### Core Components

#### sdk-core (`crates/sdk-core`)

The core library provides:

- **`start_grpc_server()`**: Main function to start the gRPC server
- **`FunctionInvocation`**: Struct containing invocation details and a oneshot channel for responses
- **`FunctionHandler`**: Trait for handling function invocations

```rust
pub async fn start_grpc_server(
    providers: Vec<ProviderController>,
    socket_path: PathBuf,
    function_handler: FunctionHandler,
) -> Result<()>
```

The server implements the `SomaSdkService` gRPC service defined in `crates/sdk-proto/proto/soma_sdk_service.proto` and listens on a Unix domain socket:

- `Metadata`: Returns list of available providers
- `HealthCheck`: Server health check
- `InvokeFunction`: Invokes a registered function

#### Channel-Based Communication

When a function is invoked via gRPC:

1. Server creates a `FunctionInvocation` with a tokio oneshot channel
2. Calls the `FunctionHandler` with the invocation details
3. Handler processes the request (calling JS/Python code)
4. Handler sends response back via the oneshot channel
5. Server receives response and returns it to the gRPC client

```
┌─────────────┐         ┌──────────────┐         ┌────────────────┐
│  gRPC       │ Invoke  │  sdk-core    │ Channel │  Language      │
│  Client     ├────────>│  Server      ├────────>│  Runtime       │
│             │         │              │         │  (JS/Python)   │
│             │<────────┤              │<────────┤                │
└─────────────┘ Response└──────────────┘ Response└────────────────┘
```

### Language Bindings

#### Node.js (sdk-js / soma-js)

Located in `crates/sdk-js`, this provides a native Node.js module.

**Exposed Functions:**

- `startSdkServer(providersJson: string, socketPath: string): Promise<void>`

**Types:**

- `JsInvocationRequest`: Request details passed to JavaScript handlers
- `JsInvocationResponse`: Response from JavaScript handlers

**Current Status:**

The module currently returns placeholder responses (`{}`). To fully implement:

1. Add a registration mechanism for JS callbacks
2. Use napi threadsafe functions to call JS from Rust
3. Handle async responses properly

**Example Usage:**

```javascript
const { startSdkServer } = require('./index.node');

const providers = [
  {
    type_id: "my_provider",
    name: "My Provider",
    // ... provider definition
  }
];

await startSdkServer(JSON.stringify(providers), "/tmp/soma-sdk.sock");
```

#### Python (sdk-py)

Located in `crates/sdk-py`, this provides a Python extension module.

**Exposed Functions:**

- `start_sdk_server(providers_json: str, socket_path: str) -> None`

**Classes:**

- `InvocationRequest`: Request details
- `InvocationResponse`: Response object

**Current Status:**

Similar to the JS implementation, this returns placeholder responses. To fully implement:

1. Add a registration mechanism for Python callbacks
2. Call Python functions from Rust using PyO3
3. Handle async/await properly with Python's asyncio

**Example Usage:**

```python
import json
from sdk_py import start_sdk_server

providers = [
    {
        "type_id": "my_provider",
        "name": "My Provider",
        # ... provider definition
    }
]

start_sdk_server(json.dumps(providers), "/tmp/soma-sdk.sock")
```

## Protocol Buffers

The gRPC service is defined in `crates/sdk-proto/proto/soma_sdk_service.proto`.

Key message types:

- **ProviderController**: Defines a provider with its functions and credentials
- **FunctionController**: Defines a function with parameters and output schemas
- **InvokeFunctionRequest**: Contains provider/function IDs, credentials, and parameters
- **InvokeFunctionResponse**: Returns either data or an error

Protobuf types are generated with Serde support for JSON serialization.

## Building

### Prerequisites

- Rust toolchain
- Node.js (for sdk-js)
- Python 3.8+ (for sdk-py)

### Build Commands

```bash
# Check all SDK packages
cargo check -p sdk-core -p soma-js -p sdk-py

# Build Node.js module
cd crates/sdk-js
npm install
npm run build

# Build Python module
cd crates/sdk-py
pip install maturin
maturin develop
```

## Next Steps

To complete the implementation, the following enhancements are needed:

### 1. Function Registration Mechanism

Both language bindings need a way to register callback functions:

```javascript
// JavaScript
registerFunctionHandler("provider_id", "function_id", async (request) => {
  return {
    success: true,
    data: JSON.stringify(result)
  };
});
```

```python
# Python
def handler(request):
    return InvocationResponse(
        success=True,
        data=json.dumps(result)
    )

register_function_handler("provider_id", "function_id", handler)
```

### 2. Async Response Handling

Currently, the implementations send responses immediately. For proper async handling:

- **Node.js**: Use napi threadsafe functions with promises
- **Python**: Integrate with Python's asyncio event loop

### 3. Error Handling

Improve error propagation from language runtimes back to gRPC clients.

### 4. Testing

Add integration tests that:
- Start a gRPC server
- Register function handlers
- Make gRPC calls
- Verify responses

## File Structure

```
crates/
├── sdk-core/          # Core Rust library
│   ├── src/lib.rs     # Main server implementation
│   └── Cargo.toml
├── sdk-proto/         # Protocol buffer definitions
│   ├── proto/
│   │   └── soma_sdk_service.proto
│   ├── build.rs       # Proto code generation
│   └── Cargo.toml
├── sdk-js/            # Node.js bindings
│   ├── src/lib.rs     # napi-rs implementation
│   ├── example.js     # Usage example
│   └── Cargo.toml
└── sdk-py/            # Python bindings
    ├── src/lib.rs     # PyO3 implementation
    ├── example.py     # Usage example
    └── Cargo.toml
```

## References

- [napi-rs Documentation](https://napi.rs/)
- [PyO3 Documentation](https://pyo3.rs/)
- [tonic (gRPC for Rust)](https://github.com/hyperium/tonic)
- [Protocol Buffers](https://protobuf.dev/)
