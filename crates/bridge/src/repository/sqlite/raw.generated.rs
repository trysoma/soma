

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
  }

  pub async fn create_resource_server_credential(
    conn: &shared::libsql::Connection
    ,params: create_resource_server_credential_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO resource_server_credential (id, type_id, metadata, value, created_at, updated_at, next_rotation_time)
VALUES (?, ?, ?, ?, ?, ?, ?)"#, libsql::params![
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
  }
  pub async fn get_resource_server_credential_by_id(
      conn: &shared::libsql::Connection
      ,params: get_resource_server_credential_by_id_params<'_>
  ) -> Result<Option<Row_get_resource_server_credential_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, type_id, metadata, value, created_at, updated_at, next_rotation_time
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
  }

  pub async fn create_user_credential(
    conn: &shared::libsql::Connection
    ,params: create_user_credential_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO user_credential (id, type_id, metadata, value, created_at, updated_at, next_rotation_time)
VALUES (?, ?, ?, ?, ?, ?, ?)"#, libsql::params![
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
  }
  pub async fn get_user_credential_by_id(
      conn: &shared::libsql::Connection
      ,params: get_user_credential_by_id_params<'_>
  ) -> Result<Option<Row_get_user_credential_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, type_id, metadata, value, created_at, updated_at, next_rotation_time
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
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct create_provider_instance_params<'a> {
      pub id: &'a 
          String
      ,
      pub resource_server_credential_id: &'a 
          shared::primitives::WrappedUuidV4
      ,
      pub user_credential_id: &'a 
          shared::primitives::WrappedUuidV4
      ,
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
  }

  pub async fn create_provider_instance(
    conn: &shared::libsql::Connection
    ,params: create_provider_instance_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO provider_instance (id, resource_server_credential_id, user_credential_id, created_at, updated_at, provider_controller_type_id, credential_controller_type_id)
VALUES (?, ?, ?, ?, ?, ?, ?)"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.resource_server_credential_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.user_credential_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
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
      pub resource_server_credential_id:shared::primitives::WrappedUuidV4,
      pub user_credential_id:shared::primitives::WrappedUuidV4,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
      pub provider_controller_type_id:String,
      pub credential_controller_type_id:String,
  }
  pub async fn get_provider_instance_by_id(
      conn: &shared::libsql::Connection
      ,params: get_provider_instance_by_id_params<'_>
  ) -> Result<Option<Row_get_provider_instance_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, resource_server_credential_id, user_credential_id, created_at, updated_at, provider_controller_type_id, credential_controller_type_id
