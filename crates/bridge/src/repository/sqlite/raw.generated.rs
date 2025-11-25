

#[allow(unused)]
use serde::{Serialize, Deserialize};
  pub struct create_resource_server_credential_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
      pub type_id: &'a 
          String
      ,
      pub metadata: &'a 
          crate::logic::Metadata
      ,
      pub value: &'a 
          shared::primitives::WrappedJsonValue
      ,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub next_rotation_time: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub dek_alias: &'a 
          String
      ,
  }

  pub async fn create_resource_server_credential(
    conn: &shared::libsql::Connection
    ,params: create_resource_server_credential_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO resource_server_credential (id, type_id, metadata, value, created_at, updated_at, next_rotation_time, dek_alias)
VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#, libsql::params![
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.type_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <crate::logic::Metadata as TryInto<libsql::Value>>::try_into(params.metadata.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.value.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(params.created_at.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(params.updated_at.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              match params.next_rotation_time.clone() {
                Some(value) => {
                  <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
              <String as TryInto<libsql::Value>>::try_into(params.dek_alias.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_resource_server_credential_by_id_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_resource_server_credential_by_id {
      pub id:shared::primitives::WrappedUuidV4,
      pub type_id:String,
      pub metadata:crate::logic::Metadata,
      pub value:shared::primitives::WrappedJsonValue,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
      pub next_rotation_time:Option<shared::primitives::WrappedChronoDateTime> ,
      pub dek_alias:String,
  }
  pub async fn get_resource_server_credential_by_id(
      conn: &shared::libsql::Connection
      ,params: get_resource_server_credential_by_id_params<'_>
  ) -> Result<Option<Row_get_resource_server_credential_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, type_id, metadata, value, created_at, updated_at, next_rotation_time, dek_alias
FROM resource_server_credential
WHERE id = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_resource_server_credential_by_id {
                  id: row.get(0)?,
                  type_id: row.get(1)?,
                  metadata: row.get(2)?,
                  value: row.get(3)?,
                  created_at: row.get(4)?,
                  updated_at: row.get(5)?,
                  next_rotation_time: row.get(6)?,
                  dek_alias: row.get(7)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct create_user_credential_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
      pub type_id: &'a 
          String
      ,
      pub metadata: &'a 
          crate::logic::Metadata
      ,
      pub value: &'a 
          shared::primitives::WrappedJsonValue
      ,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub next_rotation_time: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub dek_alias: &'a 
          String
      ,
  }

  pub async fn create_user_credential(
    conn: &shared::libsql::Connection
    ,params: create_user_credential_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO user_credential (id, type_id, metadata, value, created_at, updated_at, next_rotation_time, dek_alias)
VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#, libsql::params![
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.type_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <crate::logic::Metadata as TryInto<libsql::Value>>::try_into(params.metadata.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.value.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(params.created_at.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(params.updated_at.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              match params.next_rotation_time.clone() {
                Some(value) => {
                  <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
              <String as TryInto<libsql::Value>>::try_into(params.dek_alias.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_user_credential_by_id_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_user_credential_by_id {
      pub id:shared::primitives::WrappedUuidV4,
      pub type_id:String,
      pub metadata:crate::logic::Metadata,
      pub value:shared::primitives::WrappedJsonValue,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
      pub next_rotation_time:Option<shared::primitives::WrappedChronoDateTime> ,
      pub dek_alias:String,
  }
  pub async fn get_user_credential_by_id(
      conn: &shared::libsql::Connection
      ,params: get_user_credential_by_id_params<'_>
  ) -> Result<Option<Row_get_user_credential_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, type_id, metadata, value, created_at, updated_at, next_rotation_time, dek_alias
FROM user_credential
WHERE id = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_user_credential_by_id {
                  id: row.get(0)?,
                  type_id: row.get(1)?,
                  metadata: row.get(2)?,
                  value: row.get(3)?,
                  created_at: row.get(4)?,
                  updated_at: row.get(5)?,
                  next_rotation_time: row.get(6)?,
                  dek_alias: row.get(7)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct delete_user_credential_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
  }

  pub async fn delete_user_credential(
    conn: &shared::libsql::Connection
    ,params: delete_user_credential_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM user_credential WHERE id = ?"#, libsql::params![
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct delete_resource_server_credential_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
  }

  pub async fn delete_resource_server_credential(
    conn: &shared::libsql::Connection
    ,params: delete_resource_server_credential_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM resource_server_credential WHERE id = ?"#, libsql::params![
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_user_credentials_params<'a> {
      pub cursor: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub page_size: &'a 
          i64
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_user_credentials {
      pub id:shared::primitives::WrappedUuidV4,
      pub type_id:String,
      pub metadata:crate::logic::Metadata,
      pub value:shared::primitives::WrappedJsonValue,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
      pub next_rotation_time:Option<shared::primitives::WrappedChronoDateTime> ,
      pub dek_alias:String,
  }
  pub async fn get_user_credentials(
      conn: &shared::libsql::Connection
      ,params: get_user_credentials_params<'_>
  ) -> Result<Vec<Row_get_user_credentials>, libsql::Error> {
      let stmt = conn.prepare(r#"SELECT id, type_id, metadata, value, created_at, updated_at, next_rotation_time, dek_alias
FROM user_credential WHERE (created_at < ?1 OR ?1 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?2 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_user_credentials {
              id: row.get(0)?,
              type_id: row.get(1)?,
              metadata: row.get(2)?,
              value: row.get(3)?,
              created_at: row.get(4)?,
              updated_at: row.get(5)?,
              next_rotation_time: row.get(6)?,
              dek_alias: row.get(7)?,
          });
      }

      Ok(mapped)
  }
  pub struct get_resource_server_credentials_params<'a> {
      pub cursor: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub page_size: &'a 
          i64
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_resource_server_credentials {
      pub id:shared::primitives::WrappedUuidV4,
      pub type_id:String,
      pub metadata:crate::logic::Metadata,
      pub value:shared::primitives::WrappedJsonValue,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
      pub next_rotation_time:Option<shared::primitives::WrappedChronoDateTime> ,
      pub dek_alias:String,
  }
  pub async fn get_resource_server_credentials(
      conn: &shared::libsql::Connection
      ,params: get_resource_server_credentials_params<'_>
  ) -> Result<Vec<Row_get_resource_server_credentials>, libsql::Error> {
      let stmt = conn.prepare(r#"SELECT id, type_id, metadata, value, created_at, updated_at, next_rotation_time, dek_alias
FROM resource_server_credential WHERE (created_at < ?1 OR ?1 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?2 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_resource_server_credentials {
              id: row.get(0)?,
              type_id: row.get(1)?,
              metadata: row.get(2)?,
              value: row.get(3)?,
              created_at: row.get(4)?,
              updated_at: row.get(5)?,
              next_rotation_time: row.get(6)?,
              dek_alias: row.get(7)?,
          });
      }

      Ok(mapped)
  }
  pub struct create_provider_instance_params<'a> {
      pub id: &'a 
          String
      ,
      pub display_name: &'a 
          String
      ,
      pub resource_server_credential_id: &'a 
          shared::primitives::WrappedUuidV4
      ,
      pub user_credential_id: &'a Option<
          shared::primitives::WrappedUuidV4
      >,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub provider_controller_type_id: &'a 
          String
      ,
      pub credential_controller_type_id: &'a 
          String
      ,
      pub status: &'a 
          String
      ,
      pub return_on_successful_brokering: &'a Option<
          shared::primitives::WrappedJsonValue
      >,
  }

  pub async fn create_provider_instance(
    conn: &shared::libsql::Connection
    ,params: create_provider_instance_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO provider_instance (id, display_name, resource_server_credential_id, user_credential_id, created_at, updated_at, provider_controller_type_id, credential_controller_type_id, status, return_on_successful_brokering)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.display_name.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.resource_server_credential_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              match params.user_credential_id.clone() {
                Some(value) => {
                  <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(value.clone())
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
              <String as TryInto<libsql::Value>>::try_into(params.provider_controller_type_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.credential_controller_type_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.status.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              match params.return_on_successful_brokering.clone() {
                Some(value) => {
                  <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
    ]).await
}
  pub struct update_provider_instance_params<'a> {
      pub display_name: &'a 
          String
      ,
      pub id: &'a 
          String
      ,
  }

  pub async fn update_provider_instance(
    conn: &shared::libsql::Connection
    ,params: update_provider_instance_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"UPDATE provider_instance SET display_name = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.display_name.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct update_provider_instance_after_brokering_params<'a> {
      pub user_credential_id: &'a Option<
          shared::primitives::WrappedUuidV4
      >,
      pub id: &'a 
          String
      ,
  }

  pub async fn update_provider_instance_after_brokering(
    conn: &shared::libsql::Connection
    ,params: update_provider_instance_after_brokering_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"UPDATE provider_instance SET user_credential_id = ?, status = 'active', updated_at = CURRENT_TIMESTAMP WHERE id = ?"#, libsql::params![
              match params.user_credential_id.clone() {
                Some(value) => {
                  <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_provider_instance_by_id_params<'a> {
      pub id: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_provider_instance_by_id {
      pub id:String,
      pub display_name:String,
      pub resource_server_credential_id:shared::primitives::WrappedUuidV4,
      pub user_credential_id:Option<shared::primitives::WrappedUuidV4> ,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
      pub provider_controller_type_id:String,
      pub credential_controller_type_id:String,
      pub status:String,
      pub return_on_successful_brokering:Option<shared::primitives::WrappedJsonValue> ,
      pub functions:String,
      pub resource_server_credential:String,
      pub user_credential:String,
  }
  pub async fn get_provider_instance_by_id(
      conn: &shared::libsql::Connection
      ,params: get_provider_instance_by_id_params<'_>
  ) -> Result<Option<Row_get_provider_instance_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT 
    pi.id,
    pi.display_name,
    pi.resource_server_credential_id,
    pi.user_credential_id,
    pi.created_at,
    pi.updated_at,
    pi.provider_controller_type_id,
    pi.credential_controller_type_id, pi.status, pi.return_on_successful_brokering,
    CAST(COALESCE(
        (SELECT JSON_GROUP_ARRAY(
            JSON_OBJECT(
                'function_controller_type_id', fi.function_controller_type_id,
                'provider_controller_type_id', fi.provider_controller_type_id,
                'provider_instance_id', fi.provider_instance_id,
                'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', fi.created_at),
                'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', fi.updated_at)
            )
        )
        FROM function_instance fi
        WHERE fi.provider_instance_id = pi.id
        ), JSON('[]')) AS TEXT
    ) AS functions,
    CAST(COALESCE(
        (SELECT JSON_OBJECT(
            'id', rsc.id,
            'type_id', rsc.type_id,
            'metadata', JSON(rsc.metadata),
            'value', JSON(rsc.value),
            'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', rsc.created_at),
            'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', rsc.updated_at),
            'next_rotation_time', CASE WHEN rsc.next_rotation_time IS NOT NULL THEN strftime('%Y-%m-%dT%H:%M:%fZ', rsc.next_rotation_time) ELSE NULL END,
            'dek_alias', rsc.dek_alias
        )
        FROM resource_server_credential rsc
        WHERE rsc.id = pi.resource_server_credential_id
        ), JSON('null')) AS TEXT
    ) AS resource_server_credential,
    CAST(COALESCE(
        (SELECT JSON_OBJECT(
            'id', uc.id,
            'type_id', uc.type_id,
            'metadata', JSON(uc.metadata),
            'value', JSON(uc.value),
            'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', uc.created_at),
            'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', uc.updated_at),
            'next_rotation_time', CASE WHEN uc.next_rotation_time IS NOT NULL THEN strftime('%Y-%m-%dT%H:%M:%fZ', uc.next_rotation_time) ELSE NULL END,
            'dek_alias', uc.dek_alias
        )
        FROM user_credential uc
        WHERE uc.id = pi.user_credential_id
        ), JSON('null')) AS TEXT
    ) AS user_credential
FROM provider_instance pi
WHERE pi.id = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_provider_instance_by_id {
                  id: row.get(0)?,
                  display_name: row.get(1)?,
                  resource_server_credential_id: row.get(2)?,
                  user_credential_id: row.get(3)?,
                  created_at: row.get(4)?,
                  updated_at: row.get(5)?,
                  provider_controller_type_id: row.get(6)?,
                  credential_controller_type_id: row.get(7)?,
                  status: row.get(8)?,
                  return_on_successful_brokering: row.get(9)?,
                  functions: row.get(10)?,
                  resource_server_credential: row.get(11)?,
                  user_credential: row.get(12)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct delete_provider_instance_params<'a> {
      pub id: &'a 
          String
      ,
  }

  pub async fn delete_provider_instance(
    conn: &shared::libsql::Connection
    ,params: delete_provider_instance_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM provider_instance WHERE id = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct create_function_instance_params<'a> {
      pub function_controller_type_id: &'a 
          String
      ,
      pub provider_controller_type_id: &'a 
          String
      ,
      pub provider_instance_id: &'a 
          String
      ,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
  }

  pub async fn create_function_instance(
    conn: &shared::libsql::Connection
    ,params: create_function_instance_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO function_instance (function_controller_type_id, provider_controller_type_id, provider_instance_id, created_at, updated_at)
VALUES (?, ?, ?, ?, ?)"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.function_controller_type_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.provider_controller_type_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.provider_instance_id.clone())
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
  pub struct get_function_instance_by_id_params<'a> {
      pub function_controller_type_id: &'a 
          String
      ,
      pub provider_controller_type_id: &'a 
          String
      ,
      pub provider_instance_id: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_function_instance_by_id {
      pub function_controller_type_id:String,
      pub provider_controller_type_id:String,
      pub provider_instance_id:String,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_function_instance_by_id(
      conn: &shared::libsql::Connection
      ,params: get_function_instance_by_id_params<'_>
  ) -> Result<Option<Row_get_function_instance_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT function_controller_type_id, provider_controller_type_id, provider_instance_id, created_at, updated_at
FROM function_instance
WHERE function_controller_type_id = ? AND provider_controller_type_id = ? AND provider_instance_id = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.function_controller_type_id.clone(),params.provider_controller_type_id.clone(),params.provider_instance_id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_function_instance_by_id {
                  function_controller_type_id: row.get(0)?,
                  provider_controller_type_id: row.get(1)?,
                  provider_instance_id: row.get(2)?,
                  created_at: row.get(3)?,
                  updated_at: row.get(4)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct delete_function_instance_params<'a> {
      pub function_controller_type_id: &'a 
          String
      ,
      pub provider_controller_type_id: &'a 
          String
      ,
      pub provider_instance_id: &'a 
          String
      ,
  }

  pub async fn delete_function_instance(
    conn: &shared::libsql::Connection
    ,params: delete_function_instance_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM function_instance WHERE function_controller_type_id = ? AND provider_controller_type_id = ? AND provider_instance_id = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.function_controller_type_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.provider_controller_type_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.provider_instance_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct create_broker_state_params<'a> {
      pub id: &'a 
          String
      ,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub provider_instance_id: &'a 
          String
      ,
      pub provider_controller_type_id: &'a 
          String
      ,
      pub credential_controller_type_id: &'a 
          String
      ,
      pub metadata: &'a 
          crate::logic::Metadata
      ,
      pub action: &'a 
          shared::primitives::WrappedJsonValue
      ,
  }

  pub async fn create_broker_state(
    conn: &shared::libsql::Connection
    ,params: create_broker_state_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO broker_state (id, created_at, updated_at, provider_instance_id, provider_controller_type_id, credential_controller_type_id, metadata, action)
VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(params.created_at.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(params.updated_at.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.provider_instance_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.provider_controller_type_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.credential_controller_type_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <crate::logic::Metadata as TryInto<libsql::Value>>::try_into(params.metadata.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.action.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_broker_state_by_id_params<'a> {
      pub id: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_broker_state_by_id {
      pub id:String,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
      pub provider_instance_id:String,
      pub provider_controller_type_id:String,
      pub credential_controller_type_id:String,
      pub metadata:crate::logic::Metadata,
      pub action:shared::primitives::WrappedJsonValue,
  }
  pub async fn get_broker_state_by_id(
      conn: &shared::libsql::Connection
      ,params: get_broker_state_by_id_params<'_>
  ) -> Result<Option<Row_get_broker_state_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, created_at, updated_at, provider_instance_id, provider_controller_type_id, credential_controller_type_id, metadata, action
FROM broker_state
WHERE id = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_broker_state_by_id {
                  id: row.get(0)?,
                  created_at: row.get(1)?,
                  updated_at: row.get(2)?,
                  provider_instance_id: row.get(3)?,
                  provider_controller_type_id: row.get(4)?,
                  credential_controller_type_id: row.get(5)?,
                  metadata: row.get(6)?,
                  action: row.get(7)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct delete_broker_state_params<'a> {
      pub id: &'a 
          String
      ,
  }

  pub async fn delete_broker_state(
    conn: &shared::libsql::Connection
    ,params: delete_broker_state_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM broker_state WHERE id = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_function_instance_with_credentials_params<'a> {
      pub function_controller_type_id: &'a 
          String
      ,
      pub provider_controller_type_id: &'a 
          String
      ,
      pub provider_instance_id: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_function_instance_with_credentials {
      pub function_instance_function_controller_type_id:String,
      pub function_instance_provider_controller_type_id:String,
      pub function_instance_provider_instance_id:String,
      pub function_instance_created_at:shared::primitives::WrappedChronoDateTime,
      pub function_instance_updated_at:shared::primitives::WrappedChronoDateTime,
      pub provider_instance_id:String,
      pub provider_instance_display_name:String,
      pub provider_instance_resource_server_credential_id:shared::primitives::WrappedUuidV4,
      pub provider_instance_user_credential_id:Option<shared::primitives::WrappedUuidV4> ,
      pub provider_instance_created_at:shared::primitives::WrappedChronoDateTime,
      pub provider_instance_updated_at:shared::primitives::WrappedChronoDateTime,
      pub provider_instance_provider_controller_type_id:String,
      pub credential_controller_type_id:String,
      pub provider_instance_status:String,
      pub provider_instance_return_on_successful_brokering:Option<shared::primitives::WrappedJsonValue> ,
      pub resource_server_credential_id:shared::primitives::WrappedUuidV4,
      pub resource_server_credential_type_id:String,
      pub resource_server_credential_metadata:crate::logic::Metadata,
      pub resource_server_credential_value:shared::primitives::WrappedJsonValue,
      pub resource_server_credential_created_at:shared::primitives::WrappedChronoDateTime,
      pub resource_server_credential_updated_at:shared::primitives::WrappedChronoDateTime,
      pub resource_server_credential_next_rotation_time:Option<shared::primitives::WrappedChronoDateTime> ,
      pub resource_server_credential_dek_alias:String,
      pub user_credential_id:Option<shared::primitives::WrappedUuidV4> ,
      pub user_credential_type_id:Option<String> ,
      pub user_credential_metadata:Option<crate::logic::Metadata> ,
      pub user_credential_value:Option<shared::primitives::WrappedJsonValue> ,
      pub user_credential_created_at:Option<shared::primitives::WrappedChronoDateTime> ,
      pub user_credential_updated_at:Option<shared::primitives::WrappedChronoDateTime> ,
      pub user_credential_next_rotation_time:Option<shared::primitives::WrappedChronoDateTime> ,
      pub user_credential_dek_alias:Option<String> ,
  }
  pub async fn get_function_instance_with_credentials(
      conn: &shared::libsql::Connection
      ,params: get_function_instance_with_credentials_params<'_>
  ) -> Result<Option<Row_get_function_instance_with_credentials>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT
    fi.function_controller_type_id as function_instance_function_controller_type_id,
    fi.provider_controller_type_id as function_instance_provider_controller_type_id,
    fi.provider_instance_id as function_instance_provider_instance_id,
    fi.created_at as function_instance_created_at,
    fi.updated_at as function_instance_updated_at,
    pi.id as provider_instance_id,
    pi.display_name as provider_instance_display_name,
    pi.resource_server_credential_id as provider_instance_resource_server_credential_id,
    pi.user_credential_id as provider_instance_user_credential_id,
    pi.created_at as provider_instance_created_at,
    pi.updated_at as provider_instance_updated_at,
    pi.provider_controller_type_id as provider_instance_provider_controller_type_id,
    pi.credential_controller_type_id,
    pi.status as provider_instance_status,
    pi.return_on_successful_brokering as provider_instance_return_on_successful_brokering,
    rsc.id as resource_server_credential_id,
    rsc.type_id as resource_server_credential_type_id,
    rsc.metadata as resource_server_credential_metadata,
    rsc.value as resource_server_credential_value,
    rsc.created_at as resource_server_credential_created_at,
    rsc.updated_at as resource_server_credential_updated_at,
    rsc.next_rotation_time as resource_server_credential_next_rotation_time,
    rsc.dek_alias as resource_server_credential_dek_alias,
    uc.id as user_credential_id,
    uc.type_id as user_credential_type_id,
    uc.metadata as user_credential_metadata,
    uc.value as user_credential_value,
    uc.created_at as user_credential_created_at,
    uc.updated_at as user_credential_updated_at,
    uc.next_rotation_time as user_credential_next_rotation_time,
    uc.dek_alias as user_credential_dek_alias
FROM function_instance fi
JOIN provider_instance pi ON fi.provider_instance_id = pi.id
JOIN resource_server_credential rsc ON pi.resource_server_credential_id = rsc.id
LEFT JOIN user_credential uc ON pi.user_credential_id = uc.id
WHERE fi.function_controller_type_id = ? AND fi.provider_controller_type_id = ? AND fi.provider_instance_id = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.function_controller_type_id.clone(),params.provider_controller_type_id.clone(),params.provider_instance_id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_function_instance_with_credentials {
                  function_instance_function_controller_type_id: row.get(0)?,
                  function_instance_provider_controller_type_id: row.get(1)?,
                  function_instance_provider_instance_id: row.get(2)?,
                  function_instance_created_at: row.get(3)?,
                  function_instance_updated_at: row.get(4)?,
                  provider_instance_id: row.get(5)?,
                  provider_instance_display_name: row.get(6)?,
                  provider_instance_resource_server_credential_id: row.get(7)?,
                  provider_instance_user_credential_id: row.get(8)?,
                  provider_instance_created_at: row.get(9)?,
                  provider_instance_updated_at: row.get(10)?,
                  provider_instance_provider_controller_type_id: row.get(11)?,
                  credential_controller_type_id: row.get(12)?,
                  provider_instance_status: row.get(13)?,
                  provider_instance_return_on_successful_brokering: row.get(14)?,
                  resource_server_credential_id: row.get(15)?,
                  resource_server_credential_type_id: row.get(16)?,
                  resource_server_credential_metadata: row.get(17)?,
                  resource_server_credential_value: row.get(18)?,
                  resource_server_credential_created_at: row.get(19)?,
                  resource_server_credential_updated_at: row.get(20)?,
                  resource_server_credential_next_rotation_time: row.get(21)?,
                  resource_server_credential_dek_alias: row.get(22)?,
                  user_credential_id: row.get(23)?,
                  user_credential_type_id: row.get(24)?,
                  user_credential_metadata: row.get(25)?,
                  user_credential_value: row.get(26)?,
                  user_credential_created_at: row.get(27)?,
                  user_credential_updated_at: row.get(28)?,
                  user_credential_next_rotation_time: row.get(29)?,
                  user_credential_dek_alias: row.get(30)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct get_provider_instances_params<'a> {
      pub cursor: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub status: &'a Option<
          String
      >,
      pub provider_controller_type_id: &'a Option<
          String
      >,
      pub page_size: &'a 
          i64
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_provider_instances {
      pub id:String,
      pub display_name:String,
      pub resource_server_credential_id:shared::primitives::WrappedUuidV4,
      pub user_credential_id:Option<shared::primitives::WrappedUuidV4> ,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
      pub provider_controller_type_id:String,
      pub credential_controller_type_id:String,
      pub status:String,
      pub return_on_successful_brokering:Option<shared::primitives::WrappedJsonValue> ,
      pub functions:String,
      pub resource_server_credential:String,
      pub user_credential:String,
  }
  pub async fn get_provider_instances(
      conn: &shared::libsql::Connection
      ,params: get_provider_instances_params<'_>
  ) -> Result<Vec<Row_get_provider_instances>, libsql::Error> {
      let stmt = conn.prepare(r#"SELECT
    pi.id,
    pi.display_name,
    pi.resource_server_credential_id,
    pi.user_credential_id,
    pi.created_at,
    pi.updated_at,
    pi.provider_controller_type_id,
    pi.credential_controller_type_id,
    pi.status,
    pi.return_on_successful_brokering,
    CAST(COALESCE(
        (SELECT JSON_GROUP_ARRAY(
            JSON_OBJECT(
                'function_controller_type_id', fi.function_controller_type_id,
                'provider_controller_type_id', fi.provider_controller_type_id,
                'provider_instance_id', fi.provider_instance_id,
                'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', fi.created_at),
                'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', fi.updated_at)
            )
        )
        FROM function_instance fi
        WHERE fi.provider_instance_id = pi.id
        ), JSON('[]')) AS TEXT
    ) AS functions,
    CAST(COALESCE(
        (SELECT JSON_OBJECT(
            'id', rsc.id,
            'type_id', rsc.type_id,
            'metadata', JSON(rsc.metadata),
            'value', JSON(rsc.value),
            'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', rsc.created_at),
            'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', rsc.updated_at),
            'next_rotation_time', CASE WHEN rsc.next_rotation_time IS NOT NULL THEN strftime('%Y-%m-%dT%H:%M:%fZ', rsc.next_rotation_time) ELSE NULL END,
            'dek_alias', rsc.dek_alias
        )
        FROM resource_server_credential rsc
        WHERE rsc.id = pi.resource_server_credential_id
        ), JSON('null')) AS TEXT
    ) AS resource_server_credential,
    CAST(COALESCE(
        (SELECT JSON_OBJECT(
            'id', uc.id,
            'type_id', uc.type_id,
            'metadata', JSON(uc.metadata),
            'value', JSON(uc.value),
            'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', uc.created_at),
            'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', uc.updated_at),
            'next_rotation_time', CASE WHEN uc.next_rotation_time IS NOT NULL THEN strftime('%Y-%m-%dT%H:%M:%fZ', uc.next_rotation_time) ELSE NULL END,
            'dek_alias', uc.dek_alias
        )
        FROM user_credential uc
        WHERE uc.id = pi.user_credential_id
        ), JSON('null')) AS TEXT
    ) AS user_credential
FROM provider_instance pi
WHERE (pi.created_at < ?1 OR ?1 IS NULL)
  AND (CAST(pi.status = ?2 AS TEXT) OR ?2 IS NULL)
  AND (CAST(pi.provider_controller_type_id = ?3 AS TEXT) OR ?3 IS NULL)
ORDER BY pi.created_at DESC
LIMIT CAST(?4 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.cursor.clone(),params.status.clone(),params.provider_controller_type_id.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_provider_instances {
              id: row.get(0)?,
              display_name: row.get(1)?,
              resource_server_credential_id: row.get(2)?,
              user_credential_id: row.get(3)?,
              created_at: row.get(4)?,
              updated_at: row.get(5)?,
              provider_controller_type_id: row.get(6)?,
              credential_controller_type_id: row.get(7)?,
              status: row.get(8)?,
              return_on_successful_brokering: row.get(9)?,
              functions: row.get(10)?,
              resource_server_credential: row.get(11)?,
              user_credential: row.get(12)?,
          });
      }

      Ok(mapped)
  }
  pub struct get_function_instances_params<'a> {
      pub cursor: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub provider_instance_id: &'a Option<
          String
      >,
      pub page_size: &'a 
          i64
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_function_instances {
      pub function_controller_type_id:String,
      pub provider_controller_type_id:String,
      pub provider_instance_id:String,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_function_instances(
      conn: &shared::libsql::Connection
      ,params: get_function_instances_params<'_>
  ) -> Result<Vec<Row_get_function_instances>, libsql::Error> {
      let stmt = conn.prepare(r#"SELECT function_controller_type_id, provider_controller_type_id, provider_instance_id, created_at, updated_at
FROM function_instance
WHERE (created_at < ?1 OR ?1 IS NULL)
  AND (CAST(provider_instance_id = ?2 AS TEXT) OR ?2 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?3 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.cursor.clone(),params.provider_instance_id.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_function_instances {
              function_controller_type_id: row.get(0)?,
              provider_controller_type_id: row.get(1)?,
              provider_instance_id: row.get(2)?,
              created_at: row.get(3)?,
              updated_at: row.get(4)?,
          });
      }

      Ok(mapped)
  }
  pub struct get_provider_instances_grouped_by_function_controller_type_id_params<'a> {
      pub function_controller_type_ids: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_provider_instances_grouped_by_function_controller_type_id {
      pub function_controller_type_id:String,
      pub provider_instances:String,
  }
  pub async fn get_provider_instances_grouped_by_function_controller_type_id(
      conn: &shared::libsql::Connection
      ,params: get_provider_instances_grouped_by_function_controller_type_id_params<'_>
  ) -> Result<Vec<Row_get_provider_instances_grouped_by_function_controller_type_id>, libsql::Error> {
      let stmt = conn.prepare(r#"SELECT
    fi.function_controller_type_id,
    CAST(
        JSON_GROUP_ARRAY(
            JSON_OBJECT(
                'id', pi.id,
                'display_name', pi.display_name,
                'provider_controller_type_id', pi.provider_controller_type_id,
                'credential_controller_type_id', pi.credential_controller_type_id,
                'status', pi.status,
                'return_on_successful_brokering', pi.return_on_successful_brokering,
                'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', pi.created_at),
                'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', pi.updated_at),

                -- resource server credential
                'resource_server_credential', COALESCE((
                    SELECT JSON_OBJECT(
                        'id', rsc.id,
                        'type_id', rsc.type_id,
                        'metadata', JSON(rsc.metadata),
                        'value', JSON(rsc.value),
                        'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', rsc.created_at),
                        'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', rsc.updated_at),
                        'next_rotation_time', CASE
                            WHEN rsc.next_rotation_time IS NOT NULL
                            THEN strftime('%Y-%m-%dT%H:%M:%fZ', rsc.next_rotation_time)
                            ELSE NULL END,
                        'dek_alias', rsc.dek_alias
                    )
                    FROM resource_server_credential rsc
                    WHERE rsc.id = pi.resource_server_credential_id
                ), JSON('null')),

                -- user credential
                'user_credential', COALESCE((
                    SELECT JSON_OBJECT(
                        'id', uc.id,
                        'type_id', uc.type_id,
                        'metadata', JSON(uc.metadata),
                        'value', JSON(uc.value),
                        'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', uc.created_at),
                        'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', uc.updated_at),
                        'next_rotation_time', CASE
                            WHEN uc.next_rotation_time IS NOT NULL
                            THEN strftime('%Y-%m-%dT%H:%M:%fZ', uc.next_rotation_time)
                            ELSE NULL END,
                        'dek_alias', uc.dek_alias
                    )
                    FROM user_credential uc
                    WHERE uc.id = pi.user_credential_id
                ), JSON('null')),

                -- include function_instance metadata
                'function_instance', JSON_OBJECT(
                    'provider_controller_type_id', fi.provider_controller_type_id,
                    'provider_instance_id', fi.provider_instance_id,
                    'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', fi.created_at),
                    'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', fi.updated_at)
                )
            )
        ) AS TEXT
    ) AS provider_instances
FROM function_instance fi
JOIN provider_instance pi ON fi.provider_instance_id = pi.id
WHERE (
    fi.function_controller_type_id IN (?1)
    OR ?1 IS NULL
)
GROUP BY fi.function_controller_type_id
ORDER BY fi.function_controller_type_id ASC"#).await?;
      let mut rows = stmt.query(libsql::params![params.function_controller_type_ids.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_provider_instances_grouped_by_function_controller_type_id {
              function_controller_type_id: row.get(0)?,
              provider_instances: row.get(1)?,
          });
      }

      Ok(mapped)
  }
  pub struct update_resource_server_credential_params<'a> {
      pub value: &'a Option<
          shared::primitives::WrappedJsonValue
      >,
      pub metadata: &'a Option<
          shared::primitives::WrappedJsonValue
      >,
      pub next_rotation_time: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub updated_at: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
  }

  pub async fn update_resource_server_credential(
    conn: &shared::libsql::Connection
    ,params: update_resource_server_credential_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"UPDATE resource_server_credential
SET value = CASE WHEN CAST(?1 AS JSON) IS NOT NULL
    THEN ?1
    ELSE value
    END,
    metadata = CASE WHEN CAST(?2 AS JSON) IS NOT NULL
    THEN ?2
    ELSE metadata
    END,
    next_rotation_time = CASE WHEN CAST(?3 AS DATETIME) IS NOT NULL
    THEN ?3
    ELSE next_rotation_time
    END,
    updated_at = CASE WHEN CAST(?4 AS DATETIME) IS NOT NULL
    THEN ?4
    ELSE CURRENT_TIMESTAMP
    END
WHERE id = ?5"#, libsql::params![
              match params.value.clone() {
                Some(value) => {
                  <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
              match params.metadata.clone() {
                Some(value) => {
                  <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
              match params.next_rotation_time.clone() {
                Some(value) => {
                  <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
              match params.updated_at.clone() {
                Some(value) => {
                  <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct update_user_credential_params<'a> {
      pub value: &'a Option<
          shared::primitives::WrappedJsonValue
      >,
      pub metadata: &'a Option<
          shared::primitives::WrappedJsonValue
      >,
      pub next_rotation_time: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub updated_at: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
  }

  pub async fn update_user_credential(
    conn: &shared::libsql::Connection
    ,params: update_user_credential_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"UPDATE user_credential
SET value = CASE WHEN CAST(?1 AS JSON) IS NOT NULL
    THEN ?1
    ELSE value
    END,
    metadata = CASE WHEN CAST(?2 AS JSON) IS NOT NULL
    THEN ?2
    ELSE metadata
    END,
    next_rotation_time = CASE WHEN CAST(?3 AS DATETIME) IS NOT NULL
    THEN ?3
    ELSE next_rotation_time
    END,
    updated_at = CASE WHEN CAST(?4 AS DATETIME) IS NOT NULL
    THEN ?4
    ELSE CURRENT_TIMESTAMP
    END
WHERE id = ?5"#, libsql::params![
              match params.value.clone() {
                Some(value) => {
                  <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
              match params.metadata.clone() {
                Some(value) => {
                  <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
              match params.next_rotation_time.clone() {
                Some(value) => {
                  <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
              match params.updated_at.clone() {
                Some(value) => {
                  <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_provider_instances_with_credentials_params<'a> {
      pub cursor: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub status: &'a Option<
          String
      >,
      pub rotation_window_end: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub page_size: &'a 
          i64
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_provider_instances_with_credentials {
      pub id:String,
      pub display_name:String,
      pub provider_controller_type_id:String,
      pub credential_controller_type_id:String,
      pub status:String,
      pub return_on_successful_brokering:Option<shared::primitives::WrappedJsonValue> ,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
      pub resource_server_credential:String,
      pub user_credential:String,
  }
  pub async fn get_provider_instances_with_credentials(
      conn: &shared::libsql::Connection
      ,params: get_provider_instances_with_credentials_params<'_>
  ) -> Result<Vec<Row_get_provider_instances_with_credentials>, libsql::Error> {
      let stmt = conn.prepare(r#"SELECT
    pi.id,
    pi.display_name,
    pi.provider_controller_type_id,
    pi.credential_controller_type_id,
    pi.status,
    pi.return_on_successful_brokering,
    pi.created_at,
    pi.updated_at,
    CAST(JSON_OBJECT(
        'id', rsc.id,
        'type_id', rsc.type_id,
        'metadata', JSON(rsc.metadata),
        'value', JSON(rsc.value),
        'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', rsc.created_at),
        'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', rsc.updated_at),
        'next_rotation_time', CASE
            WHEN rsc.next_rotation_time IS NOT NULL
            THEN strftime('%Y-%m-%dT%H:%M:%fZ', rsc.next_rotation_time)
            ELSE NULL END,
        'dek_alias', rsc.dek_alias
    ) AS TEXT) as resource_server_credential,
    CAST(COALESCE(
        CASE WHEN uc.id IS NOT NULL THEN
            JSON_OBJECT(
                'id', uc.id,
                'type_id', uc.type_id,
                'metadata', JSON(uc.metadata),
                'value', JSON(uc.value),
                'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', uc.created_at),
                'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', uc.updated_at),
                'next_rotation_time', CASE
                    WHEN uc.next_rotation_time IS NOT NULL
                    THEN strftime('%Y-%m-%dT%H:%M:%fZ', uc.next_rotation_time)
                    ELSE NULL END,
                'dek_alias', uc.dek_alias
            )
        ELSE NULL END,
    JSON('null')) AS TEXT) as user_credential
FROM provider_instance pi
INNER JOIN resource_server_credential rsc ON rsc.id = pi.resource_server_credential_id
LEFT JOIN user_credential uc ON uc.id = pi.user_credential_id
WHERE (pi.created_at < ?1 OR ?1 IS NULL)
  AND (pi.status = ?2 OR ?2 IS NULL)
  AND (
    (rsc.next_rotation_time IS NOT NULL AND datetime(rsc.next_rotation_time) <= ?3)
    OR
    (uc.next_rotation_time IS NOT NULL AND datetime(uc.next_rotation_time) <= ?3)
    OR
    ?3 IS NULL
  )
ORDER BY pi.created_at DESC
LIMIT CAST(?4 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.cursor.clone(),params.status.clone(),params.rotation_window_end.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_provider_instances_with_credentials {
              id: row.get(0)?,
              display_name: row.get(1)?,
              provider_controller_type_id: row.get(2)?,
              credential_controller_type_id: row.get(3)?,
              status: row.get(4)?,
              return_on_successful_brokering: row.get(5)?,
              created_at: row.get(6)?,
              updated_at: row.get(7)?,
              resource_server_credential: row.get(8)?,
              user_credential: row.get(9)?,
          });
      }

      Ok(mapped)
  }
