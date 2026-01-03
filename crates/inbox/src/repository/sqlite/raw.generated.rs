

#[allow(unused)]
use serde::{Serialize, Deserialize};
  pub struct insert_event_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
      pub kind: &'a 
          String
      ,
      pub payload: &'a 
          shared::primitives::WrappedJsonValue
      ,
      pub inbox_id: &'a Option<
          String
      >,
      pub inbox_settings: &'a 
          shared::primitives::WrappedJsonValue
      ,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
  }

  pub async fn insert_event(
    conn: &shared::libsql::Connection
    ,params: insert_event_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO event (
    id,
    kind,
    payload,
    inbox_id,
    inbox_settings,
    created_at
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
              <String as TryInto<libsql::Value>>::try_into(params.kind.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.payload.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              match params.inbox_id.clone() {
                Some(value) => {
                  <String as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.inbox_settings.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(params.created_at.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_event_by_id_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_event_by_id {
      pub id:shared::primitives::WrappedUuidV4,
      pub kind:String,
      pub payload:shared::primitives::WrappedJsonValue,
      pub inbox_id:Option<String> ,
      pub inbox_settings:shared::primitives::WrappedJsonValue,
      pub created_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_event_by_id(
      conn: &shared::libsql::Connection
      ,params: get_event_by_id_params<'_>
  ) -> Result<Option<Row_get_event_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, kind, payload, inbox_id, inbox_settings, created_at
FROM event WHERE id = ?1"#).await?;
      let res = stmt.query_row(
          libsql::params![params.id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_event_by_id {
                  id: row.get(0)?,
                  kind: row.get(1)?,
                  payload: row.get(2)?,
                  inbox_id: row.get(3)?,
                  inbox_settings: row.get(4)?,
                  created_at: row.get(5)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct get_events_params<'a> {
      pub cursor: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub page_size: &'a 
          i64
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_events {
      pub id:shared::primitives::WrappedUuidV4,
      pub kind:String,
      pub payload:shared::primitives::WrappedJsonValue,
      pub inbox_id:Option<String> ,
      pub inbox_settings:shared::primitives::WrappedJsonValue,
      pub created_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_events(
      conn: &shared::libsql::Connection
      ,params: get_events_params<'_>
  ) -> Result<Vec<Row_get_events>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, kind, payload, inbox_id, inbox_settings, created_at
FROM event
WHERE (created_at < ?1 OR ?1 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?2 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_events {
              id: row.get(0)?,
              kind: row.get(1)?,
              payload: row.get(2)?,
              inbox_id: row.get(3)?,
              inbox_settings: row.get(4)?,
              created_at: row.get(5)?,
          });
      }

      Ok(mapped)
  }
  pub struct get_events_by_inbox_params<'a> {
      pub inbox_id: &'a Option<
          String
      >,
      pub cursor: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub page_size: &'a 
          i64
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_events_by_inbox {
      pub id:shared::primitives::WrappedUuidV4,
      pub kind:String,
      pub payload:shared::primitives::WrappedJsonValue,
      pub inbox_id:Option<String> ,
      pub inbox_settings:shared::primitives::WrappedJsonValue,
      pub created_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_events_by_inbox(
      conn: &shared::libsql::Connection
      ,params: get_events_by_inbox_params<'_>
  ) -> Result<Vec<Row_get_events_by_inbox>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, kind, payload, inbox_id, inbox_settings, created_at
FROM event
WHERE inbox_id = ?1
  AND (created_at < ?2 OR ?2 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?3 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.inbox_id.clone(),params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_events_by_inbox {
              id: row.get(0)?,
              kind: row.get(1)?,
              payload: row.get(2)?,
              inbox_id: row.get(3)?,
              inbox_settings: row.get(4)?,
              created_at: row.get(5)?,
          });
      }

      Ok(mapped)
  }
  pub struct get_events_by_kind_params<'a> {
      pub kind: &'a 
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
  pub struct Row_get_events_by_kind {
      pub id:shared::primitives::WrappedUuidV4,
      pub kind:String,
      pub payload:shared::primitives::WrappedJsonValue,
      pub inbox_id:Option<String> ,
      pub inbox_settings:shared::primitives::WrappedJsonValue,
      pub created_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_events_by_kind(
      conn: &shared::libsql::Connection
      ,params: get_events_by_kind_params<'_>
  ) -> Result<Vec<Row_get_events_by_kind>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, kind, payload, inbox_id, inbox_settings, created_at
FROM event
WHERE kind = ?1
  AND (created_at < ?2 OR ?2 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?3 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.kind.clone(),params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_events_by_kind {
              id: row.get(0)?,
              kind: row.get(1)?,
              payload: row.get(2)?,
              inbox_id: row.get(3)?,
              inbox_settings: row.get(4)?,
              created_at: row.get(5)?,
          });
      }

      Ok(mapped)
  }
  pub struct delete_events_before_params<'a> {
      pub before_date: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
  }

  pub async fn delete_events_before(
    conn: &shared::libsql::Connection
    ,params: delete_events_before_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM event WHERE created_at < ?1"#, libsql::params![
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(params.before_date.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct insert_inbox_params<'a> {
      pub id: &'a 
          String
      ,
      pub provider_id: &'a 
          String
      ,
      pub status: &'a 
          crate::logic::inbox::InboxStatus
      ,
      pub configuration: &'a 
          shared::primitives::WrappedJsonValue
      ,
      pub settings: &'a 
          shared::primitives::WrappedJsonValue
      ,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
  }

  pub async fn insert_inbox(
    conn: &shared::libsql::Connection
    ,params: insert_inbox_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO inbox (
    id,
    provider_id,
    status,
    configuration,
    settings,
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
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.provider_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <crate::logic::inbox::InboxStatus as TryInto<libsql::Value>>::try_into(params.status.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.configuration.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.settings.clone())
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
  pub struct update_inbox_params<'a> {
      pub configuration: &'a 
          shared::primitives::WrappedJsonValue
      ,
      pub settings: &'a 
          shared::primitives::WrappedJsonValue
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub id: &'a 
          String
      ,
  }

  pub async fn update_inbox(
    conn: &shared::libsql::Connection
    ,params: update_inbox_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"UPDATE inbox SET
    configuration = ?1,
    settings = ?2,
    updated_at = ?3
WHERE id = ?4"#, libsql::params![
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.configuration.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.settings.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(params.updated_at.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct update_inbox_status_params<'a> {
      pub status: &'a 
          crate::logic::inbox::InboxStatus
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub id: &'a 
          String
      ,
  }

  pub async fn update_inbox_status(
    conn: &shared::libsql::Connection
    ,params: update_inbox_status_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"UPDATE inbox SET
    status = ?1,
    updated_at = ?2
WHERE id = ?3"#, libsql::params![
              <crate::logic::inbox::InboxStatus as TryInto<libsql::Value>>::try_into(params.status.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(params.updated_at.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct delete_inbox_params<'a> {
      pub id: &'a 
          String
      ,
  }

  pub async fn delete_inbox(
    conn: &shared::libsql::Connection
    ,params: delete_inbox_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM inbox WHERE id = ?1"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_inbox_by_id_params<'a> {
      pub id: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_inbox_by_id {
      pub id:String,
      pub provider_id:String,
      pub status:crate::logic::inbox::InboxStatus,
      pub configuration:shared::primitives::WrappedJsonValue,
      pub settings:shared::primitives::WrappedJsonValue,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_inbox_by_id(
      conn: &shared::libsql::Connection
      ,params: get_inbox_by_id_params<'_>
  ) -> Result<Option<Row_get_inbox_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, provider_id, status, configuration, settings, created_at, updated_at
FROM inbox WHERE id = ?1"#).await?;
      let res = stmt.query_row(
          libsql::params![params.id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_inbox_by_id {
                  id: row.get(0)?,
                  provider_id: row.get(1)?,
                  status: row.get(2)?,
                  configuration: row.get(3)?,
                  settings: row.get(4)?,
                  created_at: row.get(5)?,
                  updated_at: row.get(6)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct get_inboxes_params<'a> {
      pub cursor: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub page_size: &'a 
          i64
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_inboxes {
      pub id:String,
      pub provider_id:String,
      pub status:crate::logic::inbox::InboxStatus,
      pub configuration:shared::primitives::WrappedJsonValue,
      pub settings:shared::primitives::WrappedJsonValue,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_inboxes(
      conn: &shared::libsql::Connection
      ,params: get_inboxes_params<'_>
  ) -> Result<Vec<Row_get_inboxes>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, provider_id, status, configuration, settings, created_at, updated_at
FROM inbox
WHERE (created_at < ?1 OR ?1 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?2 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_inboxes {
              id: row.get(0)?,
              provider_id: row.get(1)?,
              status: row.get(2)?,
              configuration: row.get(3)?,
              settings: row.get(4)?,
              created_at: row.get(5)?,
              updated_at: row.get(6)?,
          });
      }

      Ok(mapped)
  }
  pub struct get_inboxes_by_provider_params<'a> {
      pub provider_id: &'a 
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
  pub struct Row_get_inboxes_by_provider {
      pub id:String,
      pub provider_id:String,
      pub status:crate::logic::inbox::InboxStatus,
      pub configuration:shared::primitives::WrappedJsonValue,
      pub settings:shared::primitives::WrappedJsonValue,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_inboxes_by_provider(
      conn: &shared::libsql::Connection
      ,params: get_inboxes_by_provider_params<'_>
  ) -> Result<Vec<Row_get_inboxes_by_provider>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, provider_id, status, configuration, settings, created_at, updated_at
FROM inbox
WHERE provider_id = ?1
  AND (created_at < ?2 OR ?2 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?3 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.provider_id.clone(),params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_inboxes_by_provider {
              id: row.get(0)?,
              provider_id: row.get(1)?,
              status: row.get(2)?,
              configuration: row.get(3)?,
              settings: row.get(4)?,
              created_at: row.get(5)?,
              updated_at: row.get(6)?,
          });
      }

      Ok(mapped)
  }
  pub struct get_enabled_inboxes_params<'a> {
      pub cursor: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub page_size: &'a 
          i64
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_enabled_inboxes {
      pub id:String,
      pub provider_id:String,
      pub status:crate::logic::inbox::InboxStatus,
      pub configuration:shared::primitives::WrappedJsonValue,
      pub settings:shared::primitives::WrappedJsonValue,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_enabled_inboxes(
      conn: &shared::libsql::Connection
      ,params: get_enabled_inboxes_params<'_>
  ) -> Result<Vec<Row_get_enabled_inboxes>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, provider_id, status, configuration, settings, created_at, updated_at
FROM inbox
WHERE status = 'enabled'
  AND (created_at < ?1 OR ?1 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?2 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_enabled_inboxes {
              id: row.get(0)?,
              provider_id: row.get(1)?,
              status: row.get(2)?,
              configuration: row.get(3)?,
              settings: row.get(4)?,
              created_at: row.get(5)?,
              updated_at: row.get(6)?,
          });
      }

      Ok(mapped)
  }
  pub struct insert_message_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
      pub thread_id: &'a 
          shared::primitives::WrappedUuidV4
      ,
      pub role: &'a 
          crate::logic::message::MessageRole
      ,
      pub parts: &'a 
          shared::primitives::WrappedJsonValue
      ,
      pub metadata: &'a Option<
          shared::primitives::WrappedJsonValue
      >,
      pub inbox_settings: &'a 
          shared::primitives::WrappedJsonValue
      ,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
  }

  pub async fn insert_message(
    conn: &shared::libsql::Connection
    ,params: insert_message_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO message (
    id,
    thread_id,
    role,
    parts,
    metadata,
    inbox_settings,
    created_at,
    updated_at
) VALUES (
    ?1,
    ?2,
    ?3,
    ?4,
    ?5,
    ?6,
    ?7,
    ?8
)"#, libsql::params![
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.thread_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <crate::logic::message::MessageRole as TryInto<libsql::Value>>::try_into(params.role.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.parts.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              match params.metadata.clone() {
                Some(value) => {
                  <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.inbox_settings.clone())
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
  pub struct update_message_params<'a> {
      pub parts: &'a 
          shared::primitives::WrappedJsonValue
      ,
      pub metadata: &'a Option<
          shared::primitives::WrappedJsonValue
      >,
      pub inbox_settings: &'a 
          shared::primitives::WrappedJsonValue
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
  }

  pub async fn update_message(
    conn: &shared::libsql::Connection
    ,params: update_message_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"UPDATE message SET
    parts = ?1,
    metadata = ?2,
    inbox_settings = ?3,
    updated_at = ?4
WHERE id = ?5"#, libsql::params![
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.parts.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              match params.metadata.clone() {
                Some(value) => {
                  <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.inbox_settings.clone())
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
  pub struct delete_message_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
  }

  pub async fn delete_message(
    conn: &shared::libsql::Connection
    ,params: delete_message_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM message WHERE id = ?1"#, libsql::params![
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_message_by_id_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_message_by_id {
      pub id:shared::primitives::WrappedUuidV4,
      pub thread_id:shared::primitives::WrappedUuidV4,
      pub role:crate::logic::message::MessageRole,
      pub parts:shared::primitives::WrappedJsonValue,
      pub metadata:Option<shared::primitives::WrappedJsonValue> ,
      pub inbox_settings:shared::primitives::WrappedJsonValue,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_message_by_id(
      conn: &shared::libsql::Connection
      ,params: get_message_by_id_params<'_>
  ) -> Result<Option<Row_get_message_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, thread_id, role, parts, metadata, inbox_settings, created_at, updated_at
FROM message WHERE id = ?1"#).await?;
      let res = stmt.query_row(
          libsql::params![params.id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_message_by_id {
                  id: row.get(0)?,
                  thread_id: row.get(1)?,
                  role: row.get(2)?,
                  parts: row.get(3)?,
                  metadata: row.get(4)?,
                  inbox_settings: row.get(5)?,
                  created_at: row.get(6)?,
                  updated_at: row.get(7)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct get_messages_params<'a> {
      pub cursor: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub page_size: &'a 
          i64
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_messages {
      pub id:shared::primitives::WrappedUuidV4,
      pub thread_id:shared::primitives::WrappedUuidV4,
      pub role:crate::logic::message::MessageRole,
      pub parts:shared::primitives::WrappedJsonValue,
      pub metadata:Option<shared::primitives::WrappedJsonValue> ,
      pub inbox_settings:shared::primitives::WrappedJsonValue,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_messages(
      conn: &shared::libsql::Connection
      ,params: get_messages_params<'_>
  ) -> Result<Vec<Row_get_messages>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, thread_id, role, parts, metadata, inbox_settings, created_at, updated_at
FROM message
WHERE (created_at < ?1 OR ?1 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?2 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_messages {
              id: row.get(0)?,
              thread_id: row.get(1)?,
              role: row.get(2)?,
              parts: row.get(3)?,
              metadata: row.get(4)?,
              inbox_settings: row.get(5)?,
              created_at: row.get(6)?,
              updated_at: row.get(7)?,
          });
      }

      Ok(mapped)
  }
  pub struct get_messages_by_thread_params<'a> {
      pub thread_id: &'a 
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
  pub struct Row_get_messages_by_thread {
      pub id:shared::primitives::WrappedUuidV4,
      pub thread_id:shared::primitives::WrappedUuidV4,
      pub role:crate::logic::message::MessageRole,
      pub parts:shared::primitives::WrappedJsonValue,
      pub metadata:Option<shared::primitives::WrappedJsonValue> ,
      pub inbox_settings:shared::primitives::WrappedJsonValue,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_messages_by_thread(
      conn: &shared::libsql::Connection
      ,params: get_messages_by_thread_params<'_>
  ) -> Result<Vec<Row_get_messages_by_thread>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, thread_id, role, parts, metadata, inbox_settings, created_at, updated_at
FROM message
WHERE thread_id = ?1
  AND (created_at < ?2 OR ?2 IS NULL)
ORDER BY created_at ASC
LIMIT CAST(?3 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.thread_id.clone(),params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_messages_by_thread {
              id: row.get(0)?,
              thread_id: row.get(1)?,
              role: row.get(2)?,
              parts: row.get(3)?,
              metadata: row.get(4)?,
              inbox_settings: row.get(5)?,
              created_at: row.get(6)?,
              updated_at: row.get(7)?,
          });
      }

      Ok(mapped)
  }
  pub struct delete_messages_by_thread_params<'a> {
      pub thread_id: &'a 
          shared::primitives::WrappedUuidV4
      ,
  }

  pub async fn delete_messages_by_thread(
    conn: &shared::libsql::Connection
    ,params: delete_messages_by_thread_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM message WHERE thread_id = ?1"#, libsql::params![
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.thread_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct insert_thread_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
      pub title: &'a Option<
          String
      >,
      pub metadata: &'a Option<
          shared::primitives::WrappedJsonValue
      >,
      pub inbox_settings: &'a 
          shared::primitives::WrappedJsonValue
      ,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
  }

  pub async fn insert_thread(
    conn: &shared::libsql::Connection
    ,params: insert_thread_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO thread (
    id,
    title,
    metadata,
    inbox_settings,
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
              match params.title.clone() {
                Some(value) => {
                  <String as TryInto<libsql::Value>>::try_into(value.clone())
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
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.inbox_settings.clone())
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
  pub struct update_thread_params<'a> {
      pub title: &'a Option<
          String
      >,
      pub metadata: &'a Option<
          shared::primitives::WrappedJsonValue
      >,
      pub inbox_settings: &'a 
          shared::primitives::WrappedJsonValue
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
  }

  pub async fn update_thread(
    conn: &shared::libsql::Connection
    ,params: update_thread_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"UPDATE thread SET
    title = ?1,
    metadata = ?2,
    inbox_settings = ?3,
    updated_at = ?4
WHERE id = ?5"#, libsql::params![
              match params.title.clone() {
                Some(value) => {
                  <String as TryInto<libsql::Value>>::try_into(value.clone())
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
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.inbox_settings.clone())
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
  pub struct delete_thread_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
  }

  pub async fn delete_thread(
    conn: &shared::libsql::Connection
    ,params: delete_thread_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM thread WHERE id = ?1"#, libsql::params![
              <shared::primitives::WrappedUuidV4 as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_thread_by_id_params<'a> {
      pub id: &'a 
          shared::primitives::WrappedUuidV4
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_thread_by_id {
      pub id:shared::primitives::WrappedUuidV4,
      pub title:Option<String> ,
      pub metadata:Option<shared::primitives::WrappedJsonValue> ,
      pub inbox_settings:shared::primitives::WrappedJsonValue,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_thread_by_id(
      conn: &shared::libsql::Connection
      ,params: get_thread_by_id_params<'_>
  ) -> Result<Option<Row_get_thread_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, title, metadata, inbox_settings, created_at, updated_at
FROM thread WHERE id = ?1"#).await?;
      let res = stmt.query_row(
          libsql::params![params.id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_thread_by_id {
                  id: row.get(0)?,
                  title: row.get(1)?,
                  metadata: row.get(2)?,
                  inbox_settings: row.get(3)?,
                  created_at: row.get(4)?,
                  updated_at: row.get(5)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct get_threads_params<'a> {
      pub cursor: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub page_size: &'a 
          i64
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_threads {
      pub id:shared::primitives::WrappedUuidV4,
      pub title:Option<String> ,
      pub metadata:Option<shared::primitives::WrappedJsonValue> ,
      pub inbox_settings:shared::primitives::WrappedJsonValue,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_threads(
      conn: &shared::libsql::Connection
      ,params: get_threads_params<'_>
  ) -> Result<Vec<Row_get_threads>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, title, metadata, inbox_settings, created_at, updated_at
FROM thread
WHERE (created_at < ?1 OR ?1 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?2 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_threads {
              id: row.get(0)?,
              title: row.get(1)?,
              metadata: row.get(2)?,
              inbox_settings: row.get(3)?,
              created_at: row.get(4)?,
              updated_at: row.get(5)?,
          });
      }

      Ok(mapped)
  }
