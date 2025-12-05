use std::{fmt, str::FromStr};

use anyhow;
use base64::Engine;
use libsql::FromValue;
use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use utoipa::{
    IntoParams, PartialSchema, ToSchema,
    openapi::{ObjectBuilder, Type, schema::AdditionalProperties},
};

use crate::error::CommonError;

pub type WrappedNodeId = String;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(transparent)]
pub struct WrappedUuidV4(uuid::Uuid);

impl Default for WrappedUuidV4 {
    fn default() -> Self {
        Self::new()
    }
}

impl WrappedUuidV4 {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl FromStr for WrappedUuidV4 {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(uuid::Uuid::parse_str(s)?))
    }
}

// impl From<WrappedUuidV4> for libsql::Value {
//     fn from(value: WrappedUuidV4) -> Self {
//         return value.0.to_string().into();
//     }
// }

impl fmt::Display for WrappedUuidV4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for WrappedUuidV4 {
    type Error = CommonError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self(uuid::Uuid::parse_str(&value).map_err(|_e| {
            CommonError::InvalidRequest {
                msg: "invalid uuid".to_string(),
                source: None,
            }
        })?))
    }
}

impl libsql::FromValue for WrappedUuidV4 {
    fn from_sql(val: libsql::Value) -> libsql::Result<Self>
    where
        Self: Sized,
    {
        match val {
            libsql::Value::Text(s) => Ok(WrappedUuidV4::try_from(s).unwrap()),
            libsql::Value::Null => Err(libsql::Error::NullValue),
            _ => Err(libsql::Error::InvalidColumnType),
        }
    }
}

impl From<libsql::Value> for WrappedUuidV4 {
    fn from(val: libsql::Value) -> Self {
        match val {
            libsql::Value::Text(s) => WrappedUuidV4::try_from(s).unwrap(),
            _ => panic!("Cannot convert {val:?} to WrappedUuidV4"),
        }
    }
}

impl From<WrappedUuidV4> for libsql::Value {
    fn from(val: WrappedUuidV4) -> Self {
        libsql::Value::Text(val.0.to_string())
    }
}

pub type LoadSqlMigrationsCallback =
    fn() -> BTreeMap<&'static str, BTreeMap<&'static str, &'static str>>;

pub trait SqlMigrationLoader {
    fn load_sql_migrations() -> BTreeMap<&'static str, BTreeMap<&'static str, &'static str>>;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct WrappedJsonValue(serde_json::Value);

impl WrappedJsonValue {
    pub fn get_inner(&self) -> &serde_json::Value {
        &self.0
    }

    pub fn into_inner(self) -> serde_json::Value {
        self.0
    }
}

impl WrappedJsonValue {
    pub fn new(value: serde_json::Value) -> Self {
        Self(value)
    }
}

impl From<serde_json::Value> for WrappedJsonValue {
    fn from(value: serde_json::Value) -> Self {
        Self(value)
    }
}

impl From<WrappedJsonValue> for libsql::Value {
    fn from(value: WrappedJsonValue) -> Self {
        libsql::Value::Text(serde_json::to_string(&value.0).unwrap())
    }
}

impl From<WrappedJsonValue> for serde_json::Value {
    fn from(value: WrappedJsonValue) -> Self {
        value.0
    }
}

impl libsql::FromValue for WrappedJsonValue {
    fn from_sql(val: libsql::Value) -> libsql::Result<Self>
    where
        Self: Sized,
    {
        match val {
            libsql::Value::Text(s) => Ok(WrappedJsonValue::new(
                serde_json::from_str(&s).map_err(|_e| libsql::Error::InvalidColumnType)?,
            )),
            libsql::Value::Null => Err(libsql::Error::NullValue),
            _ => Err(libsql::Error::InvalidColumnType),
        }
    }
}

impl TryInto<WrappedJsonValue> for Schema {
    type Error = Box<dyn std::error::Error + Send + Sync>;
    fn try_into(self) -> Result<WrappedJsonValue, Self::Error> {
        let json_value = serde_json::to_value(self)?;
        Ok(WrappedJsonValue::from(json_value))
    }
}

impl TryFrom<libsql::Value> for WrappedJsonValue {
    type Error = CommonError;

