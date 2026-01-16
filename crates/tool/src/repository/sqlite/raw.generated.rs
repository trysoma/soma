

#[allow(unused)]
use serde::{Serialize, Deserialize};
  pub struct create_mcp_server_instance_params<'a> {
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

  pub async fn create_mcp_server_instance(
    conn: &shared::libsql::Connection
    ,params: create_mcp_server_instance_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO mcp_server_instance (id, name, created_at, updated_at)
VALUES (?, ?, ?, ?)"#, libsql::params![
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
  pub struct get_mcp_server_instance_by_id_params<'a> {
      pub id: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_mcp_server_instance_by_id {
      pub id:String,
      pub name:String,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
      pub tools:String,
  }
  pub async fn get_mcp_server_instance_by_id(
      conn: &shared::libsql::Connection
      ,params: get_mcp_server_instance_by_id_params<'_>
  ) -> Result<Option<Row_get_mcp_server_instance_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT
    msi.id,
    msi.name,
    msi.created_at,
    msi.updated_at,
    CAST(COALESCE(
        (SELECT JSON_GROUP_ARRAY(
            JSON_OBJECT(
                'mcp_server_instance_id', msif.mcp_server_instance_id,
                'tool_deployment_type_id', msif.tool_deployment_type_id,
                'tool_group_deployment_type_id', msif.tool_group_deployment_type_id,
                'tool_group_id', msif.tool_group_id,
                'tool_name', msif.tool_name,
                'tool_description', msif.tool_description,
                'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', msif.created_at),
                'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', msif.updated_at)
            )
        )
        FROM mcp_server_instance_tool msif
        WHERE msif.mcp_server_instance_id = msi.id
        ), JSON('[]')) AS TEXT
    ) AS tools
FROM mcp_server_instance msi
WHERE msi.id = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_mcp_server_instance_by_id {
                  id: row.get(0)?,
                  name: row.get(1)?,
                  created_at: row.get(2)?,
                  updated_at: row.get(3)?,
                  tools: row.get(4)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct update_mcp_server_instance_params<'a> {
      pub name: &'a 
          String
      ,
      pub id: &'a 
          String
      ,
  }

  pub async fn update_mcp_server_instance(
    conn: &shared::libsql::Connection
    ,params: update_mcp_server_instance_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"UPDATE mcp_server_instance
SET name = ?, updated_at = CURRENT_TIMESTAMP
WHERE id = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.name.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct delete_mcp_server_instance_params<'a> {
      pub id: &'a 
          String
      ,
  }

  pub async fn delete_mcp_server_instance(
    conn: &shared::libsql::Connection
    ,params: delete_mcp_server_instance_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM mcp_server_instance WHERE id = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct list_mcp_server_instances_params<'a> {
      pub cursor: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub page_size: &'a 
          i64
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_list_mcp_server_instances {
      pub id:String,
      pub name:String,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
      pub tools:String,
  }
  pub async fn list_mcp_server_instances(
      conn: &shared::libsql::Connection
      ,params: list_mcp_server_instances_params<'_>
  ) -> Result<Vec<Row_list_mcp_server_instances>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT
    msi.id,
    msi.name,
    msi.created_at,
    msi.updated_at,
    CAST(COALESCE(
        (SELECT JSON_GROUP_ARRAY(
            JSON_OBJECT(
                'mcp_server_instance_id', msif.mcp_server_instance_id,
                'tool_deployment_type_id', msif.tool_deployment_type_id,
                'tool_group_deployment_type_id', msif.tool_group_deployment_type_id,
                'tool_group_id', msif.tool_group_id,
                'tool_name', msif.tool_name,
                'tool_description', msif.tool_description,
                'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', msif.created_at),
                'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', msif.updated_at)
            )
        )
        FROM mcp_server_instance_tool msif
        WHERE msif.mcp_server_instance_id = msi.id
        ), JSON('[]')) AS TEXT
    ) AS tools
