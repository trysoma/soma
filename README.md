# Soma

## Prerequisites

Before you begin, ensure you have the following installed:

* [Rust](https://www.rust-lang.org/) ([GitHub](https://github.com/rust-lang/rust))
* [SQLC](https://sqlc.dev/) ([GitHub](https://github.com/sqlc-dev/sqlc))
* [Node.js](https://nodejs.org/) ([GitHub](https://github.com/nodejs/node))
* [PNPM](https://pnpm.io/) ([GitHub](https://github.com/pnpm/pnpm))
* [Docker](https://www.docker.com/) ([GitHub](https://github.com/moby/moby)) (for local development)
* [Atlas](https://atlasgo.io/) ([GitHub](https://github.com/ariga/atlas)) (for database migrations)

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
make dev-insurance-bot
```

### variables

Update the .env.example and .env.local file for variables to get loaded locally.
TODO: how to insert secrets + env vars for deployments as well as any secrets that we need locally

### Database development

Suggested workflow:

```bash
# create a new migration for the internal database
make db-internal-create-migration NAME=update-user-table
#if you get errors about atlas hash, you may need to run
make db-internal-hash
# modify the migration in ./app/dbs/internal/migrations/xxx.sql
# create an sqlc queries in ./app/dbs/internal/queries/xxx.sql
# modify ./sqlc.yaml if needed to add any custom column mappings, etc.
make db-generate-py-models
#now import your generated python code from ./app/dbs/internal/generated
```
Database migrations run via atlas CLI and run on start of the FastAPI server.


### Retool database (for Retool development)

To enable local Retool development, apply the Retool schema to the local Docker Postgres instance (`postgres-retool` on port 5433):

```bash
# ensure Docker services are running
docker-compose up

# apply the Retool schema to the local retool database
make db-retool-apply
```

Creating and maintaining Retool migrations:

```bash
# create a new migration for the retool database
make db-retool-create-migration NAME=initial-seed

# if you see errors about migration hash, update it
make db-retool-hash
```


### API development

1. Add or edit a route in ./app/routes/v1/xxx.py, ensure it's included / imported in ./app/routes/v1/_\_init__.py
2. Run FastAPI (or if already running, just let hot reload run)
3. on FastAPI startup / hot reload, we generate the swagger / OpenAPI definition in ./openapi.json if you want to review
4. on FastAPI startup / hot reload, we generate the updated typescript types and react API client in ./frontend/src/@types/openapi.d.ts but you can use the client which is strongly typed in @/lib/api-client (fetch client for browser or node) or @/lib/api-client.client (react hooks client side client)

### Commit checks

1. ensure your branch has a changie file in .changes/unreleased summarizing your changes, otherwise run ```changie new``` and follow the instructions
2. ```make lint``` to check for lint errors. ```make lint-fix``` to fix them
3. ```make test``` to run your tests

## Troubleshooting