    fn try_from(val: libsql::Value) -> Result<Self, Self::Error> {
        match val {
            libsql::Value::Text(s) => Ok(WrappedJsonValue::new(serde_json::from_str(&s).map_err(
                |e| CommonError::InvalidRequest {
                    msg: format!("invalid json value: {e}"),
                    source: None,
                },
            )?)),
            _ => Err(CommonError::InvalidRequest {
                msg: "invalid value type".to_string(),
                source: None,
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(transparent)]
pub struct WrappedChronoDateTime(chrono::DateTime<chrono::Utc>);

impl WrappedChronoDateTime {
    pub fn get_inner(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.0
    }

    pub fn new(value: chrono::DateTime<chrono::Utc>) -> Self {
        Self(value)
    }

    pub fn now() -> Self {
        Self(chrono::Utc::now())
    }
}

impl TryFrom<String> for WrappedChronoDateTime {
    type Error = CommonError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // Try SQLite datetime format first, then fall back to RFC3339
        let parsed = chrono::NaiveDateTime::parse_from_str(value.as_str(), "%Y-%m-%d %H:%M:%S%.f")
            .map(|naive| naive.and_utc())
            .or_else(|_| chrono::DateTime::parse_from_rfc3339(value.as_str()).map(|dt| dt.into()))
            .map_err(|_e| CommonError::InvalidRequest {
                msg: "invalid datetime value".to_string(),
                source: None,
            })?;

        Ok(WrappedChronoDateTime::new(parsed))
    }
}

impl TryFrom<&str> for WrappedChronoDateTime {
    type Error = CommonError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Try SQLite datetime format first, then fall back to RFC3339
        let parsed = chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S%.f")
            .map(|naive| naive.and_utc())
            .or_else(|_| chrono::DateTime::parse_from_rfc3339(value).map(|dt| dt.into()))
            .map_err(|_e| CommonError::InvalidRequest {
                msg: "invalid datetime value".to_string(),
                source: None,
            })?;

        Ok(WrappedChronoDateTime::new(parsed))
    }
}

impl fmt::Display for WrappedChronoDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_rfc3339())
    }
}

impl From<chrono::DateTime<chrono::Utc>> for WrappedChronoDateTime {
    fn from(value: chrono::DateTime<chrono::Utc>) -> Self {
        Self(value)
    }
}

impl libsql::FromValue for WrappedChronoDateTime {
    fn from_sql(val: libsql::Value) -> libsql::Result<Self>
    where
        Self: Sized,
    {
        match val {
            libsql::Value::Text(s) => {
                // Try SQLite datetime format first, then fall back to RFC3339
                let parsed =
                    chrono::NaiveDateTime::parse_from_str(s.as_str(), "%Y-%m-%d %H:%M:%S%.f")
                        .map(|naive| naive.and_utc())
                        .or_else(|_| {
                            chrono::DateTime::parse_from_rfc3339(s.as_str()).map(|dt| dt.into())
                        })
                        .map_err(|_e| libsql::Error::InvalidColumnType)?;

                Ok(WrappedChronoDateTime::new(parsed))
            }
            libsql::Value::Null => Err(libsql::Error::NullValue),
            _ => Err(libsql::Error::InvalidColumnType),
        }
    }
}

impl From<libsql::Value> for WrappedChronoDateTime {
    fn from(value: libsql::Value) -> Self {
        Self::from_sql(value).unwrap()
    }
}

impl From<WrappedChronoDateTime> for chrono::DateTime<chrono::Utc> {
    fn from(value: WrappedChronoDateTime) -> Self {
        value.0
    }
}

impl From<WrappedChronoDateTime> for libsql::Value {
    fn from(value: WrappedChronoDateTime) -> Self {
        // Use SQLite's expected datetime format instead of RFC3339
        libsql::Value::Text(value.0.format("%Y-%m-%d %H:%M:%S%.f").to_string())
    }
}

// Pagination types
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, IntoParams)]
#[into_params(style = Form, parameter_in = Query)]
pub struct PaginationRequest {
    pub page_size: i64,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PaginatedResponse<T: ToSchema + Serialize> {
    pub items: Vec<T>,
    pub next_page_token: Option<String>,
}

impl<T: ToSchema + Serialize> ToSchema for PaginatedResponse<T> {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Owned(format!("{}PaginatedResponse", T::name()))
    }

    fn schemas(
        schemas: &mut Vec<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>,
    ) {
        schemas.push((T::name().to_string(), T::schema()));
        T::schemas(schemas);
        schemas.push((format!("{}PaginatedResponse", T::name()), Self::schema()));
    }
}

impl<T: ToSchema + Serialize> PartialSchema for PaginatedResponse<T> {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::RefOr::T(utoipa::openapi::schema::Schema::Object(
            ObjectBuilder::new()
                .schema_type(Type::Object)
                .property(
                    "items",
                    utoipa::openapi::ArrayBuilder::new()
                        .schema_type(utoipa::openapi::schema::Type::Array)
                        .items(utoipa::openapi::schema::Ref::from_schema_name(T::name())),
                )
                .property(
                    "next_page_token",
                    utoipa::openapi::ObjectBuilder::new()
                        .schema_type(utoipa::openapi::schema::Type::String),
                )
                .required("items")
                .build(),
        ))
    }
}

