// Atlas configuration for database migrations

// soma - local environment
env "soma" {
  src = "file://crates/soma/dbs/soma/schema.sql"

  migration {
    dir = "file://crates/soma/dbs/soma/migrations"
  }

  //we don't actually use atlas to deploy to an env
  url = "sqlite://file?mode=memory"
  dev = "sqlite://file?mode=memory"
}

env "bridge" {
  src = "file://crates/bridge/dbs/bridge/schema.sql"

  migration {
    dir = "file://crates/bridge/dbs/bridge/migrations"
  }
  
  //we don't actually use atlas to deploy to an env
  url = "sqlite://file?mode=memory"
  dev = "sqlite://file?mode=memory"
}


