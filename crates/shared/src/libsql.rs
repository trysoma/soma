use std::ops::Deref;
use std::path::Path;
use std::{collections::BTreeMap, fs, path::PathBuf};

use crate::error::CommonError;
use libsql::params::IntoParams;
use libsql::{BatchRows, Database, Rows};
use tempfile::TempDir;
use tracing::info;
use url::Url;

pub async fn write_migrations_to_temp_dir(
    migrations: &BTreeMap<&str, &str>,
) -> Result<PathBuf, anyhow::Error> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.keep();

    for (filename, contents) in migrations {
        let file_path = temp_path.join(filename);
        fs::write(file_path, contents)?;
    }

    Ok(temp_path)
}

#[derive(Debug, Clone)]
pub struct Connection(pub libsql::Connection);

impl Connection {
    pub fn new(connection: libsql::Connection) -> Self {
        Self(connection)
    }
}

impl Deref for Connection {
    type Target = libsql::Connection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[macro_export]
macro_rules! execute_with_retry {
    ($operation:expr) => {
        execute_with_retry!($operation, 10)
    };
    ($operation:expr, $max_retries:expr) => {{
        async {
            let mut _retries = 0u32;
            let _max_retries: u32 = $max_retries;

            loop {
                match $operation.await {
                    Ok(result) => break Ok(result),
                    Err(err) => {
                        let err_str = err.to_string();
                        if err_str.contains("database is locked") || err_str.contains("SQLITE_BUSY")
                        {
                            tracing::warn!("Database is locked, retrying... {:?}", err);
                            if _retries >= _max_retries {
                                break Err(err);
                            }

                            _retries += 1;

                            // Very low delay with exponential backoff
                            let delay_us = 10_000 * (1 << _retries.min(6));
                            tokio::time::sleep(std::time::Duration::from_micros(delay_us)).await;
                        } else {
                            tracing::error!("Error executing with retry: {:?}", err);
                            break Err(err);
                        }
                    }
                }
            }
        }
        .await
    }};
}

impl Connection {
    /// Execute sql query provided some type that implements [`IntoParams`] returning
    /// on success the number of rows that were changed.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # async fn run(conn: &libsql::Connection) {
    /// # use libsql::params;
    /// conn.execute("INSERT INTO foo (id) VALUES (?1)", [42]).await.unwrap();
    /// conn.execute("INSERT INTO foo (id, name) VALUES (?1, ?2)", params![42, "baz"]).await.unwrap();
    /// # }
    /// ```
    ///
    /// For more info on how to pass params check [`IntoParams`]'s docs.
    pub async fn execute(&self, sql: &str, params: impl IntoParams) -> libsql::Result<u64> {
        tracing::trace!("executing `{}`", sql);
        let params = params.into_params()?;
        execute_with_retry!(self.0.execute(sql, params.clone()), 10)
    }

    /// Execute a batch set of statements.
    ///
    /// # Return
    ///
    /// This returns a `BatchRows` currently only the `remote`  and `local` connection supports this feature and
    /// all other connection types will return an empty set always.
    pub async fn execute_batch(&self, sql: &str) -> libsql::Result<BatchRows> {
        tracing::trace!("executing batch `{}`", sql);
        execute_with_retry!(self.0.execute_batch(sql), 10)
    }

    /// Execute a batch set of statements atomically in a transaction.
    ///
    /// # Return
    ///
    /// This returns a `BatchRows` currently only the `remote` and `local` connection supports this feature and
    /// all other connection types will return an empty set always.
    pub async fn execute_transactional_batch(&self, sql: &str) -> libsql::Result<BatchRows> {
        tracing::trace!("executing batch transactional `{}`", sql);
        execute_with_retry!(self.0.execute_transactional_batch(sql), 10)
    }

