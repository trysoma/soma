// Suppress warnings for generated OpenAPI client code
#[allow(unused_imports)]
#[allow(clippy::all)]
mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated/src/lib.rs"));
}

pub use generated::*;
