use std::{fmt, net::SocketAddr, str::FromStr};

use anyhow;
use base64::Engine;
use libsql::FromValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use utoipa::{
    IntoParams, PartialSchema, ToSchema,
    openapi::{Object, ObjectBuilder, OneOf, Schema, Type},
};


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

    pub fn to_string(&self) -> String {
        self.0.to_string()
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
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self(uuid::Uuid::parse_str(&value).unwrap()))
    }
}

impl libsql::FromValue for WrappedUuidV4 {
    fn from_sql(val: libsql::Value) -> libsql::Result<Self>
    where
        Self: Sized,
    {
        match val {
            libsql::Value::Text(s) => Ok(WrappedUuidV4::try_from(s).unwrap()),
            _ => unreachable!("invalid value type"),
        }
    }
}

impl From<libsql::Value> for WrappedUuidV4 {
    fn from(val: libsql::Value) -> Self {
        match val {
            libsql::Value::Text(s) => WrappedUuidV4::try_from(s).unwrap(),
            _ => unreachable!("invalid value type"),
        }
    }
}

impl From<WrappedUuidV4> for libsql::Value {
    fn from(val: WrappedUuidV4) -> Self {
        libsql::Value::Text(val.to_string())
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
            _ => unreachable!("invalid value type"),
        }
    }
}

impl TryFrom<libsql::Value> for WrappedJsonValue {
    type Error = anyhow::Error;

    fn try_from(val: libsql::Value) -> Result<Self, Self::Error> {
        match val {
            libsql::Value::Text(s) => Ok(WrappedJsonValue::new(
                serde_json::from_str(&s)
                    .map_err(|e| anyhow::anyhow!("invalid json value: {}", e))?,
            )),
            _ => Err(anyhow::anyhow!("invalid value type")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
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
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // Try SQLite datetime format first, then fall back to RFC3339
        let parsed = chrono::NaiveDateTime::parse_from_str(value.as_str(), "%Y-%m-%d %H:%M:%S%.f")
            .map(|naive| naive.and_utc())
            .or_else(|_| chrono::DateTime::parse_from_rfc3339(value.as_str()).map(|dt| dt.into()))
            .map_err(|_e| anyhow::anyhow!("invalid datetime value"))?;

        Ok(WrappedChronoDateTime::new(parsed))
    }
}

impl TryFrom<&str> for WrappedChronoDateTime {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Try SQLite datetime format first, then fall back to RFC3339
        let parsed = chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S%.f")
            .map(|naive| naive.and_utc())
            .or_else(|_| chrono::DateTime::parse_from_rfc3339(value).map(|dt| dt.into()))
            .map_err(|_e| anyhow::anyhow!("invalid datetime value"))?;

        Ok(WrappedChronoDateTime::new(parsed))
    }
}

impl ToString for WrappedChronoDateTime {
    fn to_string(&self) -> String {
        self.0.to_rfc3339()
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
            _ => unreachable!("invalid value type"),
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
                .required("next_page_token")
                .build(),
        ))
    }
}

/// Decode a base64-encoded pagination token back to a vector of strings
pub fn decode_pagination_token(token: &str) -> anyhow::Result<Vec<String>> {
    let decoded_bytes = base64::engine::general_purpose::STANDARD.decode(token)?;
    let decoded_str = String::from_utf8(decoded_bytes)?;
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