/// Decode a base64-encoded pagination token back to a vector of strings
pub fn decode_pagination_token(token: &str) -> Result<Vec<String>, CommonError> {
    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(token)
        .map_err(|_e| CommonError::InvalidRequest {
            msg: "invalid base64 string".to_string(),
            source: None,
        })?;
    let decoded_str =
        String::from_utf8(decoded_bytes).map_err(|_e| CommonError::InvalidRequest {
            msg: "invalid utf8 string".to_string(),
            source: None,
        })?;
    Ok(decoded_str.split("__").map(|s| s.to_string()).collect())
}

impl<T: ToSchema + Serialize> PaginatedResponse<T> {
    /// Create a paginated response from a list of items fetched with `page_size + 1`.
    ///
    /// This function expects that you fetched `page_size + 1` items from the database.
    /// It will:
    /// - Check if there are more items than `page_size` (indicating more pages exist)
    /// - Remove the extra item if present
    /// - Generate the next page token from the last item's composite key
    ///
    /// # Arguments
    /// * `items` - The list of items fetched (should be `page_size + 1` items)
    /// * `pagination` - The original pagination request
    /// * `get_id` - A closure that extracts a vector of strings (composite key) from an item
    pub fn from_items_with_extra<F>(
        mut items: Vec<T>,
        pagination: &PaginationRequest,
        get_id: F,
    ) -> Self
    where
        F: FnOnce(&T) -> Vec<String>,
    {
        // Check if we got more items than requested (page_size + 1)
        let has_more = items.len() as i64 > pagination.page_size;

        // If we have more items than page_size, remove the extra item
        if has_more {
            items.pop();
        }

        let next_page_token = if has_more && !items.is_empty() {
            items.last().map(|item| {
                let key_parts = get_id(item);
                let composite_key = key_parts.join("__");
                base64::engine::general_purpose::STANDARD.encode(composite_key.as_bytes())
            })
        } else {
            None
        };

        Self {
            items,
            next_page_token,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct WrappedSchema(schemars::Schema);

impl WrappedSchema {
    pub fn new(value: schemars::Schema) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> schemars::Schema {
        self.0
    }

    pub fn get_inner(&self) -> &schemars::Schema {
        &self.0
    }
}
impl From<schemars::Schema> for WrappedSchema {
    fn from(value: schemars::Schema) -> Self {
        Self(value)
    }
}

impl From<WrappedSchema> for libsql::Value {
    fn from(value: WrappedSchema) -> Self {
        libsql::Value::Text(serde_json::to_string(&value.0.as_value().to_string()).unwrap())
    }
}

impl From<WrappedSchema> for schemars::Schema {
    fn from(value: WrappedSchema) -> Self {
        value.0
    }
}

impl libsql::FromValue for WrappedSchema {
    fn from_sql(val: libsql::Value) -> libsql::Result<Self>
    where
        Self: Sized,
    {
        let s = match val {
            libsql::Value::Text(s) => s,
            libsql::Value::Null => return Err(libsql::Error::NullValue),
            _ => return Err(libsql::Error::InvalidColumnType),
        };
        let value: serde_json::Value =
            serde_json::from_str(&s).map_err(|_e| libsql::Error::InvalidColumnType)?;
        let schema =
            schemars::Schema::try_from(value).map_err(|_e| libsql::Error::InvalidColumnType)?;
        Ok(WrappedSchema::new(schema))
    }
}

// impl TryInto<WrappedSchema> for schemars::Schema {
//     type Error = Box<dyn std::error::Error + Send + Sync>;
//     fn try_into(self) -> Result<WrappedSchema, Self::Error> {
//         Ok(WrappedSchema::new(self))
//     }
// }

impl TryFrom<libsql::Value> for WrappedSchema {
    type Error = CommonError;

    fn try_from(val: libsql::Value) -> Result<Self, Self::Error> {
        let s = match val {
            libsql::Value::Text(s) => s,
            libsql::Value::Null => {
                return Err(CommonError::InvalidRequest {
                    msg: "null value".to_string(),
                    source: None,
                });
            }
            _ => {
                return Err(CommonError::InvalidRequest {
                    msg: "invalid value type".to_string(),
                    source: None,
                });
            }
        };
        let value: serde_json::Value =
            serde_json::from_str(&s).map_err(|e| CommonError::InvalidRequest {
                msg: format!("invalid json value: {e}"),
                source: None,
            })?;
        let schema =
            schemars::Schema::try_from(value).map_err(|e| CommonError::InvalidRequest {
                msg: format!("invalid schema value: {e}"),
                source: None,
            })?;
        Ok(WrappedSchema::new(schema))
    }
}

impl ToSchema for WrappedSchema {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Owned("JsonSchema".to_string())
    }

    fn schemas(
        schemas: &mut Vec<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>,
    ) {
        schemas.push((Self::name().to_string(), Self::schema()));
    }
}

impl PartialSchema for WrappedSchema {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::RefOr::T(utoipa::openapi::schema::Schema::Object(
            ObjectBuilder::new()
                .schema_type(Type::Object)
                .additional_properties(Some(AdditionalProperties::FreeForm(true)))
                .build(),
        ))
    }
}
