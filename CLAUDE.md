# Project Context

When working with this codebase, prioritize readability over cleverness. Ask clarifying questions before making architectural changes.

## About This Project

This project is a rust workspace with specific bindings for Python (using PYO3) and Node.JS (napi). It's made up of a few components, but most importantly:

* `./crates/soma` -  CLI that gets built and what developers use. It has client side code as well as code to start the `soma-api-server`
* `./crates/soma-api-server` -  axum server that contains routes from all other crates and sets up the server. In dev mode, it syncs key changes into a local project's `soma.yaml` file.
* `./crates/sdk-*` - contains code that makes Soma work with language specific SDK's (typescript, python). Mainly launches a UDS GRPC server that communicates with `soma-api-server` to relay information, trigger functions, reload / regenerate code.
* `./crates/bridge` - our custom MCP server implementation that supports SaaS integration, credential injection, custom functions
* `./crates/encryption` - encryption primitives to store secret data in the DB and `soma.yaml` file


## Standards

* always add comments and documentation to function definitions, struct definitions. Not necessarily field definitions unless they are complex or ambiguos
* always ensure code linkts (make lint-fix passes), and tests pass after major changes
* prefer simplicity and re-usability over complex functions
* never write tests for the sake of tests. Tests must be useful, impactful

## Common Commands

Review the makefile for common commands

## SDLC

### SQL changes

1. Edit the relevant SQL DB schema in `crates/*/dbs/*/schema.sql`
2. Generate a new db migration (see makefile commands)
3. update / write sqlc compatible queries in `crates/*/dbs/*/queries/{name}.sql`
4. update sqlc definition to map custom columns to types. Always use rust enums (for enum types), WrappedJsonValue, WrappedDateTime and other primitives from shared crate over just strings where possible
5. run `sqlc generate` in crate root
6. update / create the relevant raw_impl.rs. See `crates/bridge/src/repository/sqlite/raw_impl.rs` as an example
7. Update repository traits, update repository trait implementations, add tests

For pagination, JSON serialization and deserialization and other coding opinions refer to `crates/bridge` first.

### API changes

1. implement a logic function that is abstracted from the api implementation in `crates/*/src/logic` folder
2. Create route in the router that invokes this logic function
3. dependency inject all required parameters to the logic function


See `crates/bridge/src/router.rs` as a starting point for code styles and opinions



## Notes