    /// Execute sql query provided some type that implements [`IntoParams`] returning
    /// on success the [`Rows`].
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # async fn run(conn: &libsql::Connection) {
    /// # use libsql::params;
    /// conn.query("SELECT foo FROM bar WHERE id = ?1", [42]).await.unwrap();
    /// conn.query("SELECT foo FROM bar WHERE id = ?1 AND name = ?2", params![42, "baz"]).await.unwrap();
    /// # }
    /// ```
    /// For more info on how to pass params check [`IntoParams`]'s docs and on how to
    /// extract values out of the rows check the [`Rows`] docs.
    pub async fn query(&self, sql: &str, params: impl IntoParams) -> libsql::Result<Rows> {
        let stmt = self.prepare(sql).await?;
        let params = params.into_params()?;
        execute_with_retry!(stmt.query(params.clone()), 10)
    }
}

pub fn construct_db_folder_path(node_id: &str, data_dir: &Path) -> PathBuf {
    let node_id_sanitized = node_id.replace("::", "_");
    data_dir.join(format!("{node_id_sanitized}/dbs"))
}

pub fn construct_db_file_path(node_id: &str, data_dir: &Path, database_name: &str) -> PathBuf {
    construct_db_folder_path(node_id, data_dir).join(format!("{database_name}/local.db"))
}

pub fn create_db_file_parent_dir(
    node_id: &str,
    data_dir: &Path,
    database_name: &str,
) -> Result<(), CommonError> {
    let dbs_path = construct_db_file_path(node_id, data_dir, database_name);
    std::fs::create_dir_all(dbs_path.parent().unwrap()).map_err(|e| {
        CommonError::Unknown(anyhow::anyhow!("failed to create dbs directory: {e}"))
    })?;
    Ok(())
}

pub fn construct_db_remote_replica_url(
    remote_url: &str,
    db_path: &Path,
    auth_token: &str,
) -> Result<String, CommonError> {
    let mut conn_url = url::Url::parse(remote_url)?;

    conn_url
        .query_pairs_mut()
        .append_pair("mode", "remote_replica")
        .append_pair("path", db_path.to_string_lossy().as_ref())
        .append_pair("auth", auth_token);

    Ok(conn_url.to_string())
}

pub struct LocalConnectionParams {
    pub path_to_db_file: PathBuf,
}

pub struct RemoteReplicaConnectionParams {
    pub path_to_db_file: PathBuf,
    pub remote_url: String,
    pub auth_token: String,
}

pub struct RemoteConnectionParams {
    pub remote_url: String,
    pub auth_token: String,
}

pub enum ConnectionType {
    Local(LocalConnectionParams),
    RemoteReplica(RemoteReplicaConnectionParams),
    Remote(RemoteConnectionParams),
}

fn get_libsql_path(url_str: &str) -> Result<String, CommonError> {
    // Extract the path portion after libsql://
    let is_relative = url_str.starts_with("libsql://./");
    let url = Url::parse(url_str).unwrap();

    if is_relative {
        Ok(format!(".{}", url.path()))
    } else {
        Ok(url.path().to_string())
    }
}

impl TryFrom<Url> for ConnectionType {
    type Error = CommonError;
    fn try_from(url: Url) -> Result<Self, Self::Error> {
        if url.scheme() != "libsql" {
            let scheme = url.scheme();
            return Err(CommonError::Unknown(anyhow::anyhow!(
                "invalid scheme: {scheme}"
            )));
        }

        let mode = match url
            .query_pairs()
            .find(|(key, _)| key == "mode")
            .map(|(_, value)| value.to_string())
        {
            Some(mode) => mode,
            None => {
                return Err(CommonError::Unknown(anyhow::anyhow!(
                    "missing mode query parameter"
                )));
            }
        };

        match mode.as_str() {
            "local" => Ok(ConnectionType::Local(LocalConnectionParams {
                path_to_db_file: PathBuf::from(get_libsql_path(url.as_ref())?),
            })),
            "remote_replica" => {
                let mut remote_url = url.clone();
                remote_url.set_query(None);

                let auth_token = match url.query_pairs().find(|(key, _)| key == "auth") {
                    Some((_, value)) => value.to_string(),
                    None => {
                        return Err(CommonError::Unknown(anyhow::anyhow!(
                            "missing auth query parameter for remote replica"
                        )));
                    }
                };

                let path_to_db_file = match url.query_pairs().find(|(key, _)| key == "path") {
                    Some((_, value)) => value.to_string(),
                    None => {
                        return Err(CommonError::Unknown(anyhow::anyhow!(
                            "missing path query parameter for remote replica"
                        )));
                    }
                };

                Ok(ConnectionType::RemoteReplica(
                    RemoteReplicaConnectionParams {
                        path_to_db_file: PathBuf::from(path_to_db_file),
                        remote_url: remote_url.to_string(),
                        auth_token,
                    },
                ))
            }
            "remote" => {
                let mut remote_url = url.clone();
                remote_url.set_query(None);

                let auth_token = match url.query_pairs().find(|(key, _)| key == "auth") {
                    Some((_, value)) => value.to_string(),
                    None => {
                        return Err(CommonError::Unknown(anyhow::anyhow!(
                            "missing auth query parameter for remote replica"
                        )));
                    }
                };

                Ok(ConnectionType::Remote(RemoteConnectionParams {
                    remote_url: remote_url.to_string(),
                    auth_token,
                }))
            }
            _ => Err(CommonError::Unknown(anyhow::anyhow!(
                "invalid mode: {mode}"
            ))),
        }
    }
}

pub fn construct_db_connection_string(
    connection_type: ConnectionType,
) -> Result<String, CommonError> {
    match connection_type {
        ConnectionType::Local(params) => {
            let path = params.path_to_db_file.to_string_lossy();
            let mut conn_url = url::Url::parse(&format!("libsql://{path}"))?;
            conn_url.query_pairs_mut().append_pair("mode", "local");
            Ok(conn_url.to_string())
        }
        ConnectionType::RemoteReplica(params) => {
            let mut conn_url = url::Url::parse(&params.remote_url)?;
            conn_url
                .query_pairs_mut()
                .append_pair("mode", "remote_replica")
                .append_pair("path", params.path_to_db_file.to_string_lossy().as_ref())
                .append_pair("auth", &params.auth_token);
            Ok(conn_url.to_string())
        }
        ConnectionType::Remote(params) => {
            let mut conn_url = url::Url::parse(&params.remote_url)?;
            conn_url
                .query_pairs_mut()
                .append_pair("auth", &params.auth_token);
            Ok(conn_url.to_string())
        }
    }
}

pub fn inject_auth_token_to_db_url(
    url: &Url,
    auth_token: &Option<String>,
) -> Result<Url, CommonError> {
    let mut conn_url = url.clone();
    if let Some(auth_token) = auth_token {
        conn_url.query_pairs_mut().append_pair("auth", auth_token);
    }
    Ok(conn_url)
}

pub type Migrations<'a> = BTreeMap<&'a str, BTreeMap<&'a str, &'a str>>;

pub fn merge_nested_migrations<'a>(mergable_migrations: Vec<Migrations<'a>>) -> Migrations<'a> {
    let mut target = Migrations::new();
    for other in mergable_migrations {
        for (outer_key, inner_map) in other {
            target
                .entry(outer_key)
                .and_modify(|existing_inner| {
                    for (inner_key, value) in inner_map.iter() {
                        existing_inner.insert(*inner_key, *value);
                    }
                })
                .or_insert(inner_map);
        }
    }
    target
}

