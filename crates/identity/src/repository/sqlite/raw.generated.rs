

#[allow(unused)]
use serde::{Serialize, Deserialize};
//  ============================================================================
//  API key table queries
//  ============================================================================
  pub struct create_api_key_params<'a> {
      pub id: &'a 
          String
      ,
      pub hashed_value: &'a 
          String
      ,
      pub description: &'a Option<
          String
      >,
      pub user_id: &'a 
          String
      ,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
  }

  pub async fn create_api_key(
    conn: &shared::libsql::Connection
    ,params: create_api_key_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"
INSERT INTO api_key (id, hashed_value, description, user_id, created_at, updated_at)
VALUES (?, ?, ?, ?, ?, ?)"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.hashed_value.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              match params.description.clone() {
                Some(value) => {
                  <String as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
              <String as TryInto<libsql::Value>>::try_into(params.user_id.clone())
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
  pub struct get_api_key_by_hashed_value_params<'a> {
      pub hashed_value: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_api_key_by_hashed_value {
      pub id:String,
      pub hashed_value:String,
      pub description:Option<String> ,
      pub user_id:String,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
      pub user_id_fk:String,
      pub user_type:crate::logic::user::UserType,
      pub user_email:Option<String> ,
      pub user_role:crate::logic::user::Role,
      pub user_description:Option<String> ,
      pub user_created_at:shared::primitives::WrappedChronoDateTime,
      pub user_updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_api_key_by_hashed_value(
      conn: &shared::libsql::Connection
      ,params: get_api_key_by_hashed_value_params<'_>
  ) -> Result<Option<Row_get_api_key_by_hashed_value>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT ak.id, ak.hashed_value, ak.description, ak.user_id, ak.created_at, ak.updated_at,
       u.id as user_id_fk, u.type as user_type, u.email as user_email, u.role as user_role,
       u.description as user_description, u.created_at as user_created_at, u.updated_at as user_updated_at
FROM api_key ak
JOIN user u ON ak.user_id = u.id
WHERE ak.hashed_value = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.hashed_value.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_api_key_by_hashed_value {
                  id: row.get(0)?,
                  hashed_value: row.get(1)?,
                  description: row.get(2)?,
                  user_id: row.get(3)?,
                  created_at: row.get(4)?,
                  updated_at: row.get(5)?,
                  user_id_fk: row.get(6)?,
                  user_type: row.get(7)?,
                  user_email: row.get(8)?,
                  user_role: row.get(9)?,
                  user_description: row.get(10)?,
                  user_created_at: row.get(11)?,
                  user_updated_at: row.get(12)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct get_api_key_by_id_params<'a> {
      pub id: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_api_key_by_id {
      pub id:String,
      pub hashed_value:String,
      pub description:Option<String> ,
      pub user_id:String,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
      pub user_id_fk:String,
      pub user_type:crate::logic::user::UserType,
      pub user_email:Option<String> ,
      pub user_role:crate::logic::user::Role,
      pub user_description:Option<String> ,
      pub user_created_at:shared::primitives::WrappedChronoDateTime,
      pub user_updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_api_key_by_id(
      conn: &shared::libsql::Connection
      ,params: get_api_key_by_id_params<'_>
  ) -> Result<Option<Row_get_api_key_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT ak.id, ak.hashed_value, ak.description, ak.user_id, ak.created_at, ak.updated_at,
       u.id as user_id_fk, u.type as user_type, u.email as user_email, u.role as user_role,
       u.description as user_description, u.created_at as user_created_at, u.updated_at as user_updated_at
FROM api_key ak
JOIN user u ON ak.user_id = u.id
WHERE ak.id = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_api_key_by_id {
                  id: row.get(0)?,
                  hashed_value: row.get(1)?,
                  description: row.get(2)?,
                  user_id: row.get(3)?,
                  created_at: row.get(4)?,
                  updated_at: row.get(5)?,
                  user_id_fk: row.get(6)?,
                  user_type: row.get(7)?,
                  user_email: row.get(8)?,
                  user_role: row.get(9)?,
                  user_description: row.get(10)?,
                  user_created_at: row.get(11)?,
                  user_updated_at: row.get(12)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct delete_api_key_params<'a> {
      pub id: &'a 
          String
      ,
  }

  pub async fn delete_api_key(
    conn: &shared::libsql::Connection
    ,params: delete_api_key_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM api_key WHERE id = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_api_keys_params<'a> {
      pub cursor: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub user_id: &'a Option<
          String
      >,
      pub page_size: &'a 
          i64
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_api_keys {
      pub id:String,
      pub hashed_value:String,
      pub description:Option<String> ,
      pub user_id:String,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_api_keys(
      conn: &shared::libsql::Connection
      ,params: get_api_keys_params<'_>
  ) -> Result<Vec<Row_get_api_keys>, libsql::Error> {
      let stmt = conn.prepare(r#"SELECT id, hashed_value, description, user_id, created_at, updated_at
FROM api_key
WHERE (created_at < ?1 OR ?1 IS NULL)
  AND (user_id = ?2 OR ?2 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?3 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.cursor.clone(),params.user_id.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_api_keys {
              id: row.get(0)?,
              hashed_value: row.get(1)?,
              description: row.get(2)?,
              user_id: row.get(3)?,
              created_at: row.get(4)?,
              updated_at: row.get(5)?,
          });
      }

      Ok(mapped)
  }
  pub struct delete_api_keys_by_user_id_params<'a> {
      pub user_id: &'a 
          String
      ,
  }

  pub async fn delete_api_keys_by_user_id(
    conn: &shared::libsql::Connection
    ,params: delete_api_keys_by_user_id_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM api_key WHERE user_id = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.user_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
//  ============================================================================
//  Group table queries
//  ============================================================================
  pub struct create_group_params<'a> {
      pub id: &'a 
          String
      ,
      pub name: &'a 
          String
      ,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
  }

  pub async fn create_group(
    conn: &shared::libsql::Connection
    ,params: create_group_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"
INSERT INTO `group` (id, name, created_at, updated_at)
VALUES (?1, ?2, ?3, ?4)"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.name.clone())
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
  pub struct get_group_by_id_params<'a> {
      pub id: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_group_by_id {
      pub id:String,
      pub name:String,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_group_by_id(
      conn: &shared::libsql::Connection
      ,params: get_group_by_id_params<'_>
  ) -> Result<Option<Row_get_group_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, name, created_at, updated_at
FROM `group`
WHERE id = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_group_by_id {
                  id: row.get(0)?,
                  name: row.get(1)?,
                  created_at: row.get(2)?,
                  updated_at: row.get(3)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct update_group_params<'a> {
      pub name: &'a 
          String
      ,
      pub id: &'a 
          String
      ,
  }

  pub async fn update_group(
    conn: &shared::libsql::Connection
    ,params: update_group_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"UPDATE `group`
SET name = ?,
    updated_at = CURRENT_TIMESTAMP
WHERE id = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.name.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct delete_group_params<'a> {
      pub id: &'a 
          String
      ,
  }

  pub async fn delete_group(
    conn: &shared::libsql::Connection
    ,params: delete_group_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM `group` WHERE id = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_groups_params<'a> {
      pub cursor: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub page_size: &'a 
          i64
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_groups {
      pub id:String,
      pub name:String,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_groups(
      conn: &shared::libsql::Connection
      ,params: get_groups_params<'_>
  ) -> Result<Vec<Row_get_groups>, libsql::Error> {
      let stmt = conn.prepare(r#"SELECT id, name, created_at, updated_at
FROM `group`
WHERE (created_at < ?1 OR ?1 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?2 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_groups {
              id: row.get(0)?,
              name: row.get(1)?,
              created_at: row.get(2)?,
              updated_at: row.get(3)?,
          });
      }

      Ok(mapped)
  }
//  ============================================================================
//  Group membership table queries
//  ============================================================================
  pub struct create_group_membership_params<'a> {
      pub group_id: &'a 
          String
      ,
      pub user_id: &'a 
          String
      ,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
  }

  pub async fn create_group_membership(
    conn: &shared::libsql::Connection
    ,params: create_group_membership_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"
INSERT INTO group_membership (group_id, user_id, created_at, updated_at)
VALUES (?1, ?2, ?3, ?4)"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.group_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.user_id.clone())
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
  pub struct delete_group_membership_params<'a> {
      pub group_id: &'a 
          String
      ,
      pub user_id: &'a 
          String
      ,
  }

  pub async fn delete_group_membership(
    conn: &shared::libsql::Connection
    ,params: delete_group_membership_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM group_membership WHERE group_id = ? AND user_id = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.group_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.user_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_group_membership_params<'a> {
      pub group_id: &'a 
          String
      ,
      pub user_id: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_group_membership {
      pub group_id:String,
      pub user_id:String,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_group_membership(
      conn: &shared::libsql::Connection
      ,params: get_group_membership_params<'_>
  ) -> Result<Option<Row_get_group_membership>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT group_id, user_id, created_at, updated_at
FROM group_membership
WHERE group_id = ? AND user_id = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.group_id.clone(),params.user_id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_group_membership {
                  group_id: row.get(0)?,
                  user_id: row.get(1)?,
                  created_at: row.get(2)?,
                  updated_at: row.get(3)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct get_group_members_params<'a> {
      pub group_id: &'a 
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
  pub struct Row_get_group_members {
      pub group_id:String,
      pub user_id:String,
      pub membership_created_at:shared::primitives::WrappedChronoDateTime,
      pub membership_updated_at:shared::primitives::WrappedChronoDateTime,
      pub user_id_fk:String,
      pub user_type:crate::logic::user::UserType,
      pub user_email:Option<String> ,
      pub user_role:crate::logic::user::Role,
      pub user_description:Option<String> ,
      pub user_created_at:shared::primitives::WrappedChronoDateTime,
      pub user_updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_group_members(
      conn: &shared::libsql::Connection
      ,params: get_group_members_params<'_>
  ) -> Result<Vec<Row_get_group_members>, libsql::Error> {
      let stmt = conn.prepare(r#"SELECT gm.group_id, gm.user_id, gm.created_at as membership_created_at, gm.updated_at as membership_updated_at,
       u.id as user_id_fk, u.type as user_type, u.email as user_email, u.role as user_role,
       u.description as user_description, u.created_at as user_created_at, u.updated_at as user_updated_at
FROM group_membership gm
JOIN user u ON gm.user_id = u.id
WHERE gm.group_id = ?1
  AND (gm.created_at < ?2 OR ?2 IS NULL)
ORDER BY gm.created_at DESC
LIMIT CAST(?3 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.group_id.clone(),params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_group_members {
              group_id: row.get(0)?,
              user_id: row.get(1)?,
              membership_created_at: row.get(2)?,
              membership_updated_at: row.get(3)?,
              user_id_fk: row.get(4)?,
              user_type: row.get(5)?,
              user_email: row.get(6)?,
              user_role: row.get(7)?,
              user_description: row.get(8)?,
              user_created_at: row.get(9)?,
              user_updated_at: row.get(10)?,
          });
      }

      Ok(mapped)
  }
  pub struct get_user_groups_params<'a> {
      pub user_id: &'a 
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
  pub struct Row_get_user_groups {
      pub group_id:String,
      pub user_id:String,
      pub membership_created_at:shared::primitives::WrappedChronoDateTime,
      pub membership_updated_at:shared::primitives::WrappedChronoDateTime,
      pub group_id_fk:String,
      pub group_name:String,
      pub group_created_at:shared::primitives::WrappedChronoDateTime,
      pub group_updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_user_groups(
      conn: &shared::libsql::Connection
      ,params: get_user_groups_params<'_>
  ) -> Result<Vec<Row_get_user_groups>, libsql::Error> {
      let stmt = conn.prepare(r#"SELECT gm.group_id, gm.user_id, gm.created_at as membership_created_at, gm.updated_at as membership_updated_at,
       g.id as group_id_fk, g.name as group_name, g.created_at as group_created_at, g.updated_at as group_updated_at
FROM group_membership gm
JOIN `group` g ON gm.group_id = g.id
WHERE gm.user_id = ?1
  AND (gm.created_at < ?2 OR ?2 IS NULL)
ORDER BY gm.created_at DESC
LIMIT CAST(?3 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.user_id.clone(),params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_user_groups {
              group_id: row.get(0)?,
              user_id: row.get(1)?,
              membership_created_at: row.get(2)?,
              membership_updated_at: row.get(3)?,
              group_id_fk: row.get(4)?,
              group_name: row.get(5)?,
              group_created_at: row.get(6)?,
              group_updated_at: row.get(7)?,
          });
      }

      Ok(mapped)
  }
  pub struct delete_group_memberships_by_group_id_params<'a> {
      pub group_id: &'a 
          String
      ,
  }

  pub async fn delete_group_memberships_by_group_id(
    conn: &shared::libsql::Connection
    ,params: delete_group_memberships_by_group_id_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM group_membership WHERE group_id = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.group_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct delete_group_memberships_by_user_id_params<'a> {
      pub user_id: &'a 
          String
      ,
  }

  pub async fn delete_group_memberships_by_user_id(
    conn: &shared::libsql::Connection
    ,params: delete_group_memberships_by_user_id_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM group_membership WHERE user_id = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.user_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
//  ============================================================================
//  JWT signing key table queries
//  ============================================================================
  pub struct create_jwt_signing_key_params<'a> {
      pub kid: &'a 
          String
      ,
      pub encrypted_private_key: &'a 
          String
      ,
      pub expires_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub public_key: &'a 
          String
      ,
      pub dek_alias: &'a 
          String
      ,
      pub invalidated: &'a 
          bool
      ,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
  }

  pub async fn create_jwt_signing_key(
    conn: &shared::libsql::Connection
    ,params: create_jwt_signing_key_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"
INSERT INTO jwt_signing_key (kid, encrypted_private_key, expires_at, public_key, dek_alias, invalidated, created_at, updated_at)
VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.kid.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.encrypted_private_key.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(params.expires_at.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.public_key.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.dek_alias.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <bool as TryInto<libsql::Value>>::try_into(params.invalidated.clone())
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
  pub struct get_jwt_signing_key_by_kid_params<'a> {
      pub kid: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_jwt_signing_key_by_kid {
      pub kid:String,
      pub encrypted_private_key:String,
      pub expires_at:shared::primitives::WrappedChronoDateTime,
      pub public_key:String,
      pub dek_alias:String,
      pub invalidated:bool,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_jwt_signing_key_by_kid(
      conn: &shared::libsql::Connection
      ,params: get_jwt_signing_key_by_kid_params<'_>
  ) -> Result<Option<Row_get_jwt_signing_key_by_kid>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT kid, encrypted_private_key, expires_at, public_key, dek_alias, invalidated, created_at, updated_at
FROM jwt_signing_key
WHERE kid = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.kid.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_jwt_signing_key_by_kid {
                  kid: row.get(0)?,
                  encrypted_private_key: row.get(1)?,
                  expires_at: row.get(2)?,
                  public_key: row.get(3)?,
                  dek_alias: row.get(4)?,
                  invalidated: row.get(5)?,
                  created_at: row.get(6)?,
                  updated_at: row.get(7)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct invalidate_jwt_signing_key_params<'a> {
      pub kid: &'a 
          String
      ,
  }

  pub async fn invalidate_jwt_signing_key(
    conn: &shared::libsql::Connection
    ,params: invalidate_jwt_signing_key_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"UPDATE jwt_signing_key
SET invalidated = 1,
    updated_at = CURRENT_TIMESTAMP
WHERE kid = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.kid.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_jwt_signing_keys_params<'a> {
      pub cursor: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub page_size: &'a 
          i64
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_jwt_signing_keys {
      pub kid:String,
      pub encrypted_private_key:String,
      pub expires_at:shared::primitives::WrappedChronoDateTime,
      pub public_key:String,
      pub dek_alias:String,
      pub invalidated:bool,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_jwt_signing_keys(
      conn: &shared::libsql::Connection
      ,params: get_jwt_signing_keys_params<'_>
  ) -> Result<Vec<Row_get_jwt_signing_keys>, libsql::Error> {
      let stmt = conn.prepare(r#"SELECT kid, encrypted_private_key, expires_at, public_key, dek_alias, invalidated, created_at, updated_at
FROM jwt_signing_key
WHERE invalidated = 0
  AND (created_at < ?1 OR ?1 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?2 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_jwt_signing_keys {
              kid: row.get(0)?,
              encrypted_private_key: row.get(1)?,
              expires_at: row.get(2)?,
              public_key: row.get(3)?,
              dek_alias: row.get(4)?,
              invalidated: row.get(5)?,
              created_at: row.get(6)?,
              updated_at: row.get(7)?,
          });
      }

      Ok(mapped)
  }
//  ============================================================================
//  OAuth state table queries
//  ============================================================================
  pub struct create_oauth_state_params<'a> {
      pub state: &'a 
          String
      ,
      pub config_id: &'a 
          String
      ,
      pub code_verifier: &'a Option<
          String
      >,
      pub nonce: &'a Option<
          String
      >,
      pub redirect_uri: &'a Option<
          String
      >,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub expires_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
  }

  pub async fn create_oauth_state(
    conn: &shared::libsql::Connection
    ,params: create_oauth_state_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"
INSERT INTO oauth_state (state, config_id, code_verifier, nonce, redirect_uri, created_at, expires_at)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.state.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.config_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              match params.code_verifier.clone() {
                Some(value) => {
                  <String as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
              match params.nonce.clone() {
                Some(value) => {
                  <String as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
              match params.redirect_uri.clone() {
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
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(params.expires_at.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_oauth_state_by_state_params<'a> {
      pub state: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_oauth_state_by_state {
      pub state:String,
      pub config_id:String,
      pub code_verifier:Option<String> ,
      pub nonce:Option<String> ,
      pub redirect_uri:Option<String> ,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub expires_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_oauth_state_by_state(
      conn: &shared::libsql::Connection
      ,params: get_oauth_state_by_state_params<'_>
  ) -> Result<Option<Row_get_oauth_state_by_state>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT state, config_id, code_verifier, nonce, redirect_uri, created_at, expires_at
FROM oauth_state
WHERE state = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.state.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_oauth_state_by_state {
                  state: row.get(0)?,
                  config_id: row.get(1)?,
                  code_verifier: row.get(2)?,
                  nonce: row.get(3)?,
                  redirect_uri: row.get(4)?,
                  created_at: row.get(5)?,
                  expires_at: row.get(6)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct delete_oauth_state_params<'a> {
      pub state: &'a 
          String
      ,
  }

  pub async fn delete_oauth_state(
    conn: &shared::libsql::Connection
    ,params: delete_oauth_state_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM oauth_state WHERE state = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.state.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct delete_expired_oauth_states_params<'a> {
      pub now: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
  }

  pub async fn delete_expired_oauth_states(
    conn: &shared::libsql::Connection
    ,params: delete_expired_oauth_states_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM oauth_state WHERE expires_at < ?1"#, libsql::params![
              <shared::primitives::WrappedChronoDateTime as TryInto<libsql::Value>>::try_into(params.now.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
//  ============================================================================
//  STS configuration table queries
//  ============================================================================
  pub struct create_sts_configuration_params<'a> {
      pub id: &'a 
          String
      ,
      pub config_type: &'a 
          String
      ,
      pub value: &'a Option<
          shared::primitives::WrappedJsonValue
      >,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
  }

  pub async fn create_sts_configuration(
    conn: &shared::libsql::Connection
    ,params: create_sts_configuration_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"
INSERT INTO sts_configuration (id, type, value, created_at, updated_at)
VALUES (?1, ?2, ?3, ?4, ?5)"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.config_type.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              match params.value.clone() {
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
  pub struct get_sts_configuration_by_id_params<'a> {
      pub id: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_sts_configuration_by_id {
      pub id:String,
      pub config_type:String,
      pub value:Option<shared::primitives::WrappedJsonValue> ,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_sts_configuration_by_id(
      conn: &shared::libsql::Connection
      ,params: get_sts_configuration_by_id_params<'_>
  ) -> Result<Option<Row_get_sts_configuration_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, type as config_type, value, created_at, updated_at
FROM sts_configuration
WHERE id = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_sts_configuration_by_id {
                  id: row.get(0)?,
                  config_type: row.get(1)?,
                  value: row.get(2)?,
                  created_at: row.get(3)?,
                  updated_at: row.get(4)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct delete_sts_configuration_params<'a> {
      pub id: &'a 
          String
      ,
  }

  pub async fn delete_sts_configuration(
    conn: &shared::libsql::Connection
    ,params: delete_sts_configuration_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM sts_configuration WHERE id = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_sts_configurations_params<'a> {
      pub cursor: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub config_type: &'a Option<
          String
      >,
      pub page_size: &'a 
          i64
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_sts_configurations {
      pub id:String,
      pub config_type:String,
      pub value:Option<shared::primitives::WrappedJsonValue> ,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_sts_configurations(
      conn: &shared::libsql::Connection
      ,params: get_sts_configurations_params<'_>
  ) -> Result<Vec<Row_get_sts_configurations>, libsql::Error> {
      let stmt = conn.prepare(r#"SELECT id, type as config_type, value, created_at, updated_at
FROM sts_configuration
WHERE (created_at < ?1 OR ?1 IS NULL)
  AND (type = ?2 OR ?2 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?3 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.cursor.clone(),params.config_type.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_sts_configurations {
              id: row.get(0)?,
              config_type: row.get(1)?,
              value: row.get(2)?,
              created_at: row.get(3)?,
              updated_at: row.get(4)?,
          });
      }

      Ok(mapped)
  }
//  ============================================================================
//  User table queries
//  ============================================================================
  pub struct create_user_params<'a> {
      pub id: &'a 
          String
      ,
      pub user_type: &'a 
          crate::logic::user::UserType
      ,
      pub email: &'a Option<
          String
      >,
      pub role: &'a 
          crate::logic::user::Role
      ,
      pub description: &'a Option<
          String
      >,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
  }

  pub async fn create_user(
    conn: &shared::libsql::Connection
    ,params: create_user_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"
INSERT INTO user (id, type, email, role, description, created_at, updated_at)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <crate::logic::user::UserType as TryInto<libsql::Value>>::try_into(params.user_type.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              match params.email.clone() {
                Some(value) => {
                  <String as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
              <crate::logic::user::Role as TryInto<libsql::Value>>::try_into(params.role.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              match params.description.clone() {
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
  pub struct get_user_by_id_params<'a> {
      pub id: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_user_by_id {
      pub id:String,
      pub user_type:crate::logic::user::UserType,
      pub email:Option<String> ,
      pub role:crate::logic::user::Role,
      pub description:Option<String> ,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_user_by_id(
      conn: &shared::libsql::Connection
      ,params: get_user_by_id_params<'_>
  ) -> Result<Option<Row_get_user_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, type as user_type, email, role, description, created_at, updated_at
FROM user
WHERE id = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_user_by_id {
                  id: row.get(0)?,
                  user_type: row.get(1)?,
                  email: row.get(2)?,
                  role: row.get(3)?,
                  description: row.get(4)?,
                  created_at: row.get(5)?,
                  updated_at: row.get(6)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct update_user_params<'a> {
      pub email: &'a Option<
          String
      >,
      pub role: &'a 
          crate::logic::user::Role
      ,
      pub description: &'a Option<
          String
      >,
      pub id: &'a 
          String
      ,
  }

  pub async fn update_user(
    conn: &shared::libsql::Connection
    ,params: update_user_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"UPDATE user
SET email = ?,
    role = ?,
    description = ?,
    updated_at = CURRENT_TIMESTAMP
WHERE id = ?"#, libsql::params![
              match params.email.clone() {
                Some(value) => {
                  <String as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
              <crate::logic::user::Role as TryInto<libsql::Value>>::try_into(params.role.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              match params.description.clone() {
                Some(value) => {
                  <String as TryInto<libsql::Value>>::try_into(value.clone())
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
  pub struct delete_user_params<'a> {
      pub id: &'a 
          String
      ,
  }

  pub async fn delete_user(
    conn: &shared::libsql::Connection
    ,params: delete_user_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM user WHERE id = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_users_params<'a> {
      pub cursor: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub user_type: &'a Option<
          crate::logic::user::UserType
      >,
      pub role: &'a Option<
          crate::logic::user::Role
      >,
      pub page_size: &'a 
          i64
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_users {
      pub id:String,
      pub user_type:crate::logic::user::UserType,
      pub email:Option<String> ,
      pub role:crate::logic::user::Role,
      pub description:Option<String> ,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_users(
      conn: &shared::libsql::Connection
      ,params: get_users_params<'_>
  ) -> Result<Vec<Row_get_users>, libsql::Error> {
      let stmt = conn.prepare(r#"SELECT id, type as user_type, email, role, description, created_at, updated_at
FROM user
WHERE (created_at < ?1 OR ?1 IS NULL)
  AND (type = ?2 OR ?2 IS NULL)
  AND (role = ?3 OR ?3 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?4 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.cursor.clone(),params.user_type.clone(),params.role.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_users {
              id: row.get(0)?,
              user_type: row.get(1)?,
              email: row.get(2)?,
              role: row.get(3)?,
              description: row.get(4)?,
              created_at: row.get(5)?,
              updated_at: row.get(6)?,
          });
      }

      Ok(mapped)
  }
//  ============================================================================
//  User auth flow configuration table queries
//  ============================================================================
  pub struct create_user_auth_flow_config_params<'a> {
      pub id: &'a 
          String
      ,
      pub config_type: &'a 
          String
      ,
      pub config: &'a 
          shared::primitives::WrappedJsonValue
      ,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
  }

  pub async fn create_user_auth_flow_config(
    conn: &shared::libsql::Connection
    ,params: create_user_auth_flow_config_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"
INSERT INTO user_auth_flow_configuration (id, type, config, created_at, updated_at)
VALUES (?1, ?2, ?3, ?4, ?5)"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.config_type.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.config.clone())
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
  pub struct get_user_auth_flow_config_by_id_params<'a> {
      pub id: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_user_auth_flow_config_by_id {
      pub id:String,
      pub config_type:String,
      pub config:shared::primitives::WrappedJsonValue,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_user_auth_flow_config_by_id(
      conn: &shared::libsql::Connection
      ,params: get_user_auth_flow_config_by_id_params<'_>
  ) -> Result<Option<Row_get_user_auth_flow_config_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, type as config_type, config, created_at, updated_at
FROM user_auth_flow_configuration
WHERE id = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_user_auth_flow_config_by_id {
                  id: row.get(0)?,
                  config_type: row.get(1)?,
                  config: row.get(2)?,
                  created_at: row.get(3)?,
                  updated_at: row.get(4)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct delete_user_auth_flow_config_params<'a> {
      pub id: &'a 
          String
      ,
  }

  pub async fn delete_user_auth_flow_config(
    conn: &shared::libsql::Connection
    ,params: delete_user_auth_flow_config_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM user_auth_flow_configuration WHERE id = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_user_auth_flow_configs_params<'a> {
      pub cursor: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub config_type: &'a Option<
          String
      >,
      pub page_size: &'a 
          i64
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_user_auth_flow_configs {
      pub id:String,
      pub config_type:String,
      pub config:shared::primitives::WrappedJsonValue,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_user_auth_flow_configs(
      conn: &shared::libsql::Connection
      ,params: get_user_auth_flow_configs_params<'_>
  ) -> Result<Vec<Row_get_user_auth_flow_configs>, libsql::Error> {
      let stmt = conn.prepare(r#"SELECT id, type as config_type, config, created_at, updated_at
FROM user_auth_flow_configuration
WHERE (created_at < ?1 OR ?1 IS NULL)
  AND (type = ?2 OR ?2 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?3 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.cursor.clone(),params.config_type.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_user_auth_flow_configs {
              id: row.get(0)?,
              config_type: row.get(1)?,
              config: row.get(2)?,
              created_at: row.get(3)?,
              updated_at: row.get(4)?,
          });
      }

      Ok(mapped)
  }
