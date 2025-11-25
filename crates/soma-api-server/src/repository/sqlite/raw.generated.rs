

#[allow(unused)]
use serde::{Serialize, Deserialize};
  pub struct insert_message_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
      pub task_id: &'a 
          shared::primitives::WrappedUuidV4
      ,
      pub reference_task_ids: &'a 
          shared::primitives::WrappedJsonValue
      ,
      pub role: &'a 
          crate::logic::task::MessageRole
      ,
      pub metadata: &'a 
          shared::primitives::WrappedJsonValue
      ,
      pub parts: &'a 
          shared::primitives::WrappedJsonValue
      ,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
  }

  pub async fn insert_message(
    conn: &shared::libsql::Connection
    ,params: insert_message_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO message (
    id,
    task_id,
    reference_task_ids,
    role,
    metadata,
    parts,
    created_at
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
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.reference_task_ids.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <crate::logic::task::MessageRole as TryInto<libsql::Value>>::try_into(params.role.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.metadata.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.parts.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(*params.created_at)
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  #[allow(dead_code)]
  pub struct get_messages_by_task_id_params<'a> {
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
  pub struct Row_get_messages_by_task_id {
      pub id:shared::primitives::WrappedUuidV4,
      pub task_id:shared::primitives::WrappedUuidV4,
      pub reference_task_ids:shared::primitives::WrappedJsonValue,
      pub role:crate::logic::task::MessageRole,
      pub metadata:shared::primitives::WrappedJsonValue,
      pub parts:shared::primitives::WrappedJsonValue,
      pub created_at:shared::primitives::WrappedChronoDateTime,
  }
  #[allow(dead_code)]
  pub async fn get_messages_by_task_id(
      conn: &shared::libsql::Connection
      ,params: get_messages_by_task_id_params<'_>
  ) -> Result<Vec<Row_get_messages_by_task_id>, libsql::Error> {
      let stmt = conn.prepare(r#"SELECT id, task_id, reference_task_ids, role, metadata, parts, created_at FROM message WHERE task_id = ?1 AND (created_at < ?2 OR ?2 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?3 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.task_id.clone(),(*params.cursor),(*params.page_size),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_messages_by_task_id {
              id: row.get(0)?,
              task_id: row.get(1)?,
              reference_task_ids: row.get(2)?,
              role: row.get(3)?,
              metadata: row.get(4)?,
              parts: row.get(5)?,
              created_at: row.get(6)?,
          });
      }

      Ok(mapped)
  }
  pub struct insert_task_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
      pub context_id: &'a 
          shared::primitives::WrappedUuidV4
      ,
      pub status: &'a 
          crate::repository::TaskStatus
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
              <crate::repository::TaskStatus as TryInto<libsql::Value>>::try_into(params.status.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(*params.status_timestamp)
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.metadata.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(*params.created_at)
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(*params.updated_at)
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct update_task_status_params<'a> {
      pub status: &'a 
          crate::repository::TaskStatus
      ,
      pub status_message_id: &'a Option<
          shared::primitives::WrappedUuidV4
      >,
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
    conn.execute(r#"UPDATE task SET status = ?1, status_message_id = ?2, status_timestamp = ?3, updated_at = ?4 WHERE id = ?5"#, libsql::params![
              <crate::repository::TaskStatus as TryInto<libsql::Value>>::try_into(params.status.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              match params.status_message_id.clone() {
                Some(value) => {
                  <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(*params.status_timestamp)
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(*params.updated_at)
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct insert_task_timeline_item_params<'a> {
      pub id: &'a 
          String
      ,
      pub task_id: &'a 
          String
      ,
      pub event_update_type: &'a 
          String
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
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.task_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.event_update_type.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.event_payload.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(*params.created_at)
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
      pub status:crate::repository::TaskStatus,
      pub status_message_id:Option<shared::primitives::WrappedUuidV4> ,
      pub status_timestamp:shared::primitives::WrappedChronoDateTime,
      pub metadata:shared::primitives::WrappedJsonValue,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_tasks(
      conn: &shared::libsql::Connection
      ,params: get_tasks_params<'_>
  ) -> Result<Vec<Row_get_tasks>, libsql::Error> {
      let stmt = conn.prepare(r#"SELECT id, context_id, status, status_message_id, status_timestamp, metadata, created_at, updated_at FROM task WHERE (created_at < ?1 OR ?1 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?2 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![(*params.cursor),(*params.page_size),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_tasks {
              id: row.get(0)?,
              context_id: row.get(1)?,
              status: row.get(2)?,
              status_message_id: row.get(3)?,
              status_timestamp: row.get(4)?,
              metadata: row.get(5)?,
              created_at: row.get(6)?,
              updated_at: row.get(7)?,
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
      let stmt = conn.prepare(r#"SELECT DISTINCT context_id, created_at FROM task WHERE (created_at < ?1 OR ?1 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?2 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![(*params.cursor),(*params.page_size),]).await?;
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
      pub status:crate::repository::TaskStatus,
      pub status_message_id:Option<shared::primitives::WrappedUuidV4> ,
      pub status_timestamp:shared::primitives::WrappedChronoDateTime,
      pub metadata:shared::primitives::WrappedJsonValue,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_tasks_by_context_id(
      conn: &shared::libsql::Connection
      ,params: get_tasks_by_context_id_params<'_>
  ) -> Result<Vec<Row_get_tasks_by_context_id>, libsql::Error> {
      let stmt = conn.prepare(r#"SELECT id, context_id, status, status_message_id, status_timestamp, metadata, created_at, updated_at FROM task WHERE context_id = ?1 AND (created_at < ?2 OR ?2 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?3 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.context_id.clone(),(*params.cursor),(*params.page_size),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_tasks_by_context_id {
              id: row.get(0)?,
              context_id: row.get(1)?,
              status: row.get(2)?,
              status_message_id: row.get(3)?,
              status_timestamp: row.get(4)?,
              metadata: row.get(5)?,
              created_at: row.get(6)?,
              updated_at: row.get(7)?,
          });
      }

      Ok(mapped)
  }
  pub struct get_task_timeline_items_params<'a> {
      pub task_id: &'a 
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
  pub struct Row_get_task_timeline_items {
      pub id:String,
      pub task_id:String,
      pub event_update_type:String,
      pub event_payload:shared::primitives::WrappedJsonValue,
      pub created_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_task_timeline_items(
      conn: &shared::libsql::Connection
      ,params: get_task_timeline_items_params<'_>
  ) -> Result<Vec<Row_get_task_timeline_items>, libsql::Error> {
      let stmt = conn.prepare(r#"SELECT id, task_id, event_update_type, event_payload, created_at FROM task_timeline WHERE task_id = ?1 AND (created_at < ?2 OR ?2 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?3 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.task_id.clone(),(*params.cursor),(*params.page_size),]).await?;
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
      pub status:crate::repository::TaskStatus,
      pub status_message_id:Option<shared::primitives::WrappedUuidV4> ,
      pub status_timestamp:shared::primitives::WrappedChronoDateTime,
      pub metadata:shared::primitives::WrappedJsonValue,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
      pub status_message:String,
      pub messages:String,
  }
  pub async fn get_task_by_id(
      conn: &shared::libsql::Connection
      ,params: get_task_by_id_params<'_>
  ) -> Result<Option<Row_get_task_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT
    t.id,
    t.context_id,
    t.status,
    t.status_message_id,
    t.status_timestamp,
    t.metadata,
    t.created_at,
    t.updated_at,
    CAST(
        CASE
            WHEN sm.id IS NULL THEN JSON('[]')
            ELSE JSON_ARRAY(
                JSON_OBJECT(
                    'id', sm.id,
                    'task_id', sm.task_id,
                    'reference_task_ids', JSON(sm.reference_task_ids),
                    'role', sm.role,
                    'metadata', JSON(sm.metadata),
                    'parts', JSON(sm.parts),
                    'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', sm.created_at)
                )
            )
        END AS TEXT
    ) AS status_message,
    (
        SELECT CAST(
            CASE
                WHEN COUNT(m2.id) = 0 THEN JSON('[]')
                ELSE JSON_GROUP_ARRAY(
                    JSON_OBJECT(
                        'id', m2.id,
                        'task_id', m2.task_id,
                        'reference_task_ids', JSON(m2.reference_task_ids),
                        'role', m2.role,
                        'metadata', JSON(m2.metadata),
                        'parts', JSON(m2.parts),
                        'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', m2.created_at)
                    )
                )
            END AS TEXT
        )
        FROM message m2
        WHERE m2.task_id = t.id
        ORDER BY m2.created_at DESC
    ) AS messages
FROM task t
LEFT JOIN message sm ON t.status_message_id = sm.id
WHERE t.id = ?1"#).await?;
      let res = stmt.query_row(
          libsql::params![params.id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_task_by_id {
                  id: row.get(0)?,
                  context_id: row.get(1)?,
                  status: row.get(2)?,
                  status_message_id: row.get(3)?,
                  status_timestamp: row.get(4)?,
                  metadata: row.get(5)?,
                  created_at: row.get(6)?,
                  updated_at: row.get(7)?,
                  status_message: row.get(8)?,
                  messages: row.get(9)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
