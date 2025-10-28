# Unix Socket gRPC Server

The SDK Core now uses Unix domain sockets for gRPC communication instead of TCP sockets.

## Why Unix Sockets?

- **Performance**: Faster IPC communication without network stack overhead
- **Security**: File system permissions control access
- **Simplicity**: No port conflicts or firewall issues
- **Local-only**: Cannot be accessed remotely

## Usage

### Starting the Server

```rust
use sdk_core::start_grpc_server;
use std::path::PathBuf;

let socket_path = PathBuf::from("/tmp/soma-sdk.sock");
start_grpc_server(providers, socket_path, handler).await?;
```

The server will:
1. Remove any existing socket file at the path
2. Create a new Unix socket listener
3. Start accepting gRPC connections over the socket

### Connecting to the Server

To connect to a Unix socket gRPC server:

**Rust (tonic):**
```rust
use tonic::transport::{Endpoint, Uri};
use tower::service_fn;

let channel = Endpoint::try_from("http://[::]:50051")?
    .connect_with_connector(service_fn(|_: Uri| {
        tokio::net::UnixStream::connect("/tmp/soma-sdk.sock")
    }))
    .await?;
```

**Node.js (grpc-js):**
```javascript
const grpc = require('@grpc/grpc-js');

const client = new YourServiceClient(
  'unix:///tmp/soma-sdk.sock',
  grpc.credentials.createInsecure()
);
```

**Python (grpcio):**
```python
import grpc

channel = grpc.insecure_channel('unix:///tmp/soma-sdk.sock')
stub = YourServiceStub(channel)
```

## Socket Cleanup

The server automatically removes the socket file before binding. However, if the server crashes, you may need to manually remove the socket file:

```bash
rm /tmp/soma-sdk.sock
```

## Permissions

The socket file inherits the umask of the process. To restrict access:

```rust
use std::os::unix::fs::PermissionsExt;

// After creating the socket
let metadata = std::fs::metadata(&socket_path)?;
let mut permissions = metadata.permissions();
permissions.set_mode(0o600); // Owner read/write only
std::fs::set_permissions(&socket_path, permissions)?;
```

## Cross-Platform Considerations

Unix sockets are available on:
- ✅ Linux
- ✅ macOS
- ✅ BSD
- ❌ Windows (use named pipes instead)

For Windows support, you would need to add conditional compilation:

```rust
#[cfg(unix)]
pub async fn start_grpc_server(socket_path: PathBuf, ...) { ... }

#[cfg(windows)]
pub async fn start_grpc_server(pipe_name: String, ...) { ... }
```