pub async fn establish_db_connection<'a>(
    connection_string: &Url,
    migrations: Option<Migrations<'a>>,
) -> Result<(Database, Connection), CommonError> {
    let connection_type = ConnectionType::try_from(connection_string.clone())?;

    async fn create_db_file_parent_dir(parent_path: Option<&Path>) -> Result<(), CommonError> {
        if let Some(path) = parent_path {
            if !std::fs::exists(path)? {
                std::fs::create_dir_all(path)?;
            }
        }
        Ok(())
    }

    let (db, conn) = match connection_type {
        ConnectionType::Local(params) => {
            info!("establishing local connection");
            let db = libsql::Builder::new_local(params.path_to_db_file.clone())
                .build()
                .await?;

            create_db_file_parent_dir(params.path_to_db_file.parent()).await?;

            let conn = db.connect()?;
            (db, conn)
        }
        ConnectionType::RemoteReplica(params) => {
            info!("establishing remote replica connection");
            create_db_file_parent_dir(params.path_to_db_file.parent()).await?;
            let db = libsql::Builder::new_remote_replica(
                params.path_to_db_file.clone(),
                params.remote_url.clone(),
                params.auth_token.clone(),
            )
            .read_your_writes(true)
            .build()
            .await?;
            let conn = db.connect()?;
            (db, conn)
        }
        ConnectionType::Remote(params) => {
            let db =
                libsql::Builder::new_remote(params.remote_url.clone(), params.auth_token.clone())
                    .build()
                    .await?;
            let conn = db.connect()?;
            (db, conn)
        }
    };

    if let Some(migrations) = migrations {
        let migrations_to_run = migrations.get("sqlite").unwrap();

        let migrations_to_run = migrations_to_run
            .iter()
            .filter(|(k, _)| k.contains(".up."))
            .map(|(k, v)| (*k, *v))
            .collect::<BTreeMap<&str, &str>>();

        let temp_dir = write_migrations_to_temp_dir(&migrations_to_run).await?;
        libsql_migration::dir::migrate(&conn, temp_dir).await?;
    }

    Ok((db, Connection(conn)))
}
