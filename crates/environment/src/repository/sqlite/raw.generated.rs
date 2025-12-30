

#[allow(unused)]
use serde::{Serialize, Deserialize};
  pub struct insert_secret_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
      pub key: &'a 
          String
      ,
      pub encrypted_secret: &'a 
          String
      ,
      pub dek_alias: &'a 
          String
      ,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
  }

  pub async fn insert_secret(
    conn: &shared::libsql::Connection
    ,params: insert_secret_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO secret (
    id,
    key,
    encrypted_secret,
    dek_alias,
    created_at,
    updated_at
) VALUES (
    ?1,
    ?2,
    ?3,
    ?4,
    ?5,
    ?6
)"#, libsql::params![
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.key.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.encrypted_secret.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.dek_alias.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(params.created_at.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(params.updated_at.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct update_secret_params<'a> {
      pub encrypted_secret: &'a 
          String
      ,
      pub dek_alias: &'a 
          String
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
  }

  pub async fn update_secret(
    conn: &shared::libsql::Connection
    ,params: update_secret_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"UPDATE secret SET
    encrypted_secret = ?1,
    dek_alias = ?2,
    updated_at = ?3
WHERE id = ?4"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.encrypted_secret.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.dek_alias.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(params.updated_at.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct delete_secret_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
  }

  pub async fn delete_secret(
    conn: &shared::libsql::Connection
    ,params: delete_secret_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM secret WHERE id = ?1"#, libsql::params![
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_secret_by_id_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_secret_by_id {
      pub id:shared::primitives::WrappedUuidV4,
      pub key:String,
      pub encrypted_secret:String,
      pub dek_alias:String,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_secret_by_id(
      conn: &shared::libsql::Connection
      ,params: get_secret_by_id_params<'_>
  ) -> Result<Option<Row_get_secret_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, "key", encrypted_secret, dek_alias, created_at, updated_at FROM secret WHERE id = ?1"#).await?;
      let res = stmt.query_row(
          libsql::params![params.id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_secret_by_id {
                  id: row.get(0)?,
                  key: row.get(1)?,
                  encrypted_secret: row.get(2)?,
                  dek_alias: row.get(3)?,
                  created_at: row.get(4)?,
                  updated_at: row.get(5)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct get_secret_by_key_params<'a> {
      pub key: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_secret_by_key {
      pub id:shared::primitives::WrappedUuidV4,
      pub key:String,
      pub encrypted_secret:String,
      pub dek_alias:String,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_secret_by_key(
      conn: &shared::libsql::Connection
      ,params: get_secret_by_key_params<'_>
  ) -> Result<Option<Row_get_secret_by_key>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, "key", encrypted_secret, dek_alias, created_at, updated_at FROM secret WHERE key = ?1"#).await?;
      let res = stmt.query_row(
          libsql::params![params.key.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_secret_by_key {
                  id: row.get(0)?,
                  key: row.get(1)?,
                  encrypted_secret: row.get(2)?,
                  dek_alias: row.get(3)?,
                  created_at: row.get(4)?,
                  updated_at: row.get(5)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct get_secrets_params<'a> {
      pub cursor: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub page_size: &'a 
          i64
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_secrets {
      pub id:shared::primitives::WrappedUuidV4,
      pub key:String,
      pub encrypted_secret:String,
      pub dek_alias:String,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_secrets(
      conn: &shared::libsql::Connection
      ,params: get_secrets_params<'_>
  ) -> Result<Vec<Row_get_secrets>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, "key", encrypted_secret, dek_alias, created_at, updated_at FROM secret WHERE (created_at < ?1 OR ?1 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?2 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_secrets {
              id: row.get(0)?,
              key: row.get(1)?,
              encrypted_secret: row.get(2)?,
              dek_alias: row.get(3)?,
              created_at: row.get(4)?,
              updated_at: row.get(5)?,
          });
      }

      Ok(mapped)
  }
  pub struct insert_variable_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
      pub key: &'a 
          String
      ,
      pub value: &'a 
          String
      ,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
  }

  pub async fn insert_variable(
    conn: &shared::libsql::Connection
    ,params: insert_variable_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO variable (
    id,
    key,
    value,
    created_at,
    updated_at
) VALUES (
    ?1,
    ?2,
    ?3,
    ?4,
    ?5
)"#, libsql::params![
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.key.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.value.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(params.created_at.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(params.updated_at.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct update_variable_params<'a> {
      pub value: &'a 
          String
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
  }

  pub async fn update_variable(
    conn: &shared::libsql::Connection
    ,params: update_variable_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"UPDATE variable SET
    value = ?1,
    updated_at = ?2
WHERE id = ?3"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.value.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(params.updated_at.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct delete_variable_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
  }

  pub async fn delete_variable(
    conn: &shared::libsql::Connection
    ,params: delete_variable_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM variable WHERE id = ?1"#, libsql::params![
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_variable_by_id_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_variable_by_id {
      pub id:shared::primitives::WrappedUuidV4,
      pub key:String,
      pub value:String,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_variable_by_id(
      conn: &shared::libsql::Connection
      ,params: get_variable_by_id_params<'_>
  ) -> Result<Option<Row_get_variable_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, "key", value, created_at, updated_at FROM variable WHERE id = ?1"#).await?;
      let res = stmt.query_row(
          libsql::params![params.id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_variable_by_id {
                  id: row.get(0)?,
                  key: row.get(1)?,
                  value: row.get(2)?,
                  created_at: row.get(3)?,
                  updated_at: row.get(4)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct get_variable_by_key_params<'a> {
      pub key: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_variable_by_key {
      pub id:shared::primitives::WrappedUuidV4,
      pub key:String,
      pub value:String,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_variable_by_key(
      conn: &shared::libsql::Connection
      ,params: get_variable_by_key_params<'_>
  ) -> Result<Option<Row_get_variable_by_key>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, "key", value, created_at, updated_at FROM variable WHERE key = ?1"#).await?;
      let res = stmt.query_row(
          libsql::params![params.key.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_variable_by_key {
                  id: row.get(0)?,
                  key: row.get(1)?,
                  value: row.get(2)?,
                  created_at: row.get(3)?,
                  updated_at: row.get(4)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct get_variables_params<'a> {
      pub cursor: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub page_size: &'a 
          i64
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_variables {
      pub id:shared::primitives::WrappedUuidV4,
      pub key:String,
      pub value:String,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_variables(
      conn: &shared::libsql::Connection
      ,params: get_variables_params<'_>
  ) -> Result<Vec<Row_get_variables>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, "key", value, created_at, updated_at FROM variable WHERE (created_at < ?1 OR ?1 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?2 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_variables {
              id: row.get(0)?,
              key: row.get(1)?,
              value: row.get(2)?,
              created_at: row.get(3)?,
              updated_at: row.get(4)?,
          });
      }

      Ok(mapped)
  }
