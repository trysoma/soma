

#[allow(unused)]
use serde::{Serialize, Deserialize};
  pub struct insert_push_notification_config_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
      pub task_id: &'a 
          shared::primitives::WrappedUuidV4
      ,
      pub url: &'a 
          String
      ,
      pub token: &'a Option<
          String
      >,
      pub authentication: &'a Option<
          shared::primitives::WrappedJsonValue
      >,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
  }

  pub async fn insert_push_notification_config(
    conn: &shared::libsql::Connection
    ,params: insert_push_notification_config_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO push_notification_config (
    id,
    task_id,
    url,
    token,
    authentication,
    created_at,
    updated_at
) VALUES (
    ?1,
    ?2,
    ?3,
    ?4,
    ?5,
    ?6,
    ?7
)"#, libsql::params![
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.task_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.url.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              match params.token.clone() {
                Some(value) => {
                  <String as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
              match params.authentication.clone() {
                Some(value) => {
                  <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(value.clone())
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
  pub struct update_push_notification_config_params<'a> {
      pub url: &'a 
          String
      ,
      pub token: &'a Option<
          String
      >,
      pub authentication: &'a Option<
          shared::primitives::WrappedJsonValue
      >,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
  }

  pub async fn update_push_notification_config(
    conn: &shared::libsql::Connection
    ,params: update_push_notification_config_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"UPDATE push_notification_config SET
    url = ?1,
    token = ?2,
    authentication = ?3,
    updated_at = ?4
WHERE id = ?5"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.url.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              match params.token.clone() {
                Some(value) => {
                  <String as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
              match params.authentication.clone() {
                Some(value) => {
                  <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(params.updated_at.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_push_notification_configs_by_task_id_params<'a> {
      pub task_id: &'a 
          shared::primitives::WrappedUuidV4
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_push_notification_configs_by_task_id {
      pub id:shared::primitives::WrappedUuidV4,
      pub task_id:shared::primitives::WrappedUuidV4,
      pub url:String,
      pub token:Option<String> ,
      pub authentication:Option<shared::primitives::WrappedJsonValue> ,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_push_notification_configs_by_task_id(
      conn: &shared::libsql::Connection
      ,params: get_push_notification_configs_by_task_id_params<'_>
  ) -> Result<Vec<Row_get_push_notification_configs_by_task_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, task_id, url, token, authentication, created_at, updated_at FROM push_notification_config WHERE task_id = ?1"#).await?;
      let mut rows = stmt.query(libsql::params![params.task_id.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_push_notification_configs_by_task_id {
              id: row.get(0)?,
              task_id: row.get(1)?,
              url: row.get(2)?,
              token: row.get(3)?,
              authentication: row.get(4)?,
              created_at: row.get(5)?,
              updated_at: row.get(6)?,
          });
      }

      Ok(mapped)
  }
  pub struct get_push_notification_config_by_id_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_push_notification_config_by_id {
      pub id:shared::primitives::WrappedUuidV4,
      pub task_id:shared::primitives::WrappedUuidV4,
      pub url:String,
      pub token:Option<String> ,
      pub authentication:Option<shared::primitives::WrappedJsonValue> ,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_push_notification_config_by_id(
      conn: &shared::libsql::Connection
      ,params: get_push_notification_config_by_id_params<'_>
  ) -> Result<Option<Row_get_push_notification_config_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, task_id, url, token, authentication, created_at, updated_at FROM push_notification_config WHERE id = ?1"#).await?;
      let res = stmt.query_row(
          libsql::params![params.id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_push_notification_config_by_id {
                  id: row.get(0)?,
                  task_id: row.get(1)?,
                  url: row.get(2)?,
                  token: row.get(3)?,
                  authentication: row.get(4)?,
                  created_at: row.get(5)?,
                  updated_at: row.get(6)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct delete_push_notification_config_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
  }

  pub async fn delete_push_notification_config(
    conn: &shared::libsql::Connection
    ,params: delete_push_notification_config_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM push_notification_config WHERE id = ?1"#, libsql::params![
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct delete_push_notification_configs_by_task_id_params<'a> {
      pub task_id: &'a 
          shared::primitives::WrappedUuidV4
      ,
  }

  pub async fn delete_push_notification_configs_by_task_id(
    conn: &shared::libsql::Connection
    ,params: delete_push_notification_configs_by_task_id_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM push_notification_config WHERE task_id = ?1"#, libsql::params![
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.task_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct insert_task_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
      pub context_id: &'a 
          shared::primitives::WrappedUuidV4
      ,
      pub status: &'a 
          crate::logic::task::TaskStatus
      ,
      pub status_timestamp: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub metadata: &'a 
          shared::primitives::WrappedJsonValue
      ,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
  }

  pub async fn insert_task(
    conn: &shared::libsql::Connection
    ,params: insert_task_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO task (
    id,
    context_id,
    status,
    status_timestamp,
    metadata,
    created_at,
    updated_at
) VALUES (
    ?1,
    ?2,
    ?3,
    ?4,
    ?5,
    ?6,
    ?7
)"#, libsql::params![
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.context_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <crate::logic::task::TaskStatus as TryInto<libsql::Value>>::try_into(params.status.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(params.status_timestamp.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.metadata.clone())
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
  pub struct update_task_status_params<'a> {
      pub status: &'a 
          crate::logic::task::TaskStatus
      ,
      pub status_timestamp: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
  }

  pub async fn update_task_status(
    conn: &shared::libsql::Connection
    ,params: update_task_status_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"UPDATE task SET status = ?1, status_timestamp = ?2, updated_at = ?3 WHERE id = ?4"#, libsql::params![
              <crate::logic::task::TaskStatus as TryInto<libsql::Value>>::try_into(params.status.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(params.status_timestamp.clone())
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
  pub struct insert_task_timeline_item_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
      pub task_id: &'a 
          shared::primitives::WrappedUuidV4
      ,
      pub event_update_type: &'a 
          crate::logic::task::TaskEventUpdateType
      ,
      pub event_payload: &'a 
          shared::primitives::WrappedJsonValue
      ,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
  }

  pub async fn insert_task_timeline_item(
    conn: &shared::libsql::Connection
    ,params: insert_task_timeline_item_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO task_timeline (
    id,
    task_id,
    event_update_type,
    event_payload,
    created_at
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
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.task_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <crate::logic::task::TaskEventUpdateType as TryInto<libsql::Value>>::try_into(params.event_update_type.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.event_payload.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(params.created_at.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_tasks_params<'a> {
      pub cursor: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub page_size: &'a 
          i64
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_tasks {
      pub id:shared::primitives::WrappedUuidV4,
      pub context_id:shared::primitives::WrappedUuidV4,
      pub status:crate::logic::task::TaskStatus,
      pub status_timestamp:shared::primitives::WrappedChronoDateTime,
      pub metadata:shared::primitives::WrappedJsonValue,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_tasks(
      conn: &shared::libsql::Connection
      ,params: get_tasks_params<'_>
  ) -> Result<Vec<Row_get_tasks>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, context_id, status, status_timestamp, metadata, created_at, updated_at FROM task WHERE (created_at < ?1 OR ?1 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?2 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_tasks {
              id: row.get(0)?,
              context_id: row.get(1)?,
              status: row.get(2)?,
              status_timestamp: row.get(3)?,
              metadata: row.get(4)?,
              created_at: row.get(5)?,
              updated_at: row.get(6)?,
          });
      }

      Ok(mapped)
  }
  pub struct get_unique_contexts_params<'a> {
      pub cursor: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub page_size: &'a 
          i64
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_unique_contexts {
      pub context_id:shared::primitives::WrappedUuidV4,
      pub created_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_unique_contexts(
      conn: &shared::libsql::Connection
      ,params: get_unique_contexts_params<'_>
  ) -> Result<Vec<Row_get_unique_contexts>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT DISTINCT context_id, created_at FROM task WHERE (created_at < ?1 OR ?1 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?2 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_unique_contexts {
              context_id: row.get(0)?,
              created_at: row.get(1)?,
          });
      }

      Ok(mapped)
  }
  pub struct get_tasks_by_context_id_params<'a> {
      pub context_id: &'a 
          shared::primitives::WrappedUuidV4
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
  pub struct Row_get_tasks_by_context_id {
      pub id:shared::primitives::WrappedUuidV4,
      pub context_id:shared::primitives::WrappedUuidV4,
      pub status:crate::logic::task::TaskStatus,
      pub status_timestamp:shared::primitives::WrappedChronoDateTime,
      pub metadata:shared::primitives::WrappedJsonValue,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_tasks_by_context_id(
      conn: &shared::libsql::Connection
      ,params: get_tasks_by_context_id_params<'_>
  ) -> Result<Vec<Row_get_tasks_by_context_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, context_id, status, status_timestamp, metadata, created_at, updated_at FROM task WHERE context_id = ?1 AND (created_at < ?2 OR ?2 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?3 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.context_id.clone(),params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_tasks_by_context_id {
              id: row.get(0)?,
              context_id: row.get(1)?,
              status: row.get(2)?,
              status_timestamp: row.get(3)?,
              metadata: row.get(4)?,
              created_at: row.get(5)?,
              updated_at: row.get(6)?,
          });
      }

      Ok(mapped)
  }
  pub struct get_task_timeline_items_params<'a> {
      pub task_id: &'a 
          shared::primitives::WrappedUuidV4
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
  pub struct Row_get_task_timeline_items {
      pub id:shared::primitives::WrappedUuidV4,
      pub task_id:shared::primitives::WrappedUuidV4,
      pub event_update_type:crate::logic::task::TaskEventUpdateType,
      pub event_payload:shared::primitives::WrappedJsonValue,
      pub created_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_task_timeline_items(
      conn: &shared::libsql::Connection
      ,params: get_task_timeline_items_params<'_>
  ) -> Result<Vec<Row_get_task_timeline_items>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, task_id, event_update_type, event_payload, created_at FROM task_timeline WHERE task_id = ?1 AND (created_at < ?2 OR ?2 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?3 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.task_id.clone(),params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_task_timeline_items {
              id: row.get(0)?,
              task_id: row.get(1)?,
              event_update_type: row.get(2)?,
              event_payload: row.get(3)?,
              created_at: row.get(4)?,
          });
      }

      Ok(mapped)
  }
  pub struct get_task_by_id_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_task_by_id {
      pub id:shared::primitives::WrappedUuidV4,
      pub context_id:shared::primitives::WrappedUuidV4,
      pub status:crate::logic::task::TaskStatus,
      pub status_timestamp:shared::primitives::WrappedChronoDateTime,
      pub metadata:shared::primitives::WrappedJsonValue,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_task_by_id(
      conn: &shared::libsql::Connection
      ,params: get_task_by_id_params<'_>
  ) -> Result<Option<Row_get_task_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, context_id, status, status_timestamp, metadata, created_at, updated_at FROM task WHERE id = ?1"#).await?;
      let res = stmt.query_row(
          libsql::params![params.id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_task_by_id {
                  id: row.get(0)?,
                  context_id: row.get(1)?,
                  status: row.get(2)?,
                  status_timestamp: row.get(3)?,
                  metadata: row.get(4)?,
                  created_at: row.get(5)?,
                  updated_at: row.get(6)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
