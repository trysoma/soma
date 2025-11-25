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

env "bridge" {
  src = "file://crates/bridge/dbs/bridge/schema.sql"

  migration {
    dir = "file://crates/bridge/dbs/bridge/migrations?format=goose"
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


