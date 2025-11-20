# Soma

**Note:** For documentation, tutorials and getting started with Soma, please visit our [documentation](https://docs.trysoma.ai). Any documentation here is for contributing and setting up this repo.

## Prerequisites

Before you begin, ensure you have the following installed:

* [Rust](https://www.rust-lang.org/) ([GitHub](https://github.com/rust-lang/rust))
* [SQLC](https://sqlc.dev/) ([GitHub](https://github.com/sqlc-dev/sqlc))
* [Node.js](https://nodejs.org/) ([GitHub](https://github.com/nodejs/node))
* [PNPM](https://pnpm.io/) ([GitHub](https://github.com/pnpm/pnpm))
* [Docker](https://www.docker.com/) ([GitHub](https://github.com/moby/moby)) (for local development)
* [Atlas](https://atlasgo.io/) ([GitHub](https://github.com/ariga/atlas)) (for database migrations) (ensure you install community version: ```curl -sSf https://atlasgo.sh | sh -s -- --community -y```)

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
3. As part of the ```build.rs``` of the soma crate, we generate a root openapi.json file and generate typescript API client in ```crates/soma/app/src/@types/openapi.d.ts```

### Commit checks

1. ensure your branch has a changie file in .changes/unreleased summarizing your changes, otherwise run ```changie new``` and follow the instructions
2. ```make lint``` to check for lint errors. ```make lint-fix``` to fix them
3. ```make test``` to run your tests

## Troubleshooting