# SDK Core

Core library for building Soma SDKs in different languages (JavaScript, Python, etc.).

## Architecture

The SDK core provides a gRPC server that handles function invocations via a channel-based architecture over Unix sockets:

1. **Provider Registration**: Define your providers and functions using the `ProviderController` protobuf message
2. **Function Handlers**: Register callbacks that will be invoked when a function is called
3. **Channel Communication**: Each invocation uses a tokio oneshot channel to send requests and receive responses
4. **Unix Socket**: Server listens on a Unix domain socket for IPC communication

## Core Function

### `start_grpc_server`

```rust
pub async fn start_grpc_server(
    providers: Vec<ProviderController>,
    socket_path: PathBuf,
    function_handler: FunctionHandler,
) -> Result<()>
```

Starts a gRPC server that:
- Listens on a Unix domain socket (e.g., `/tmp/soma-sdk.sock`)
- Exposes the `SomaSdkService` with metadata, health check, and function invocation endpoints
- Routes function invocations to the provided handler
- Uses oneshot channels for request/response communication
- Automatically removes existing socket files before binding

## Flow

1. Client calls `InvokeFunction` on the gRPC server
2. Server creates a `FunctionInvocation` with a oneshot channel
3. Handler is called with the invocation details
4. Handler (in JS/Python) processes the request and sends response via the channel
5. Server receives response and returns it to the gRPC client

## Language Bindings

- **Node.js**: See `crates/sdk-js`