FROM provider_instance
WHERE id = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_provider_instance_by_id {
                  id: row.get(0)?,
                  resource_server_credential_id: row.get(1)?,
                  user_credential_id: row.get(2)?,
                  created_at: row.get(3)?,
                  updated_at: row.get(4)?,
                  provider_controller_type_id: row.get(5)?,
                  credential_controller_type_id: row.get(6)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct create_function_instance_params<'a> {
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
      pub function_controller_type_id: &'a 
          String
      ,
  }

  pub async fn create_function_instance(
    conn: &shared::libsql::Connection
    ,params: create_function_instance_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO function_instance (id, created_at, updated_at, provider_instance_id, function_controller_type_id)
VALUES (?, ?, ?, ?, ?)"#, libsql::params![
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
              <String as TryInto<libsql::Value>>::try_into(params.function_controller_type_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_function_instance_by_id_params<'a> {
      pub id: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_function_instance_by_id {
      pub id:String,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
      pub provider_instance_id:String,
      pub function_controller_type_id:String,
  }
  pub async fn get_function_instance_by_id(
      conn: &shared::libsql::Connection
      ,params: get_function_instance_by_id_params<'_>
  ) -> Result<Option<Row_get_function_instance_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, created_at, updated_at, provider_instance_id, function_controller_type_id
FROM function_instance
WHERE id = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_function_instance_by_id {
                  id: row.get(0)?,
                  created_at: row.get(1)?,
                  updated_at: row.get(2)?,
                  provider_instance_id: row.get(3)?,
                  function_controller_type_id: row.get(4)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct delete_function_instance_params<'a> {
      pub id: &'a 
          String
      ,
  }

  pub async fn delete_function_instance(
    conn: &shared::libsql::Connection
    ,params: delete_function_instance_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM function_instance WHERE id = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
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
      pub resource_server_cred_id: &'a 
          shared::primitives::WrappedUuidV4
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
    conn.execute(r#"INSERT INTO broker_state (id, created_at, updated_at, resource_server_cred_id, provider_controller_type_id, credential_controller_type_id, metadata, action)
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
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.resource_server_cred_id.clone())
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
      pub resource_server_cred_id:shared::primitives::WrappedUuidV4,
      pub provider_controller_type_id:String,
      pub credential_controller_type_id:String,
      pub metadata:crate::logic::Metadata,
      pub action:shared::primitives::WrappedJsonValue,
  }
  pub async fn get_broker_state_by_id(
      conn: &shared::libsql::Connection
      ,params: get_broker_state_by_id_params<'_>
  ) -> Result<Option<Row_get_broker_state_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, created_at, updated_at, resource_server_cred_id, provider_controller_type_id, credential_controller_type_id, metadata, action
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
                  resource_server_cred_id: row.get(3)?,
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
      pub id: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_function_instance_with_credentials {
      pub function_instance_id:String,
      pub function_instance_created_at:shared::primitives::WrappedChronoDateTime,
      pub function_instance_updated_at:shared::primitives::WrappedChronoDateTime,
      pub function_instance_provider_instance_id:String,
      pub function_controller_type_id:String,
      pub provider_instance_id:String,
      pub provider_instance_resource_server_credential_id:shared::primitives::WrappedUuidV4,
      pub provider_instance_user_credential_id:shared::primitives::WrappedUuidV4,
      pub provider_instance_created_at:shared::primitives::WrappedChronoDateTime,
      pub provider_instance_updated_at:shared::primitives::WrappedChronoDateTime,
      pub provider_controller_type_id:String,
      pub credential_controller_type_id:String,
      pub resource_server_credential_id:shared::primitives::WrappedUuidV4,
      pub resource_server_credential_type_id:String,
      pub resource_server_credential_metadata:crate::logic::Metadata,
      pub resource_server_credential_value:shared::primitives::WrappedJsonValue,
      pub resource_server_credential_created_at:shared::primitives::WrappedChronoDateTime,
      pub resource_server_credential_updated_at:shared::primitives::WrappedChronoDateTime,
      pub resource_server_credential_next_rotation_time:Option<shared::primitives::WrappedChronoDateTime> ,
      pub user_credential_id:shared::primitives::WrappedUuidV4,
      pub user_credential_type_id:String,
      pub user_credential_metadata:crate::logic::Metadata,
      pub user_credential_value:shared::primitives::WrappedJsonValue,
      pub user_credential_created_at:shared::primitives::WrappedChronoDateTime,
      pub user_credential_updated_at:shared::primitives::WrappedChronoDateTime,
      pub user_credential_next_rotation_time:Option<shared::primitives::WrappedChronoDateTime> ,
  }
  pub async fn get_function_instance_with_credentials(
      conn: &shared::libsql::Connection
      ,params: get_function_instance_with_credentials_params<'_>
  ) -> Result<Option<Row_get_function_instance_with_credentials>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT
    fi.id as function_instance_id,
    fi.created_at as function_instance_created_at,
    fi.updated_at as function_instance_updated_at,
    fi.provider_instance_id as function_instance_provider_instance_id,
    fi.function_controller_type_id,
    pi.id as provider_instance_id,
    pi.resource_server_credential_id as provider_instance_resource_server_credential_id,
    pi.user_credential_id as provider_instance_user_credential_id,
    pi.created_at as provider_instance_created_at,
    pi.updated_at as provider_instance_updated_at,
    pi.provider_controller_type_id,
    pi.credential_controller_type_id,
    rsc.id as resource_server_credential_id,
    rsc.type_id as resource_server_credential_type_id,
    rsc.metadata as resource_server_credential_metadata,
    rsc.value as resource_server_credential_value,
    rsc.created_at as resource_server_credential_created_at,
    rsc.updated_at as resource_server_credential_updated_at,
    rsc.next_rotation_time as resource_server_credential_next_rotation_time,
    uc.id as user_credential_id,
    uc.type_id as user_credential_type_id,
    uc.metadata as user_credential_metadata,
    uc.value as user_credential_value,
    uc.created_at as user_credential_created_at,
    uc.updated_at as user_credential_updated_at,
    uc.next_rotation_time as user_credential_next_rotation_time
FROM function_instance fi
JOIN provider_instance pi ON fi.provider_instance_id = pi.id
JOIN resource_server_credential rsc ON pi.resource_server_credential_id = rsc.id
JOIN user_credential uc ON pi.user_credential_id = uc.id
WHERE fi.id = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_function_instance_with_credentials {
                  function_instance_id: row.get(0)?,
                  function_instance_created_at: row.get(1)?,
                  function_instance_updated_at: row.get(2)?,
                  function_instance_provider_instance_id: row.get(3)?,
                  function_controller_type_id: row.get(4)?,
                  provider_instance_id: row.get(5)?,
                  provider_instance_resource_server_credential_id: row.get(6)?,
                  provider_instance_user_credential_id: row.get(7)?,
                  provider_instance_created_at: row.get(8)?,
                  provider_instance_updated_at: row.get(9)?,
                  provider_controller_type_id: row.get(10)?,
                  credential_controller_type_id: row.get(11)?,
                  resource_server_credential_id: row.get(12)?,
                  resource_server_credential_type_id: row.get(13)?,
                  resource_server_credential_metadata: row.get(14)?,
                  resource_server_credential_value: row.get(15)?,
                  resource_server_credential_created_at: row.get(16)?,
                  resource_server_credential_updated_at: row.get(17)?,
                  resource_server_credential_next_rotation_time: row.get(18)?,
                  user_credential_id: row.get(19)?,
                  user_credential_type_id: row.get(20)?,
                  user_credential_metadata: row.get(21)?,
                  user_credential_value: row.get(22)?,
                  user_credential_created_at: row.get(23)?,
                  user_credential_updated_at: row.get(24)?,
                  user_credential_next_rotation_time: row.get(25)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct create_data_encryption_key_params<'a> {
      pub id: &'a 
          String
      ,
      pub envelope_encryption_key_id: &'a 
          crate::logic::EnvelopeEncryptionKeyId
      ,
      pub encryption_key: &'a 
          crate::logic::EncryptedDataKey
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
              <crate::logic::EnvelopeEncryptionKeyId as TryInto<libsql::Value>>::try_into(params.envelope_encryption_key_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <crate::logic::EncryptedDataKey as TryInto<libsql::Value>>::try_into(params.encryption_key.clone())
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
      pub envelope_encryption_key_id:crate::logic::EnvelopeEncryptionKeyId,
      pub encryption_key:crate::logic::EncryptedDataKey,
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
