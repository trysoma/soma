Soma is designed to run as both a CLI & standalone web server. Reivew all of the changes (since our last commit, on this branch, since main) and add short, concise logs with attribute information where necessary. We want to avoid verbose logs at the info level up because this does run as a CLI which developers use. We want precise, to the point logs. Do not use emojis
Levels:
- Trace = Explain execution mechanics. What exact steps happened? What syscall / IO / protocol interaction occurred? What happened inside a loop or retry mechanism?
- Debug = Explain decisions and state. Why did the program take this path? Which config/flag/env affected behavior? What high-level action is happening?
- Info = Communicate what the system is doing at a high level. What major step just happened? Has an important lifecycle event occurred? Did a significant operation complete successfully?
- Warning =  Signal something unexpected that didn’t stop execution. What went wrong? What did the system do instead? Is this safe to ignore short-term?
- Error = Record failure of an operation. What failed? Why did it fail? What context is needed to debug or report it?
...
INFO = what
WARN = something odd
ERROR = failure
DEBUG = why
TRACE = how
Example trace logs:
```rust
trace!("Opening database file");
trace!(sql = %query, "Executing SQL");
trace!(bytes = n, "Wrote to socket");
```
Example debug logs:
```rust
debug!("--clean flag set, removing local DB");
debug!(env = %env, "Resolved runtime environment");
debug!(timeout_ms, "Using configured timeout");
```
Example info logs:
```rust
info!("Server starting");
info!("Server listening on 0.0.0.0:8080");
info!("Database connection established");
info!(migration = %name, "Executed database migration");
info!(count = n, "Processed events");
```
Bad info logs;
```rust
info!("Opening file descriptor");
info!("Executing SQL: SELECT ...");
info!("Retrying connection (attempt 3)");
```
Example warn logs:
```rust
warn!("Config file not found, using defaults");
warn!(retry = attempt, "Retrying database connection");
warn!("Cache miss rate is high");
warn!("Failed to load optional plugin, continuing");
```
Bad warn logs:
```rust
warn!("Failed to connect to database"); // if app cannot continue
```
Good error logs:
```rust
error!(error = %err, "Failed to connect to database");
error!(path = %path.display(), "Failed to delete file");
error!(request_id, "Request failed");
```
Bad error logs:
```rust
error!("Retrying connection"); // not a failure yet
```
We use open telemetry (or will be if not implemented) and the tracing library. Follow idiomatic rust and rust best practices.