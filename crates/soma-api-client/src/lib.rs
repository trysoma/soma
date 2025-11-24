#![allow(clippy::uninlined_format_args)]
#![allow(clippy::empty_docs)]
#![allow(clippy::needless_return)]

// Include the generated OpenAPI client code
// OpenAPI Generator creates a src/lib.rs in the generated directory
// The generated code is copied to src/generated/ during build
// We include it as a module so mod declarations resolve correctly
#[path = "generated/lib.rs"]
mod generated;

pub use generated::*;
