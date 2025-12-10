{change_wanted}

in crates: {crate}

Ensure to review crates/$crate/dbs/*/schema.sql and crates/$crate/dbs/*/queries/*.sql. Follow this process to implement changes:
1. implement sql schema changes
2. add / edit queries in crates/$crate/dbs/*/queries/*.sql
3. use make file command to generate db migration for changes if there's a schema change
4. use make file command to generate new db hash
5. update crates/$crate/sqlc.yaml to map columns to specific rust types. Always use WrappedJsonValue, WrappedDatetime, WrappedUuid for json, date time, uuid columns. use custom rust enums for enum type columns from $crate, add them to logic folder in $crate if they dont exist.
6. run sqlc generate
7. update the repository trait in src/$crate/repository/mod.rs
8. implement any type conversions from the generated sqlc types to types in the logic folders in  src/$crate/repository/sqlite/raw_from.rs
9. implement the repository trait for sqlite using the generated functions
10. Add tests for the repository functions

For example:

`crates/bridge/dbs/bridge/schema.sql`:

```sql
CREATE TABLE IF NOT EXISTS user_credential (
    id TEXT PRIMARY KEY,
    type_id TEXT NOT NULL,
    metadata JSON NOT NULL,
    value JSON NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    next_rotation_time DATETIME,
    dek_alias TEXT NOT NULL
);
```
`crates/bridge/dbs/bridge/queries.sql`:

```sql

-- name: get_user_credentials :many
SELECT id, type_id, metadata, value, created_at, updated_at, next_rotation_time, dek_alias
FROM user_credential WHERE (created_at < sqlc.narg(cursor) OR sqlc.narg(cursor) IS NULL)
ORDER BY created_at DESC
LIMIT CAST(sqlc.arg(page_size) AS INTEGER) + 1;

-- name: create_user_credential :exec
INSERT INTO user_credential (id, type_id, metadata, value, created_at, updated_at, next_rotation_time, dek_alias)
VALUES (?, ?, ?, ?, ?, ?, ?, ?);

-- name: get_user_credential_by_id :one
SELECT id, type_id, metadata, value, created_at, updated_at, next_rotation_time, dek_alias
FROM user_credential
WHERE id = ?;

-- name: delete_user_credential :exec
DELETE FROM user_credential WHERE id = ?;
```

`crates/bridge/sqlc.yaml`:

```yaml
....
            # user_credential table
            - db_type: text
                column: user_credential.id
                rust_type: "shared::primitives::WrappedUuidV4"
            - db_type: json
                column: user_credential.metadata
                rust_type: "crate::logic::Metadata"
            - db_type: json
                column: user_credential.value
                rust_type: "shared::primitives::WrappedJsonValue"
            - db_type: datetime
                column: user_credential.created_at
                rust_type: "shared::primitives::WrappedChronoDateTime"
            - db_type: datetime
                column: user_credential.updated_at
                rust_type: "shared::primitives::WrappedChronoDateTime"
            - db_type: datetime
                column: user_credential.next_rotation_time
                rust_type: "shared::primitives::WrappedChronoDateTime"

....
```

`$ cd crates/bridge && sqlc generate`

`crates/bridge/src/repository/sqlite/raw_impl.rs`
```rust
use super::{ Row_get_user_credential_by_id, Row_get_user_credentials}


// Helper function to deserialize optional user credential JSON object
fn deserialize_user_credential(
    json_value: &str,
) -> Result<Option<UserCredentialSerialized>, CommonError> {
    if json_value.is_empty() || json_value == "null" || json_value.trim().is_empty() {
        return Ok(None);
    }

    let cred: UserCredentialSerialized =
        serde_json::from_str(json_value).map_err(|e| CommonError::Repository {
            msg: format!("Failed to deserialize user credential JSON: {e}"),
            source: Some(e.into()),
        })?;
    Ok(Some(cred))
}


impl TryFrom<Row_get_user_credential_by_id> for UserCredentialSerialized {
    type Error = CommonError;
    fn try_from(row: Row_get_user_credential_by_id) -> Result<Self, Self::Error> {
        Ok(UserCredentialSerialized {
            id: row.id,
            type_id: row.type_id,
            metadata: row.metadata,
            value: row.value,
            created_at: row.created_at,
            updated_at: row.updated_at,
            next_rotation_time: row.next_rotation_time,
            dek_alias: row.dek_alias,
        })
    }
}

impl TryFrom<Row_get_user_credentials> for UserCredentialSerialized {
    type Error = CommonError;
    fn try_from(row: Row_get_user_credentials) -> Result<Self, Self::Error> {
        Ok(UserCredentialSerialized {
            id: row.id,
            type_id: row.type_id,
            metadata: row.metadata,
            value: row.value,
            created_at: row.created_at,
            updated_at: row.updated_at,
            next_rotation_time: row.next_rotation_time,
            dek_alias: row.dek_alias,
        })
    }
}
```

`crates/bridge/src/repository/mod.rs`

```rust
use crate::logic::credential::{
    UserCredentialSerialized,
};


// Repository trait
#[allow(async_fn_in_trait)]
pub trait ProviderRepositoryLike {
    
    async fn create_user_credential(
        &self,
        params: &CreateUserCredential,
    ) -> Result<(), CommonError>;

    async fn get_user_credential_by_id(
        &self,
        id: &WrappedUuidV4,
    ) -> Result<Option<UserCredentialSerialized>, CommonError>;

    async fn delete_user_credential(&self, id: &WrappedUuidV4) -> Result<(), CommonError>;

    async fn list_user_credentials(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<UserCredentialSerialized>, CommonError>;
}
```

`crates/bridge/src/logic/credential/mod.rs`

```rust
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct UserCredentialSerialized {
    pub id: WrappedUuidV4,
    pub type_id: String,
    pub metadata: Metadata,
    pub value: WrappedJsonValue,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
    pub next_rotation_time: Option<WrappedChronoDateTime>,
    pub dek_alias: String,
}
```


This example has the serialized logic struct because it deals with WrappedJsonValue