# Soma

**Note:** For documentation, tutorials and getting started with Soma, please visit our [documentation](https://docs.trysoma.ai). Any documentation here is for contributing and setting up this repo.

> [!IMPORTANT]  
> This is Alpha software and should not be used in production. There are outstanding authentication and production deployment features that need to be merged before being able to use this safely. Additionally, the API is subject to breaking changes in it's current state

Current work in progress:

- [x] Bridge MCP server. Support credential encryption, rotation, injection to MCP tool calls. Support existing SaaS providers out the box, support adding your own custom MCP functions.
- [x] A2A (Agent 2 Agent spec) proxy endpoint for agents. There is a managed proxy endpoint for triggering your agents and a debug chat UI in the dev server.
- [x] Fault-tolerance and suspension / resumability. Close integration with Restate which provides this functionality. 
- [x] KMS encryption for MCP credentials & agent secrets
- [x] Configurable Authentication middleware (API key management, Oauth2, OIDC endpoint protection)
- [ ] ```soma start``` command for production Dockerfile scenario's
- [ ] Group approval workflows (support suspending execution when approval from one or more internal users is required or customer / chat user approval)
- [ ] Multi-instance Bridge MCP. Support for creating multiple smaller Bridge MCP servers with less tools so that you can have multiple agents or sub-flows within an agent using different MCP server tools
- [ ] Moniker AI Gateway. Outbound API gateway for all LLM requests to proxy existing providers. This will provide improved observability and automatically integrate all LLM providers with Restate at the network level as opposed to framework level.
- [ ] Python SDK support.
- [ ] Windows Support.
- [ ] Production deployment best pracices



## Prerequisites

Before you begin, ensure you have the following installed:

* [Rust](https://www.rust-lang.org/) ([GitHub](https://github.com/rust-lang/rust))
* [SQLC](https://sqlc.dev/) ([GitHub](https://github.com/sqlc-dev/sqlc))
* [Node.js](https://nodejs.org/) ([GitHub](https://github.com/nodejs/node))
* [PNPM](https://pnpm.io/) ([GitHub](https://github.com/pnpm/pnpm))
* [Docker](https://www.docker.com/) ([GitHub](https://github.com/moby/moby)) (for local development)
* [Atlas](https://atlasgo.io/) ([GitHub](https://github.com/ariga/atlas)) (for database migrations) (ensure you install community version: ```curl -sSf https://atlasgo.sh | sh -s -- --community -y```)
* [OpenAPI Generator](https://openapi-generator.tech/docs/installation) (npm install is easiest: ```npm install @openapitools/openapi-generator-cli -g```)
* [Cargo about](https://github.com/EmbarkStudios/cargo-about) (for license information generation)
## Development

### Getting started

```bash
# install al JS deps
make install
```

From here, you can start developing locally. If you would like to develop and test a project, you can run the insurance bot, however you'll need an OPENAI_KEY

```bash
# run insurance bot. The Rust code doesn't support HMR so you will need to restart between 
# Rust changes and re-build the JS pnpm packages (make build if you want)
make dev-insurance-claim-bot
```

### Database development

Suggested workflow:

```bash
# edit either bridge or soma schema.sql file (e.g. crates/soma/dbs/soma/schema.sql)
make db-bridge-generate-migration NAME=migration_name # or make db-soma-generate-migration NAME=migration_name 
make db-bridge-generate-hash # or db-soma-generate-hash
```
Database migrations run via Rust server on startup


### API development

1. Add or edit axum routes
2. Re-build and Restart the rust server (e.g. ```make dev-insurance-claim-bot```)
3. As part of the ```build.rs``` of the soma crate, we generate a root openapi.json file and generate typescript API client in ```crates/soma-frontend/app/src/@types/openapi.d.ts```

### Commit checks

1. ensure your branch has a changie file in .changes/unreleased summarizing your changes, otherwise run ```changie new``` and follow the instructions
2. ```make lint``` to check for lint errors. ```make lint-fix``` to fix them
3. ```make test``` to run your tests

## Troubleshooting



Soma is building support for fine-grained authentication and access management. This allows end users to query Soma's API's with specific credentials. We intend to support:

* Oauth tokens supplied in standard Authorization header, allow the user to configure Oauth credential config, JWKS url and a way to extract groups and map them to Soma permissions
* API keys generated in Soma dev UI. Allow engineers to generate API keys in the soma dev UI, encrypt them with a KMS key provided and specify their permissions per API key
* All internal API's should use the same protections. 
* Agents running over UDS GRPC should get an API key injected into the runtime to make requests back to the Soma API
* The UDS server itself that runtimes spawn should be protected via an internal API key such that only the runtime that spawned them can communicate with them
* Ensure Restate workflow handlers are secure