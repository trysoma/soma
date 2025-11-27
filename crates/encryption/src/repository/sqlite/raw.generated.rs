

#[allow(unused)]
use serde::{Serialize, Deserialize};
  pub struct create_envelope_encryption_key_params<'a> {
      pub id: &'a 
          String
      ,
      pub key_type: &'a 
          crate::repository::EnvelopeEncryptionKeyType
      ,
      pub local_file_name: &'a Option<
          String
      >,
      pub aws_arn: &'a Option<
          String
      >,
      pub aws_region: &'a Option<
          String
      >,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
  }

  pub async fn create_envelope_encryption_key(
    conn: &shared::libsql::Connection
    ,params: create_envelope_encryption_key_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO envelope_encryption_key (id, key_type, local_file_name, aws_arn, aws_region, created_at, updated_at)
VALUES (?, ?, ?, ?, ?, ?, ?)"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <crate::repository::EnvelopeEncryptionKeyType as TryInto<libsql::Value>>::try_into(params.key_type.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              match params.local_file_name.clone() {
                Some(value) => {
                  <String as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
              match params.aws_arn.clone() {
                Some(value) => {
                  <String as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
              match params.aws_region.clone() {
                Some(value) => {
                  <String as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(params.created_at.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(params.updated_at.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_envelope_encryption_key_by_id_params<'a> {
      pub id: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_envelope_encryption_key_by_id {
      pub id:String,
      pub key_type:crate::repository::EnvelopeEncryptionKeyType,
      pub local_file_name:Option<String> ,
      pub aws_arn:Option<String> ,
      pub aws_region:Option<String> ,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_envelope_encryption_key_by_id(
      conn: &shared::libsql::Connection
      ,params: get_envelope_encryption_key_by_id_params<'_>
  ) -> Result<Option<Row_get_envelope_encryption_key_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, key_type, local_file_name, aws_arn, aws_region, created_at, updated_at
FROM envelope_encryption_key
WHERE id = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_envelope_encryption_key_by_id {
                  id: row.get(0)?,
                  key_type: row.get(1)?,
                  local_file_name: row.get(2)?,
                  aws_arn: row.get(3)?,
                  aws_region: row.get(4)?,
                  created_at: row.get(5)?,
                  updated_at: row.get(6)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_envelope_encryption_keys {
      pub id:String,
      pub key_type:crate::repository::EnvelopeEncryptionKeyType,
      pub local_file_name:Option<String> ,
      pub aws_arn:Option<String> ,
      pub aws_region:Option<String> ,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_envelope_encryption_keys(
      conn: &shared::libsql::Connection
  ) -> Result<Vec<Row_get_envelope_encryption_keys>, libsql::Error> {
      let stmt = conn.prepare(r#"SELECT id, key_type, local_file_name, aws_arn, aws_region, created_at, updated_at
FROM envelope_encryption_key
ORDER BY created_at DESC"#).await?;
      let mut rows = stmt.query(libsql::params![]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_envelope_encryption_keys {
              id: row.get(0)?,
              key_type: row.get(1)?,
              local_file_name: row.get(2)?,
              aws_arn: row.get(3)?,
              aws_region: row.get(4)?,
              created_at: row.get(5)?,
              updated_at: row.get(6)?,
          });
      }

      Ok(mapped)
  }
  pub struct get_envelope_encryption_keys_paginated_params<'a> {
      pub cursor: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub page_size: &'a 
          i64
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_envelope_encryption_keys_paginated {
      pub id:String,
      pub key_type:crate::repository::EnvelopeEncryptionKeyType,
      pub local_file_name:Option<String> ,
      pub aws_arn:Option<String> ,
      pub aws_region:Option<String> ,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_envelope_encryption_keys_paginated(
      conn: &shared::libsql::Connection
      ,params: get_envelope_encryption_keys_paginated_params<'_>
  ) -> Result<Vec<Row_get_envelope_encryption_keys_paginated>, libsql::Error> {
      let stmt = conn.prepare(r#"SELECT id, key_type, local_file_name, aws_arn, aws_region, created_at, updated_at
FROM envelope_encryption_key 
WHERE (created_at < ?1 OR ?1 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?2 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_envelope_encryption_keys_paginated {
              id: row.get(0)?,
              key_type: row.get(1)?,
              local_file_name: row.get(2)?,
              aws_arn: row.get(3)?,
              aws_region: row.get(4)?,
              created_at: row.get(5)?,
              updated_at: row.get(6)?,
          });
      }

      Ok(mapped)
  }
  pub struct get_data_encryption_keys_by_envelope_key_id_params<'a> {
      pub envelope_encryption_key_id: &'a 
          String
      ,
      pub cursor: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub page_size: &'a 
          i64
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_data_encryption_keys_by_envelope_key_id {
      pub id:String,
      pub envelope_encryption_key_id:String,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_data_encryption_keys_by_envelope_key_id(
      conn: &shared::libsql::Connection
      ,params: get_data_encryption_keys_by_envelope_key_id_params<'_>
  ) -> Result<Vec<Row_get_data_encryption_keys_by_envelope_key_id>, libsql::Error> {
      let stmt = conn.prepare(r#"SELECT id, envelope_encryption_key_id, created_at, updated_at
FROM data_encryption_key 
WHERE envelope_encryption_key_id = ?
  AND (created_at < ?2 OR ?2 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?3 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.envelope_encryption_key_id.clone(),params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_data_encryption_keys_by_envelope_key_id {
              id: row.get(0)?,
              envelope_encryption_key_id: row.get(1)?,
              created_at: row.get(2)?,
              updated_at: row.get(3)?,
          });
      }

      Ok(mapped)
  }
  pub struct delete_envelope_encryption_key_params<'a> {
      pub id: &'a 
          String
      ,
  }

  pub async fn delete_envelope_encryption_key(
    conn: &shared::libsql::Connection
    ,params: delete_envelope_encryption_key_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM envelope_encryption_key WHERE id = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct create_data_encryption_key_params<'a> {
      pub id: &'a 
          String
      ,
      pub envelope_encryption_key_id: &'a 
          String
      ,
      pub encryption_key: &'a 
          crate::logic::dek::EncryptedDataEncryptionKey
      ,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
  }

  pub async fn create_data_encryption_key(
    conn: &shared::libsql::Connection
    ,params: create_data_encryption_key_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO data_encryption_key (id, envelope_encryption_key_id, encryption_key, created_at, updated_at)
VALUES (?, ?, ?, ?, ?)"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.envelope_encryption_key_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <crate::logic::dek::EncryptedDataEncryptionKey as TryInto<libsql::Value>>::try_into(params.encryption_key.clone())
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
  pub struct get_data_encryption_key_by_id_params<'a> {
      pub id: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_data_encryption_key_by_id {
      pub id:String,
      pub envelope_encryption_key_id:String,
      pub encryption_key:crate::logic::dek::EncryptedDataEncryptionKey,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_data_encryption_key_by_id(
      conn: &shared::libsql::Connection
      ,params: get_data_encryption_key_by_id_params<'_>
  ) -> Result<Option<Row_get_data_encryption_key_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, envelope_encryption_key_id, encryption_key, created_at, updated_at
FROM data_encryption_key
WHERE id = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_data_encryption_key_by_id {
                  id: row.get(0)?,
                  envelope_encryption_key_id: row.get(1)?,
                  encryption_key: row.get(2)?,
                  created_at: row.get(3)?,
                  updated_at: row.get(4)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct get_data_encryption_key_by_id_with_envelope_params<'a> {
      pub id: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_data_encryption_key_by_id_with_envelope {
      pub id:String,
      pub envelope_encryption_key_id:String,
      pub encryption_key:crate::logic::dek::EncryptedDataEncryptionKey,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
      pub key_type:crate::repository::EnvelopeEncryptionKeyType,
      pub local_file_name:Option<String> ,
      pub aws_arn:Option<String> ,
      pub aws_region:Option<String> ,
  }
  pub async fn get_data_encryption_key_by_id_with_envelope(
      conn: &shared::libsql::Connection
      ,params: get_data_encryption_key_by_id_with_envelope_params<'_>
  ) -> Result<Option<Row_get_data_encryption_key_by_id_with_envelope>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT 
    dek.id,
    dek.envelope_encryption_key_id,
    dek.encryption_key,
    dek.created_at,
    dek.updated_at,
    eek.key_type,
    eek.local_file_name,
    eek.aws_arn,
    eek.aws_region
FROM data_encryption_key dek
JOIN envelope_encryption_key eek ON dek.envelope_encryption_key_id = eek.id
WHERE dek.id = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_data_encryption_key_by_id_with_envelope {
                  id: row.get(0)?,
                  envelope_encryption_key_id: row.get(1)?,
                  encryption_key: row.get(2)?,
                  created_at: row.get(3)?,
                  updated_at: row.get(4)?,
                  key_type: row.get(5)?,
                  local_file_name: row.get(6)?,
                  aws_arn: row.get(7)?,
                  aws_region: row.get(8)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct delete_data_encryption_key_params<'a> {
      pub id: &'a 
          String
      ,
  }

  pub async fn delete_data_encryption_key(
    conn: &shared::libsql::Connection
    ,params: delete_data_encryption_key_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM data_encryption_key WHERE id = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_data_encryption_keys_params<'a> {
      pub cursor: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub page_size: &'a 
          i64
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_data_encryption_keys {
      pub id:String,
      pub envelope_encryption_key_id:String,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_data_encryption_keys(
      conn: &shared::libsql::Connection
      ,params: get_data_encryption_keys_params<'_>
  ) -> Result<Vec<Row_get_data_encryption_keys>, libsql::Error> {
      let stmt = conn.prepare(r#"SELECT id, envelope_encryption_key_id, created_at, updated_at
FROM data_encryption_key 
WHERE (created_at < ?1 OR ?1 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?2 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_data_encryption_keys {
              id: row.get(0)?,
              envelope_encryption_key_id: row.get(1)?,
              created_at: row.get(2)?,
              updated_at: row.get(3)?,
          });
      }

      Ok(mapped)
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_all_data_encryption_keys_with_envelope_keys {
      pub id:String,
      pub envelope_encryption_key_id:String,
      pub encryption_key:crate::logic::dek::EncryptedDataEncryptionKey,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
      pub key_type:crate::repository::EnvelopeEncryptionKeyType,
      pub local_file_name:Option<String> ,
      pub aws_arn:Option<String> ,
      pub aws_region:Option<String> ,
  }
  pub async fn get_all_data_encryption_keys_with_envelope_keys(
      conn: &shared::libsql::Connection
  ) -> Result<Vec<Row_get_all_data_encryption_keys_with_envelope_keys>, libsql::Error> {
      let stmt = conn.prepare(r#"SELECT
    dek.id,
    dek.envelope_encryption_key_id,
    dek.encryption_key,
    dek.created_at,
    dek.updated_at,
    eek.key_type,
    eek.local_file_name,
    eek.aws_arn,
    eek.aws_region
FROM data_encryption_key dek
JOIN envelope_encryption_key eek ON dek.envelope_encryption_key_id = eek.id"#).await?;
      let mut rows = stmt.query(libsql::params![]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_all_data_encryption_keys_with_envelope_keys {
              id: row.get(0)?,
              envelope_encryption_key_id: row.get(1)?,
              encryption_key: row.get(2)?,
              created_at: row.get(3)?,
              updated_at: row.get(4)?,
              key_type: row.get(5)?,
              local_file_name: row.get(6)?,
              aws_arn: row.get(7)?,
              aws_region: row.get(8)?,
          });
      }

      Ok(mapped)
  }
  pub struct create_data_encryption_key_alias_params<'a> {
      pub alias: &'a 
          String
      ,
      pub data_encryption_key_id: &'a 
          String
      ,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
  }

  pub async fn create_data_encryption_key_alias(
    conn: &shared::libsql::Connection
    ,params: create_data_encryption_key_alias_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO data_encryption_key_alias (alias, data_encryption_key_id, created_at)
VALUES (?, ?, ?)"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.alias.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.data_encryption_key_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(params.created_at.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_data_encryption_key_alias_by_alias_params<'a> {
      pub alias: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_data_encryption_key_alias_by_alias {
      pub alias:String,
      pub data_encryption_key_id:String,
      pub created_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_data_encryption_key_alias_by_alias(
      conn: &shared::libsql::Connection
      ,params: get_data_encryption_key_alias_by_alias_params<'_>
  ) -> Result<Option<Row_get_data_encryption_key_alias_by_alias>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT alias, data_encryption_key_id, created_at
FROM data_encryption_key_alias
WHERE alias = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.alias.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_data_encryption_key_alias_by_alias {
                  alias: row.get(0)?,
                  data_encryption_key_id: row.get(1)?,
                  created_at: row.get(2)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct get_data_encryption_key_by_alias_params<'a> {
      pub alias: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_data_encryption_key_by_alias {
      pub id:String,
      pub envelope_encryption_key_id:String,
      pub encryption_key:crate::logic::dek::EncryptedDataEncryptionKey,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_data_encryption_key_by_alias(
      conn: &shared::libsql::Connection
      ,params: get_data_encryption_key_by_alias_params<'_>
  ) -> Result<Option<Row_get_data_encryption_key_by_alias>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT dek.id, dek.envelope_encryption_key_id, dek.encryption_key, dek.created_at, dek.updated_at
FROM data_encryption_key dek
JOIN data_encryption_key_alias alias ON dek.id = alias.data_encryption_key_id
WHERE alias.alias = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.alias.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_data_encryption_key_by_alias {
                  id: row.get(0)?,
                  envelope_encryption_key_id: row.get(1)?,
                  encryption_key: row.get(2)?,
                  created_at: row.get(3)?,
                  updated_at: row.get(4)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct delete_data_encryption_key_alias_params<'a> {
      pub alias: &'a 
          String
      ,
  }

  pub async fn delete_data_encryption_key_alias(
    conn: &shared::libsql::Connection
    ,params: delete_data_encryption_key_alias_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM data_encryption_key_alias WHERE alias = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.alias.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct list_aliases_for_dek_params<'a> {
      pub data_encryption_key_id: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_list_aliases_for_dek {
      pub alias:String,
      pub data_encryption_key_id:String,
      pub created_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn list_aliases_for_dek(
      conn: &shared::libsql::Connection
      ,params: list_aliases_for_dek_params<'_>
  ) -> Result<Vec<Row_list_aliases_for_dek>, libsql::Error> {
      let stmt = conn.prepare(r#"SELECT alias, data_encryption_key_id, created_at
FROM data_encryption_key_alias
WHERE data_encryption_key_id = ?
ORDER BY created_at ASC"#).await?;
      let mut rows = stmt.query(libsql::params![params.data_encryption_key_id.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_list_aliases_for_dek {
              alias: row.get(0)?,
              data_encryption_key_id: row.get(1)?,
              created_at: row.get(2)?,
          });
      }

      Ok(mapped)
  }
  pub struct update_data_encryption_key_alias_params<'a> {
      pub data_encryption_key_id: &'a 
          String
      ,
      pub alias: &'a 
          String
      ,
  }

  pub async fn update_data_encryption_key_alias(
    conn: &shared::libsql::Connection
    ,params: update_data_encryption_key_alias_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"UPDATE data_encryption_key_alias
SET data_encryption_key_id = ?
WHERE alias = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.data_encryption_key_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.alias.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