FROM mcp_server_instance msi
WHERE (msi.created_at < ?1 OR ?1 IS NULL)
ORDER BY msi.created_at DESC
LIMIT CAST(?2 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_list_mcp_server_instances {
              id: row.get(0)?,
              name: row.get(1)?,
              created_at: row.get(2)?,
              updated_at: row.get(3)?,
              tools: row.get(4)?,
          });
      }

      Ok(mapped)
  }
  pub struct create_mcp_server_instance_tool_params<'a> {
      pub mcp_server_instance_id: &'a 
          String
      ,
      pub tool_deployment_type_id: &'a 
          String
      ,
      pub tool_group_deployment_type_id: &'a 
          String
      ,
      pub tool_group_id: &'a 
          String
      ,
      pub tool_name: &'a 
          String
      ,
      pub tool_description: &'a Option<
          String
      >,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
  }

  pub async fn create_mcp_server_instance_tool(
    conn: &shared::libsql::Connection
    ,params: create_mcp_server_instance_tool_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO mcp_server_instance_tool (mcp_server_instance_id, tool_deployment_type_id, tool_group_deployment_type_id, tool_group_id, tool_name, tool_description, created_at, updated_at)
VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.mcp_server_instance_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.tool_deployment_type_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.tool_group_deployment_type_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.tool_group_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.tool_name.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              match params.tool_description.clone() {
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
  pub struct update_mcp_server_instance_tool_params<'a> {
      pub tool_name: &'a 
          String
      ,
      pub tool_description: &'a Option<
          String
      >,
      pub mcp_server_instance_id: &'a 
          String
      ,
      pub tool_deployment_type_id: &'a 
          String
      ,
      pub tool_group_deployment_type_id: &'a 
          String
      ,
      pub tool_group_id: &'a 
          String
      ,
  }

  pub async fn update_mcp_server_instance_tool(
    conn: &shared::libsql::Connection
    ,params: update_mcp_server_instance_tool_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"UPDATE mcp_server_instance_tool
SET tool_name = ?, tool_description = ?, updated_at = CURRENT_TIMESTAMP
WHERE mcp_server_instance_id = ?
  AND tool_deployment_type_id = ?
  AND tool_group_deployment_type_id = ?
  AND tool_group_id = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.tool_name.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              match params.tool_description.clone() {
                Some(value) => {
                  <String as TryInto<libsql::Value>>::try_into(value.clone())
                      .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
                },
                None => libsql::Value::Null,
              }
            ,
              <String as TryInto<libsql::Value>>::try_into(params.mcp_server_instance_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.tool_deployment_type_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.tool_group_deployment_type_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.tool_group_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct get_mcp_server_instance_tool_by_name_params<'a> {
      pub mcp_server_instance_id: &'a 
          String
      ,
      pub tool_name: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_mcp_server_instance_tool_by_name {
      pub mcp_server_instance_id:String,
      pub tool_deployment_type_id:String,
      pub tool_group_deployment_type_id:String,
      pub tool_group_id:String,
      pub tool_name:String,
      pub tool_description:Option<String> ,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_mcp_server_instance_tool_by_name(
      conn: &shared::libsql::Connection
      ,params: get_mcp_server_instance_tool_by_name_params<'_>
  ) -> Result<Option<Row_get_mcp_server_instance_tool_by_name>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT
    mcp_server_instance_id,
    tool_deployment_type_id,
    tool_group_deployment_type_id,
    tool_group_id,
    tool_name,
    tool_description,
    created_at,
    updated_at
FROM mcp_server_instance_tool
WHERE mcp_server_instance_id = ?
  AND tool_name = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.mcp_server_instance_id.clone(),params.tool_name.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_mcp_server_instance_tool_by_name {
                  mcp_server_instance_id: row.get(0)?,
                  tool_deployment_type_id: row.get(1)?,
                  tool_group_deployment_type_id: row.get(2)?,
                  tool_group_id: row.get(3)?,
                  tool_name: row.get(4)?,
                  tool_description: row.get(5)?,
                  created_at: row.get(6)?,
                  updated_at: row.get(7)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct delete_mcp_server_instance_tool_params<'a> {
      pub mcp_server_instance_id: &'a 
          String
      ,
      pub tool_deployment_type_id: &'a 
          String
      ,
      pub tool_group_deployment_type_id: &'a 
          String
      ,
      pub tool_group_id: &'a 
          String
      ,
  }

  pub async fn delete_mcp_server_instance_tool(
    conn: &shared::libsql::Connection
    ,params: delete_mcp_server_instance_tool_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM mcp_server_instance_tool
WHERE mcp_server_instance_id = ?
  AND tool_deployment_type_id = ?
  AND tool_group_deployment_type_id = ?
  AND tool_group_id = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.mcp_server_instance_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.tool_deployment_type_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.tool_group_deployment_type_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.tool_group_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct delete_all_mcp_server_instance_tools_params<'a> {
      pub mcp_server_instance_id: &'a 
          String
      ,
  }

  pub async fn delete_all_mcp_server_instance_tools(
    conn: &shared::libsql::Connection
    ,params: delete_all_mcp_server_instance_tools_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM mcp_server_instance_tool WHERE mcp_server_instance_id = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.mcp_server_instance_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct list_mcp_server_instance_tools_params<'a> {
      pub mcp_server_instance_id: &'a 
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
  pub struct Row_list_mcp_server_instance_tools {
      pub mcp_server_instance_id:String,
      pub tool_deployment_type_id:String,
      pub tool_group_deployment_type_id:String,
      pub tool_group_id:String,
      pub tool_name:String,
      pub tool_description:Option<String> ,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn list_mcp_server_instance_tools(
      conn: &shared::libsql::Connection
      ,params: list_mcp_server_instance_tools_params<'_>
  ) -> Result<Vec<Row_list_mcp_server_instance_tools>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT
    mcp_server_instance_id,
    tool_deployment_type_id,
    tool_group_deployment_type_id,
    tool_group_id,
    tool_name,
    tool_description,
    created_at,
    updated_at
FROM mcp_server_instance_tool
WHERE mcp_server_instance_id = ?
  AND (created_at < ?2 OR ?2 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?3 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.mcp_server_instance_id.clone(),params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_list_mcp_server_instance_tools {
              mcp_server_instance_id: row.get(0)?,
              tool_deployment_type_id: row.get(1)?,
              tool_group_deployment_type_id: row.get(2)?,
              tool_group_id: row.get(3)?,
              tool_name: row.get(4)?,
              tool_description: row.get(5)?,
              created_at: row.get(6)?,
              updated_at: row.get(7)?,
          });
      }

      Ok(mapped)
  }
//  Tool CRUD operations
  pub struct create_tool_group_deployment_params<'a> {
      pub type_id: &'a 
          String
      ,
      pub deployment_id: &'a 
          String
      ,
      pub name: &'a 
          String
      ,
      pub documentation: &'a 
          String
      ,
      pub categories: &'a 
          shared::primitives::WrappedJsonValue
      ,
      pub endpoint_type: &'a 
          crate::logic::EndpointType
      ,
      pub endpoint_configuration: &'a 
          shared::primitives::WrappedJsonValue
      ,
      pub metadata: &'a 
          crate::logic::Metadata
      ,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
  }

  pub async fn create_tool_group_deployment(
    conn: &shared::libsql::Connection
    ,params: create_tool_group_deployment_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"
INSERT INTO tool_group_deployment (type_id, deployment_id, name, documentation, categories, endpoint_type, endpoint_configuration, metadata, created_at, updated_at)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.type_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.deployment_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.name.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.documentation.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.categories.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <crate::logic::EndpointType as TryInto<libsql::Value>>::try_into(params.endpoint_type.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <shared::primitives::WrappedJsonValue as TryInto<libsql::Value>>::try_into(params.endpoint_configuration.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <crate::logic::Metadata as TryInto<libsql::Value>>::try_into(params.metadata.clone())
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
  pub struct get_tool_group_deployment_by_id_params<'a> {
      pub type_id: &'a 
          String
      ,
      pub deployment_id: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_tool_group_deployment_by_id {
      pub type_id:String,
      pub deployment_id:String,
      pub name:String,
      pub documentation:String,
      pub categories:shared::primitives::WrappedJsonValue,
      pub endpoint_type:crate::logic::EndpointType,
      pub endpoint_configuration:shared::primitives::WrappedJsonValue,
      pub metadata:crate::logic::Metadata,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_tool_group_deployment_by_id(
      conn: &shared::libsql::Connection
      ,params: get_tool_group_deployment_by_id_params<'_>
  ) -> Result<Option<Row_get_tool_group_deployment_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT type_id, deployment_id, name, documentation, categories, endpoint_type, endpoint_configuration, metadata, created_at, updated_at
FROM tool_group_deployment
WHERE type_id = ? AND deployment_id = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.type_id.clone(),params.deployment_id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_tool_group_deployment_by_id {
                  type_id: row.get(0)?,
                  deployment_id: row.get(1)?,
                  name: row.get(2)?,
                  documentation: row.get(3)?,
                  categories: row.get(4)?,
                  endpoint_type: row.get(5)?,
                  endpoint_configuration: row.get(6)?,
                  metadata: row.get(7)?,
                  created_at: row.get(8)?,
                  updated_at: row.get(9)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct delete_tool_group_deployment_params<'a> {
      pub type_id: &'a 
          String
      ,
      pub deployment_id: &'a 
          String
      ,
  }

  pub async fn delete_tool_group_deployment(
    conn: &shared::libsql::Connection
    ,params: delete_tool_group_deployment_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM tool_group_deployment WHERE type_id = ? AND deployment_id = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.type_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.deployment_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct list_tool_group_deployments_params<'a> {
      pub endpoint_type: &'a Option<
          crate::logic::EndpointType
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
  pub struct Row_list_tool_group_deployments {
      pub type_id:String,
      pub deployment_id:String,
      pub name:String,
      pub documentation:String,
      pub categories:shared::primitives::WrappedJsonValue,
      pub endpoint_type:crate::logic::EndpointType,
      pub endpoint_configuration:shared::primitives::WrappedJsonValue,
      pub metadata:crate::logic::Metadata,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn list_tool_group_deployments(
      conn: &shared::libsql::Connection
      ,params: list_tool_group_deployments_params<'_>
  ) -> Result<Vec<Row_list_tool_group_deployments>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT type_id, deployment_id, name, documentation, categories, endpoint_type, endpoint_configuration, metadata, created_at, updated_at
FROM tool_group_deployment
WHERE (CAST(endpoint_type = ?1 AS TEXT) OR ?1 IS NULL)
  AND (created_at < ?2 OR ?2 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?3 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.endpoint_type.clone(),params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_list_tool_group_deployments {
              type_id: row.get(0)?,
              deployment_id: row.get(1)?,
              name: row.get(2)?,
              documentation: row.get(3)?,
              categories: row.get(4)?,
              endpoint_type: row.get(5)?,
              endpoint_configuration: row.get(6)?,
              metadata: row.get(7)?,
              created_at: row.get(8)?,
              updated_at: row.get(9)?,
          });
      }

      Ok(mapped)
  }
  pub struct list_tool_group_deployments_by_category_params<'a> {
      pub category: &'a Option<
          String
      >,
      pub endpoint_type: &'a Option<
          crate::logic::EndpointType
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
  pub struct Row_list_tool_group_deployments_by_category {
      pub type_id:String,
      pub deployment_id:String,
      pub name:String,
      pub documentation:String,
      pub categories:shared::primitives::WrappedJsonValue,
      pub endpoint_type:crate::logic::EndpointType,
      pub endpoint_configuration:shared::primitives::WrappedJsonValue,
      pub metadata:crate::logic::Metadata,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn list_tool_group_deployments_by_category(
      conn: &shared::libsql::Connection
      ,params: list_tool_group_deployments_by_category_params<'_>
  ) -> Result<Vec<Row_list_tool_group_deployments_by_category>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT type_id, deployment_id, name, documentation, categories, endpoint_type, endpoint_configuration, metadata, created_at, updated_at
FROM tool_group_deployment
WHERE JSON_EXTRACT(categories, '$') LIKE '%' || ?1 || '%'
  AND (CAST(endpoint_type = ?2 AS TEXT) OR ?2 IS NULL)
  AND (created_at < ?3 OR ?3 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?4 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.category.clone(),params.endpoint_type.clone(),params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_list_tool_group_deployments_by_category {
              type_id: row.get(0)?,
              deployment_id: row.get(1)?,
              name: row.get(2)?,
              documentation: row.get(3)?,
              categories: row.get(4)?,
              endpoint_type: row.get(5)?,
              endpoint_configuration: row.get(6)?,
              metadata: row.get(7)?,
              created_at: row.get(8)?,
              updated_at: row.get(9)?,
          });
      }

      Ok(mapped)
  }
//  Tool alias operations
  pub struct create_tool_group_deployment_alias_params<'a> {
      pub tool_group_deployment_type_id: &'a 
          String
      ,
      pub tool_group_deployment_deployment_id: &'a 
          String
      ,
      pub alias: &'a 
          String
      ,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
  }

  pub async fn create_tool_group_deployment_alias(
    conn: &shared::libsql::Connection
    ,params: create_tool_group_deployment_alias_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"
INSERT INTO tool_group_deployment_alias (tool_group_deployment_type_id, tool_group_deployment_deployment_id, alias, created_at, updated_at)
VALUES (?, ?, ?, ?, ?)"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.tool_group_deployment_type_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.tool_group_deployment_deployment_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.alias.clone())
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
  pub struct get_tool_group_deployment_by_alias_params<'a> {
      pub alias: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_tool_group_deployment_by_alias {
      pub type_id:String,
      pub deployment_id:String,
      pub name:String,
      pub documentation:String,
      pub categories:shared::primitives::WrappedJsonValue,
      pub endpoint_type:crate::logic::EndpointType,
      pub endpoint_configuration:shared::primitives::WrappedJsonValue,
      pub metadata:crate::logic::Metadata,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_tool_group_deployment_by_alias(
      conn: &shared::libsql::Connection
      ,params: get_tool_group_deployment_by_alias_params<'_>
  ) -> Result<Option<Row_get_tool_group_deployment_by_alias>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT t.type_id, t.deployment_id, t.name, t.documentation, t.categories, t.endpoint_type, t.endpoint_configuration, t.metadata, t.created_at, t.updated_at
FROM tool_group_deployment t
INNER JOIN tool_group_deployment_alias ta ON t.type_id = ta.tool_group_deployment_type_id AND t.deployment_id = ta.tool_group_deployment_deployment_id
WHERE ta.alias = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.alias.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_tool_group_deployment_by_alias {
                  type_id: row.get(0)?,
                  deployment_id: row.get(1)?,
                  name: row.get(2)?,
                  documentation: row.get(3)?,
                  categories: row.get(4)?,
                  endpoint_type: row.get(5)?,
                  endpoint_configuration: row.get(6)?,
                  metadata: row.get(7)?,
                  created_at: row.get(8)?,
                  updated_at: row.get(9)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct delete_tool_group_deployment_alias_params<'a> {
      pub alias: &'a 
          String
      ,
  }

  pub async fn delete_tool_group_deployment_alias(
    conn: &shared::libsql::Connection
    ,params: delete_tool_group_deployment_alias_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM tool_group_deployment_alias WHERE alias = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.alias.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct list_tool_group_deployment_aliases_params<'a> {
      pub tool_group_deployment_type_id: &'a Option<
          String
      >,
      pub tool_group_deployment_deployment_id: &'a Option<
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
  pub struct Row_list_tool_group_deployment_aliases {
      pub tool_group_deployment_type_id:String,
      pub tool_group_deployment_deployment_id:String,
      pub alias:String,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn list_tool_group_deployment_aliases(
      conn: &shared::libsql::Connection
      ,params: list_tool_group_deployment_aliases_params<'_>
  ) -> Result<Vec<Row_list_tool_group_deployment_aliases>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT tool_group_deployment_type_id, tool_group_deployment_deployment_id, alias, created_at, updated_at
FROM tool_group_deployment_alias
WHERE (CAST(tool_group_deployment_type_id = ?1 AS TEXT) OR ?1 IS NULL)
  AND (CAST(tool_group_deployment_deployment_id = ?2 AS TEXT) OR ?2 IS NULL)
  AND (created_at < ?3 OR ?3 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?4 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.tool_group_deployment_type_id.clone(),params.tool_group_deployment_deployment_id.clone(),params.cursor.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_list_tool_group_deployment_aliases {
              tool_group_deployment_type_id: row.get(0)?,
              tool_group_deployment_deployment_id: row.get(1)?,
              alias: row.get(2)?,
              created_at: row.get(3)?,
              updated_at: row.get(4)?,
          });
      }

      Ok(mapped)
  }
  pub struct get_aliases_for_tool_group_deployment_params<'a> {
      pub tool_group_deployment_type_id: &'a 
          String
      ,
      pub tool_group_deployment_deployment_id: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_aliases_for_tool_group_deployment {
      pub alias:String,
  }
  pub async fn get_aliases_for_tool_group_deployment(
      conn: &shared::libsql::Connection
      ,params: get_aliases_for_tool_group_deployment_params<'_>
  ) -> Result<Vec<Row_get_aliases_for_tool_group_deployment>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT alias
FROM tool_group_deployment_alias
WHERE tool_group_deployment_type_id = ? AND tool_group_deployment_deployment_id = ?
ORDER BY created_at DESC"#).await?;
      let mut rows = stmt.query(libsql::params![params.tool_group_deployment_type_id.clone(),params.tool_group_deployment_deployment_id.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_aliases_for_tool_group_deployment {
              alias: row.get(0)?,
          });
      }

      Ok(mapped)
  }
  pub struct update_tool_group_deployment_alias_params<'a> {
      pub tool_group_deployment_deployment_id: &'a 
          String
      ,
      pub tool_group_deployment_type_id: &'a 
          String
      ,
      pub alias: &'a 
          String
      ,
  }

  pub async fn update_tool_group_deployment_alias(
    conn: &shared::libsql::Connection
    ,params: update_tool_group_deployment_alias_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"UPDATE tool_group_deployment_alias
SET tool_group_deployment_deployment_id = ?, updated_at = CURRENT_TIMESTAMP
WHERE tool_group_deployment_type_id = ? AND alias = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.tool_group_deployment_deployment_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.tool_group_deployment_type_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.alias.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
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
      let mut stmt = conn.prepare(r#"SELECT id, type_id, metadata, value, created_at, updated_at, next_rotation_time, dek_alias
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
      let mut stmt = conn.prepare(r#"SELECT id, type_id, metadata, value, created_at, updated_at, next_rotation_time, dek_alias
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
  pub struct create_tool_group_params<'a> {
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
      pub tool_group_deployment_type_id: &'a 
          String
      ,
      pub credential_deployment_type_id: &'a 
          String
      ,
      pub status: &'a 
          String
      ,
      pub return_on_successful_brokering: &'a Option<
          shared::primitives::WrappedJsonValue
      >,
  }

  pub async fn create_tool_group(
    conn: &shared::libsql::Connection
    ,params: create_tool_group_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO tool_group (id, display_name, resource_server_credential_id, user_credential_id, created_at, updated_at, tool_group_deployment_type_id, credential_deployment_type_id, status, return_on_successful_brokering)
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
              <String as TryInto<libsql::Value>>::try_into(params.tool_group_deployment_type_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.credential_deployment_type_id.clone())
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
  pub struct update_tool_group_params<'a> {
      pub display_name: &'a 
          String
      ,
      pub id: &'a 
          String
      ,
  }

  pub async fn update_tool_group(
    conn: &shared::libsql::Connection
    ,params: update_tool_group_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"UPDATE tool_group SET display_name = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.display_name.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct update_tool_group_after_brokering_params<'a> {
      pub user_credential_id: &'a Option<
          shared::primitives::WrappedUuidV4
      >,
      pub id: &'a 
          String
      ,
  }

  pub async fn update_tool_group_after_brokering(
    conn: &shared::libsql::Connection
    ,params: update_tool_group_after_brokering_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"UPDATE tool_group SET user_credential_id = ?, status = 'active', updated_at = CURRENT_TIMESTAMP WHERE id = ?"#, libsql::params![
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
  pub struct get_tool_group_by_id_params<'a> {
      pub id: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_tool_group_by_id {
      pub id:String,
      pub display_name:String,
      pub resource_server_credential_id:shared::primitives::WrappedUuidV4,
      pub user_credential_id:Option<shared::primitives::WrappedUuidV4> ,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
      pub tool_group_deployment_type_id:String,
      pub credential_deployment_type_id:String,
      pub status:String,
      pub return_on_successful_brokering:Option<shared::primitives::WrappedJsonValue> ,
      pub functions:String,
      pub resource_server_credential:String,
      pub user_credential:String,
  }
  pub async fn get_tool_group_by_id(
      conn: &shared::libsql::Connection
      ,params: get_tool_group_by_id_params<'_>
  ) -> Result<Option<Row_get_tool_group_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT 
    pi.id,
    pi.display_name,
    pi.resource_server_credential_id,
    pi.user_credential_id,
    pi.created_at,
    pi.updated_at,
    pi.tool_group_deployment_type_id,
    pi.credential_deployment_type_id, pi.status, pi.return_on_successful_brokering,
    CAST(COALESCE(
        (SELECT JSON_GROUP_ARRAY(
            JSON_OBJECT(
                'tool_deployment_type_id', fi.tool_deployment_type_id,
                'tool_group_deployment_type_id', fi.tool_group_deployment_type_id,
                'tool_group_id', fi.tool_group_id,
                'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', fi.created_at),
                'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', fi.updated_at)
            )
        )
        FROM tool fi
        WHERE fi.tool_group_id = pi.id
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
FROM tool_group pi
WHERE pi.id = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_tool_group_by_id {
                  id: row.get(0)?,
                  display_name: row.get(1)?,
                  resource_server_credential_id: row.get(2)?,
                  user_credential_id: row.get(3)?,
                  created_at: row.get(4)?,
                  updated_at: row.get(5)?,
                  tool_group_deployment_type_id: row.get(6)?,
                  credential_deployment_type_id: row.get(7)?,
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
  pub struct delete_tool_group_params<'a> {
      pub id: &'a 
          String
      ,
  }

  pub async fn delete_tool_group(
    conn: &shared::libsql::Connection
    ,params: delete_tool_group_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM tool_group WHERE id = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
    ]).await
}
  pub struct create_tool_params<'a> {
      pub tool_deployment_type_id: &'a 
          String
      ,
      pub tool_group_deployment_type_id: &'a 
          String
      ,
      pub tool_group_id: &'a 
          String
      ,
      pub created_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
      pub updated_at: &'a 
          shared::primitives::WrappedChronoDateTime
      ,
  }

  pub async fn create_tool(
    conn: &shared::libsql::Connection
    ,params: create_tool_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"INSERT INTO tool (tool_deployment_type_id, tool_group_deployment_type_id, tool_group_id, created_at, updated_at)
VALUES (?, ?, ?, ?, ?)"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.tool_deployment_type_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.tool_group_deployment_type_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.tool_group_id.clone())
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
  pub struct get_tool_by_id_params<'a> {
      pub tool_deployment_type_id: &'a 
          String
      ,
      pub tool_group_deployment_type_id: &'a 
          String
      ,
      pub tool_group_id: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_tool_by_id {
      pub tool_deployment_type_id:String,
      pub tool_group_deployment_type_id:String,
      pub tool_group_id:String,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_tool_by_id(
      conn: &shared::libsql::Connection
      ,params: get_tool_by_id_params<'_>
  ) -> Result<Option<Row_get_tool_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT tool_deployment_type_id, tool_group_deployment_type_id, tool_group_id, created_at, updated_at
FROM tool
WHERE tool_deployment_type_id = ? AND tool_group_deployment_type_id = ? AND tool_group_id = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.tool_deployment_type_id.clone(),params.tool_group_deployment_type_id.clone(),params.tool_group_id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_tool_by_id {
                  tool_deployment_type_id: row.get(0)?,
                  tool_group_deployment_type_id: row.get(1)?,
                  tool_group_id: row.get(2)?,
                  created_at: row.get(3)?,
                  updated_at: row.get(4)?,
              })),
          Err(libsql::Error::QueryReturnedNoRows) => Ok(None),
          Err(e) => Err(e),
      }
  }
  pub struct delete_tool_params<'a> {
      pub tool_deployment_type_id: &'a 
          String
      ,
      pub tool_group_deployment_type_id: &'a 
          String
      ,
      pub tool_group_id: &'a 
          String
      ,
  }

  pub async fn delete_tool(
    conn: &shared::libsql::Connection
    ,params: delete_tool_params<'_>
) -> Result<u64, libsql::Error> {
    conn.execute(r#"DELETE FROM tool WHERE tool_deployment_type_id = ? AND tool_group_deployment_type_id = ? AND tool_group_id = ?"#, libsql::params![
              <String as TryInto<libsql::Value>>::try_into(params.tool_deployment_type_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.tool_group_deployment_type_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.tool_group_id.clone())
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
      pub tool_group_id: &'a 
          String
      ,
      pub tool_group_deployment_type_id: &'a 
          String
      ,
      pub credential_deployment_type_id: &'a 
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
    conn.execute(r#"INSERT INTO broker_state (id, created_at, updated_at, tool_group_id, tool_group_deployment_type_id, credential_deployment_type_id, metadata, action)
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
              <String as TryInto<libsql::Value>>::try_into(params.tool_group_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.tool_group_deployment_type_id.clone())
                  .map_err(|e| libsql::Error::ToSqlConversionFailure(e.into()))?
            ,
              <String as TryInto<libsql::Value>>::try_into(params.credential_deployment_type_id.clone())
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
      pub tool_group_id:String,
      pub tool_group_deployment_type_id:String,
      pub credential_deployment_type_id:String,
      pub metadata:crate::logic::Metadata,
      pub action:shared::primitives::WrappedJsonValue,
  }
  pub async fn get_broker_state_by_id(
      conn: &shared::libsql::Connection
      ,params: get_broker_state_by_id_params<'_>
  ) -> Result<Option<Row_get_broker_state_by_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT id, created_at, updated_at, tool_group_id, tool_group_deployment_type_id, credential_deployment_type_id, metadata, action
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
                  tool_group_id: row.get(3)?,
                  tool_group_deployment_type_id: row.get(4)?,
                  credential_deployment_type_id: row.get(5)?,
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
  pub struct get_tool_with_credentials_params<'a> {
      pub tool_deployment_type_id: &'a 
          String
      ,
      pub tool_group_deployment_type_id: &'a 
          String
      ,
      pub tool_group_id: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_tool_with_credentials {
      pub tool_tool_deployment_type_id:String,
      pub tool_tool_group_deployment_type_id:String,
      pub tool_tool_group_id:String,
      pub tool_created_at:shared::primitives::WrappedChronoDateTime,
      pub tool_updated_at:shared::primitives::WrappedChronoDateTime,
      pub tool_group_id:String,
      pub tool_group_display_name:String,
      pub tool_group_resource_server_credential_id:shared::primitives::WrappedUuidV4,
      pub tool_group_user_credential_id:Option<shared::primitives::WrappedUuidV4> ,
      pub tool_group_created_at:shared::primitives::WrappedChronoDateTime,
      pub tool_group_updated_at:shared::primitives::WrappedChronoDateTime,
      pub tool_group_tool_group_deployment_type_id:String,
      pub credential_deployment_type_id:String,
      pub tool_group_status:String,
      pub tool_group_return_on_successful_brokering:Option<shared::primitives::WrappedJsonValue> ,
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
  pub async fn get_tool_with_credentials(
      conn: &shared::libsql::Connection
      ,params: get_tool_with_credentials_params<'_>
  ) -> Result<Option<Row_get_tool_with_credentials>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT
    fi.tool_deployment_type_id as tool_tool_deployment_type_id,
    fi.tool_group_deployment_type_id as tool_tool_group_deployment_type_id,
    fi.tool_group_id as tool_tool_group_id,
    fi.created_at as tool_created_at,
    fi.updated_at as tool_updated_at,
    pi.id as tool_group_id,
    pi.display_name as tool_group_display_name,
    pi.resource_server_credential_id as tool_group_resource_server_credential_id,
    pi.user_credential_id as tool_group_user_credential_id,
    pi.created_at as tool_group_created_at,
    pi.updated_at as tool_group_updated_at,
    pi.tool_group_deployment_type_id as tool_group_tool_group_deployment_type_id,
    pi.credential_deployment_type_id,
    pi.status as tool_group_status,
    pi.return_on_successful_brokering as tool_group_return_on_successful_brokering,
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
FROM tool fi
JOIN tool_group pi ON fi.tool_group_id = pi.id
JOIN resource_server_credential rsc ON pi.resource_server_credential_id = rsc.id
LEFT JOIN user_credential uc ON pi.user_credential_id = uc.id
WHERE fi.tool_deployment_type_id = ? AND fi.tool_group_deployment_type_id = ? AND fi.tool_group_id = ?"#).await?;
      let res = stmt.query_row(
          libsql::params![params.tool_deployment_type_id.clone(),params.tool_group_deployment_type_id.clone(),params.tool_group_id.clone(),],
      ).await;

      match res {
          Ok(row) => Ok(Some(Row_get_tool_with_credentials {
                  tool_tool_deployment_type_id: row.get(0)?,
                  tool_tool_group_deployment_type_id: row.get(1)?,
                  tool_tool_group_id: row.get(2)?,
                  tool_created_at: row.get(3)?,
                  tool_updated_at: row.get(4)?,
                  tool_group_id: row.get(5)?,
                  tool_group_display_name: row.get(6)?,
                  tool_group_resource_server_credential_id: row.get(7)?,
                  tool_group_user_credential_id: row.get(8)?,
                  tool_group_created_at: row.get(9)?,
                  tool_group_updated_at: row.get(10)?,
                  tool_group_tool_group_deployment_type_id: row.get(11)?,
                  credential_deployment_type_id: row.get(12)?,
                  tool_group_status: row.get(13)?,
                  tool_group_return_on_successful_brokering: row.get(14)?,
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
  pub struct get_tool_groups_params<'a> {
      pub cursor: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub status: &'a Option<
          String
      >,
      pub tool_group_deployment_type_id: &'a Option<
          String
      >,
      pub page_size: &'a 
          i64
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_tool_groups {
      pub id:String,
      pub display_name:String,
      pub resource_server_credential_id:shared::primitives::WrappedUuidV4,
      pub user_credential_id:Option<shared::primitives::WrappedUuidV4> ,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
      pub tool_group_deployment_type_id:String,
      pub credential_deployment_type_id:String,
      pub status:String,
      pub return_on_successful_brokering:Option<shared::primitives::WrappedJsonValue> ,
      pub functions:String,
      pub resource_server_credential:String,
      pub user_credential:String,
  }
  pub async fn get_tool_groups(
      conn: &shared::libsql::Connection
      ,params: get_tool_groups_params<'_>
  ) -> Result<Vec<Row_get_tool_groups>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT
    pi.id,
    pi.display_name,
    pi.resource_server_credential_id,
    pi.user_credential_id,
    pi.created_at,
    pi.updated_at,
    pi.tool_group_deployment_type_id,
    pi.credential_deployment_type_id,
    pi.status,
    pi.return_on_successful_brokering,
    CAST(COALESCE(
        (SELECT JSON_GROUP_ARRAY(
            JSON_OBJECT(
                'tool_deployment_type_id', fi.tool_deployment_type_id,
                'tool_group_deployment_type_id', fi.tool_group_deployment_type_id,
                'tool_group_id', fi.tool_group_id,
                'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', fi.created_at),
                'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', fi.updated_at)
            )
        )
        FROM tool fi
        WHERE fi.tool_group_id = pi.id
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
FROM tool_group pi
WHERE (pi.created_at < ?1 OR ?1 IS NULL)
  AND (CAST(pi.status = ?2 AS TEXT) OR ?2 IS NULL)
  AND (CAST(pi.tool_group_deployment_type_id = ?3 AS TEXT) OR ?3 IS NULL)
ORDER BY pi.created_at DESC
LIMIT CAST(?4 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.cursor.clone(),params.status.clone(),params.tool_group_deployment_type_id.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_tool_groups {
              id: row.get(0)?,
              display_name: row.get(1)?,
              resource_server_credential_id: row.get(2)?,
              user_credential_id: row.get(3)?,
              created_at: row.get(4)?,
              updated_at: row.get(5)?,
              tool_group_deployment_type_id: row.get(6)?,
              credential_deployment_type_id: row.get(7)?,
              status: row.get(8)?,
              return_on_successful_brokering: row.get(9)?,
              functions: row.get(10)?,
              resource_server_credential: row.get(11)?,
              user_credential: row.get(12)?,
          });
      }

      Ok(mapped)
  }
  pub struct get_tools_params<'a> {
      pub cursor: &'a Option<
          shared::primitives::WrappedChronoDateTime
      >,
      pub tool_group_id: &'a Option<
          String
      >,
      pub page_size: &'a 
          i64
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_tools {
      pub tool_deployment_type_id:String,
      pub tool_group_deployment_type_id:String,
      pub tool_group_id:String,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
  }
  pub async fn get_tools(
      conn: &shared::libsql::Connection
      ,params: get_tools_params<'_>
  ) -> Result<Vec<Row_get_tools>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT tool_deployment_type_id, tool_group_deployment_type_id, tool_group_id, created_at, updated_at
FROM tool
WHERE (created_at < ?1 OR ?1 IS NULL)
  AND (CAST(tool_group_id = ?2 AS TEXT) OR ?2 IS NULL)
ORDER BY created_at DESC
LIMIT CAST(?3 AS INTEGER) + 1"#).await?;
      let mut rows = stmt.query(libsql::params![params.cursor.clone(),params.tool_group_id.clone(),params.page_size.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_tools {
              tool_deployment_type_id: row.get(0)?,
              tool_group_deployment_type_id: row.get(1)?,
              tool_group_id: row.get(2)?,
              created_at: row.get(3)?,
              updated_at: row.get(4)?,
          });
      }

      Ok(mapped)
  }
  pub struct get_tool_groups_grouped_by_tool_deployment_type_id_params<'a> {
      pub tool_deployment_type_ids: &'a 
          String
      ,
  }
    #[derive(Serialize, Deserialize, Debug)]

  #[allow(non_camel_case_types)]
  pub struct Row_get_tool_groups_grouped_by_tool_deployment_type_id {
      pub tool_deployment_type_id:String,
      pub tool_groups:String,
  }
  pub async fn get_tool_groups_grouped_by_tool_deployment_type_id(
      conn: &shared::libsql::Connection
      ,params: get_tool_groups_grouped_by_tool_deployment_type_id_params<'_>
  ) -> Result<Vec<Row_get_tool_groups_grouped_by_tool_deployment_type_id>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT
    fi.tool_deployment_type_id,
    CAST(
        JSON_GROUP_ARRAY(
            JSON_OBJECT(
                'id', pi.id,
                'display_name', pi.display_name,
                'tool_group_deployment_type_id', pi.tool_group_deployment_type_id,
                'credential_deployment_type_id', pi.credential_deployment_type_id,
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

                -- include tool metadata
                'tool', JSON_OBJECT(
                    'tool_group_deployment_type_id', fi.tool_group_deployment_type_id,
                    'tool_group_id', fi.tool_group_id,
                    'created_at', strftime('%Y-%m-%dT%H:%M:%fZ', fi.created_at),
                    'updated_at', strftime('%Y-%m-%dT%H:%M:%fZ', fi.updated_at)
                )
            )
        ) AS TEXT
    ) AS tool_groups
FROM tool fi
JOIN tool_group pi ON fi.tool_group_id = pi.id
WHERE (
    fi.tool_deployment_type_id IN (?1)
    OR ?1 IS NULL
)
GROUP BY fi.tool_deployment_type_id
ORDER BY fi.tool_deployment_type_id ASC"#).await?;
      let mut rows = stmt.query(libsql::params![params.tool_deployment_type_ids.clone(),]).await?;
      let mut mapped = vec![];

      while let Some(row) = rows.next().await? {
          mapped.push(Row_get_tool_groups_grouped_by_tool_deployment_type_id {
              tool_deployment_type_id: row.get(0)?,
              tool_groups: row.get(1)?,
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
  pub struct get_tool_groups_with_credentials_params<'a> {
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
  pub struct Row_get_tool_groups_with_credentials {
      pub id:String,
      pub display_name:String,
      pub tool_group_deployment_type_id:String,
      pub credential_deployment_type_id:String,
      pub status:String,
      pub return_on_successful_brokering:Option<shared::primitives::WrappedJsonValue> ,
      pub created_at:shared::primitives::WrappedChronoDateTime,
      pub updated_at:shared::primitives::WrappedChronoDateTime,
      pub resource_server_credential:String,
      pub user_credential:String,
  }
  pub async fn get_tool_groups_with_credentials(
      conn: &shared::libsql::Connection
      ,params: get_tool_groups_with_credentials_params<'_>
  ) -> Result<Vec<Row_get_tool_groups_with_credentials>, libsql::Error> {
      let mut stmt = conn.prepare(r#"SELECT
    pi.id,
    pi.display_name,
    pi.tool_group_deployment_type_id,
    pi.credential_deployment_type_id,
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
FROM tool_group pi
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
          mapped.push(Row_get_tool_groups_with_credentials {
              id: row.get(0)?,
              display_name: row.get(1)?,
              tool_group_deployment_type_id: row.get(2)?,
              credential_deployment_type_id: row.get(3)?,
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
