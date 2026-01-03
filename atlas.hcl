// Atlas configuration for database migrations

// soma - local environment
env "soma" {
  src = "file://crates/soma-api-server/dbs/soma/schema.sql"

  migration {
    dir = "file://crates/soma-api-server/dbs/soma/migrations?format=goose"
  }

  //we don't actually use atlas to deploy to an env
  url = "sqlite://file?mode=memory"
  dev = "sqlite://file?mode=memory"
}

env "mcp" {
  src = "file://crates/mcp/dbs/mcp/schema.sql"

  migration {
    dir = "file://crates/mcp/dbs/mcp/migrations?format=goose"
  }
  
  //we don't actually use atlas to deploy to an env
  url = "sqlite://file?mode=memory"
  dev = "sqlite://file?mode=memory"
}

env "encryption" {
  src = "file://crates/encryption/dbs/encryption/schema.sql"

  migration {
    dir = "file://crates/encryption/dbs/encryption/migrations?format=goose"
  }

  //we don't actually use atlas to deploy to an env
  url = "sqlite://file?mode=memory"
  dev = "sqlite://file?mode=memory"
}

env "identity" {
  src = "file://crates/identity/dbs/identity/schema.sql"

  migration {
    dir = "file://crates/identity/dbs/identity/migrations?format=goose"
  }

  //we don't actually use atlas to deploy to an env
  url = "sqlite://file?mode=memory"
  dev = "sqlite://file?mode=memory"
}

env "environment" {
  src = "file://crates/environment/dbs/environment/schema.sql"

  migration {
    dir = "file://crates/environment/dbs/environment/migrations?format=goose"
  }

  //we don't actually use atlas to deploy to an env
  url = "sqlite://file?mode=memory"
  dev = "sqlite://file?mode=memory"
}

env "a2a" {
  src = "file://crates/a2a/dbs/a2a/schema.sql"

  migration {
    dir = "file://crates/a2a/dbs/a2a/migrations?format=goose"
  }

  //we don't actually use atlas to deploy to an env
  url = "sqlite://file?mode=memory"
  dev = "sqlite://file?mode=memory"
}

env "inbox" {
  src = "file://crates/inbox/dbs/inbox/schema.sql"

  migration {
    dir = "file://crates/inbox/dbs/inbox/migrations?format=goose"
  }

  //we don't actually use atlas to deploy to an env
  url = "sqlite://file?mode=memory"
  dev = "sqlite://file?mode=memory"
}
