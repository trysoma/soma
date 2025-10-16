

#[allow(unused)]
use serde::{Serialize, Deserialize};
  pub struct create_resource_server_credential_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
      pub credential_type: &'a 
          crate::logic::ResourceServerCredentialType
      ,
      pub credential_data: &'a 
          shared::primitives::WrappedJsonValue
      ,
      pub metadata: &'a 
          shared::primitives::WrappedJsonValue
      ,
      pub run_refresh_before: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
  }

  pub async fn create_resource_server_credential(
    conn: &shared::libsql::Connection
    ,params: create_resource_server_credential_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO resource_server_credential (id, credential_type, credential_data, metadata, run_refresh_before) VALUES (?, ?, ?, ?, ?)"#, libsql::params![
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <crate::logic::ResourceServerCredentialType as TryInto<libsql::Value>>::try_into(params.credential_type.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.credential_data.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.metadata.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              match params.run_refresh_before.clone() {
                Some(value) => {
                  <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
    ]).await
}
  pub struct create_user_credential_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
      pub credential_type: &'a 
          crate::logic::UserCredentialType
      ,
      pub credential_data: &'a 
          shared::primitives::WrappedJsonValue
      ,
      pub metadata: &'a 
          shared::primitives::WrappedJsonValue
      ,
      pub run_refresh_before: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
  }

  pub async fn create_user_credential(
    conn: &shared::libsql::Connection
    ,params: create_user_credential_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO user_credential (id, credential_type, credential_data, metadata, run_refresh_before) VALUES (?, ?, ?, ?, ?)"#, libsql::params![
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <crate::logic::UserCredentialType as TryInto<libsql::Value>>::try_into(params.credential_type.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.credential_data.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.metadata.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              match params.run_refresh_before.clone() {
                Some(value) => {
                  <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
    ]).await
}
  pub struct create_provider_instance_params<'a> {
      pub id: &'a 
          String
      ,
      pub provider_id: &'a 
          String
      ,
      pub resource_server_credential_id: &'a 
          shared::primitives::WrappedUuidV4
      ,
      pub user_credential_id: &'a 
          shared::primitives::WrappedUuidV4
      ,
  }

  pub async fn create_provider_instance(
    conn: &shared::libsql::Connection
    ,params: create_provider_instance_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO provider_instance (id, provider_id, resource_server_credential_id, user_credential_id) VALUES (?, ?, ?, ?)"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.provider_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.resource_server_credential_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.user_credential_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct create_function_instance_params<'a> {
      pub id: &'a 
          String
      ,
      pub function_id: &'a 
          String
      ,
      pub provider_instance_id: &'a 
          String
      ,
  }

  pub async fn create_function_instance(
    conn: &shared::libsql::Connection
    ,params: create_function_instance_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO function_instance (id, function_id, provider_instance_id) VALUES (?, ?, ?)"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.function_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.provider_instance_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct create_credential_exchange_state_params<'a> {
      pub id: &'a 
          String
      ,
      pub state: &'a 
          crate::logic::Metadata
      ,
  }

  pub async fn create_credential_exchange_state(
    conn: &shared::libsql::Connection
    ,params: create_credential_exchange_state_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO credential_exchange_state (id, state) VALUES (?, ?)"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <crate::logic::Metadata as TryInto<libsql::Value>>::try_into(params.state.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_credential_exchange_state_by_id_params<'a> {
      pub id: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_credential_exchange_state_by_id {
      pub id:String,
      pub state:crate::logic::Metadata,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_credential_exchange_state_by_id(
      conn: &shared::libsql::Connection
      ,params: get_credential_exchange_state_by_id_params<'_>
  ) -> Result<Option<Row_get_credential_exchange_state_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, state, created_at, updated_at FROM credential_exchange_state WHERE id = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_credential_exchange_state_by_id {
                  id: row.get(0)?,
                  state: row.get(1)?,
                  created_at: row.get(2)?,
                  updated_at: row.get(3)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
