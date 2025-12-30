//! Row conversion implementations for SQLC-generated types

use crate::logic::{secret::Secret, variable::Variable};
use shared::error::CommonError;

use super::{
    Row_get_secret_by_id, Row_get_secret_by_key, Row_get_secrets, Row_get_variable_by_id,
    Row_get_variable_by_key, Row_get_variables,
};

// Secret conversions
impl TryFrom<Row_get_secret_by_id> for Secret {
    type Error = CommonError;
    fn try_from(row: Row_get_secret_by_id) -> Result<Self, Self::Error> {
        Ok(Secret {
            id: row.id,
            key: row.key,
            encrypted_secret: row.encrypted_secret,
            dek_alias: row.dek_alias,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<Row_get_secret_by_key> for Secret {
    type Error = CommonError;
    fn try_from(row: Row_get_secret_by_key) -> Result<Self, Self::Error> {
        Ok(Secret {
            id: row.id,
            key: row.key,
            encrypted_secret: row.encrypted_secret,
            dek_alias: row.dek_alias,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<Row_get_secrets> for Secret {
    type Error = CommonError;
    fn try_from(row: Row_get_secrets) -> Result<Self, Self::Error> {
        Ok(Secret {
            id: row.id,
            key: row.key,
            encrypted_secret: row.encrypted_secret,
            dek_alias: row.dek_alias,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

// Variable conversions
impl TryFrom<Row_get_variable_by_id> for Variable {
    type Error = CommonError;
    fn try_from(row: Row_get_variable_by_id) -> Result<Self, Self::Error> {
        Ok(Variable {
            id: row.id,
            key: row.key,
            value: row.value,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<Row_get_variable_by_key> for Variable {
    type Error = CommonError;
    fn try_from(row: Row_get_variable_by_key) -> Result<Self, Self::Error> {
        Ok(Variable {
            id: row.id,
            key: row.key,
            value: row.value,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<Row_get_variables> for Variable {
    type Error = CommonError;
    fn try_from(row: Row_get_variables) -> Result<Self, Self::Error> {
        Ok(Variable {
            id: row.id,
            key: row.key,
            value: row.value,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
