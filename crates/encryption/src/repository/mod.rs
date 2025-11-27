mod sqlite;

pub use sqlite::Repository;

use std::str::FromStr;

use shared::{
    error::CommonError,
    primitives::{PaginatedResponse, PaginationRequest, WrappedChronoDateTime},
};

use crate::logic::dek::{DataEncryptionKey, DataEncryptionKeyListItem, EncryptedDataEncryptionKey};
use crate::logic::envelope::EnvelopeEncryptionKey;

/// Envelope encryption key type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EnvelopeEncryptionKeyType {
    Local,
    AwsKms,
}

impl EnvelopeEncryptionKeyType {
    /// Convert to string representation for database storage
    pub fn as_str(&self) -> &'static str {
        match self {
            EnvelopeEncryptionKeyType::Local => "local",
            EnvelopeEncryptionKeyType::AwsKms => "aws_kms",
        }
    }
}

impl std::str::FromStr for EnvelopeEncryptionKeyType {
    type Err = CommonError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "local" => Ok(EnvelopeEncryptionKeyType::Local),
            "aws_kms" => Ok(EnvelopeEncryptionKeyType::AwsKms),
            _ => Err(CommonError::Unknown(anyhow::anyhow!(
                "Invalid key_type: {s}"
            ))),
        }
    }
}

impl From<EnvelopeEncryptionKeyType> for libsql::Value {
    fn from(value: EnvelopeEncryptionKeyType) -> Self {
        libsql::Value::Text(value.as_str().to_string())
    }
}

impl TryFrom<libsql::Value> for EnvelopeEncryptionKeyType {
    type Error = CommonError;

    fn try_from(value: libsql::Value) -> Result<Self, Self::Error> {
        match value {
            libsql::Value::Text(s) => EnvelopeEncryptionKeyType::from_str(&s),
            libsql::Value::Null => Err(CommonError::Repository {
                msg: "key_type cannot be null".to_string(),
                source: None,
            }),
            _ => Err(CommonError::Repository {
                msg: "Invalid value type for key_type".to_string(),
                source: None,
            }),
        }
    }
}

impl libsql::FromValue for EnvelopeEncryptionKeyType {
    fn from_sql(val: libsql::Value) -> libsql::Result<Self>
    where
        Self: Sized,
    {
        match val {
            libsql::Value::Text(s) => EnvelopeEncryptionKeyType::from_str(&s)
                .map_err(|_e| libsql::Error::InvalidColumnType),
            libsql::Value::Null => Err(libsql::Error::NullValue),
            _ => Err(libsql::Error::InvalidColumnType),
        }
    }
}

// Repository parameter structs for envelope encryption key
#[derive(Debug, Clone)]
pub struct EnvelopeEncryptionKeyRow {
    pub id: String,
    pub key_type: EnvelopeEncryptionKeyType,
    pub local_file_name: Option<String>,
    pub aws_arn: Option<String>,
    pub aws_region: Option<String>,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

#[derive(Debug)]
pub struct CreateEnvelopeEncryptionKey {
    pub id: String,
    pub key_type: EnvelopeEncryptionKeyType,
    pub local_file_name: Option<String>,
    pub aws_arn: Option<String>,
    pub aws_region: Option<String>,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

impl From<EnvelopeEncryptionKeyRow> for CreateEnvelopeEncryptionKey {
    fn from(key: EnvelopeEncryptionKeyRow) -> Self {
        CreateEnvelopeEncryptionKey {
            id: key.id,
            key_type: key.key_type,
            local_file_name: key.local_file_name,
            aws_arn: key.aws_arn,
            aws_region: key.aws_region,
            created_at: key.created_at,
            updated_at: key.updated_at,
        }
    }
}

impl From<(EnvelopeEncryptionKey, WrappedChronoDateTime)> for CreateEnvelopeEncryptionKey {
    fn from((key, now): (EnvelopeEncryptionKey, WrappedChronoDateTime)) -> Self {
        // Extract the actual ID (ARN for AWS KMS, location for local)
        let (id, key_type, local_file_name, aws_arn, aws_region) = match &key {
            EnvelopeEncryptionKey::AwsKms(aws_kms) => (
                aws_kms.arn.clone(), // Use ARN as the ID
                EnvelopeEncryptionKeyType::AwsKms,
                None,
                Some(aws_kms.arn.clone()),
                Some(aws_kms.region.clone()),
            ),
            EnvelopeEncryptionKey::Local(local) => (
                local.file_name.clone(), // Use file_name as the ID
                EnvelopeEncryptionKeyType::Local,
                Some(local.file_name.clone()),
                None,
                None,
            ),
        };

        CreateEnvelopeEncryptionKey {
            id,
            key_type,
            local_file_name,
            aws_arn,
            aws_region,
            created_at: now,
            updated_at: now,
        }
    }
}

// Repository parameter structs for data encryption key
#[derive(Debug)]
pub struct CreateDataEncryptionKey {
    pub id: String,
    pub envelope_encryption_key_id: String, // String reference to envelope_encryption_key.id
    pub encryption_key: EncryptedDataEncryptionKey,
    pub created_at: WrappedChronoDateTime,
    pub updated_at: WrappedChronoDateTime,
}

impl From<DataEncryptionKey> for CreateDataEncryptionKey {
    fn from(dek: DataEncryptionKey) -> Self {
        // Convert EnvelopeEncryptionKey enum to string identifier
        let envelope_key_id = match &dek.envelope_encryption_key_id {
            EnvelopeEncryptionKey::AwsKms(aws_kms) => aws_kms.arn.clone(),
            EnvelopeEncryptionKey::Local(local) => local.file_name.clone(),
        };
        CreateDataEncryptionKey {
            id: dek.id,
            envelope_encryption_key_id: envelope_key_id,
            encryption_key: dek.encrypted_data_encryption_key,
            created_at: dek.created_at,
            updated_at: dek.updated_at,
        }
    }
}

// Repository trait for encryption key management
#[allow(async_fn_in_trait)]
pub trait EncryptionKeyRepositoryLike {
    async fn create_envelope_encryption_key(
        &self,
        params: &CreateEnvelopeEncryptionKey,
    ) -> Result<(), CommonError>;

    async fn get_envelope_encryption_key_by_id(
        &self,
        id: &str,
    ) -> Result<Option<EnvelopeEncryptionKey>, CommonError>;

    async fn list_envelope_encryption_keys(
        &self,
    ) -> Result<Vec<EnvelopeEncryptionKey>, CommonError>;

    async fn list_envelope_encryption_keys_paginated(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<EnvelopeEncryptionKey>, CommonError>;

    async fn delete_envelope_encryption_key(&self, id: &str) -> Result<(), CommonError>;

    async fn create_data_encryption_key(
        &self,
        params: &CreateDataEncryptionKey,
    ) -> Result<(), CommonError>;

    async fn get_data_encryption_key_by_id(
        &self,
        id: &str,
    ) -> Result<Option<DataEncryptionKey>, CommonError>;

    async fn delete_data_encryption_key(&self, id: &str) -> Result<(), CommonError>;

    async fn list_data_encryption_keys(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<DataEncryptionKeyListItem>, CommonError>;
}

/// Data encryption key alias struct
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct DataEncryptionKeyAlias {
    pub alias: String,
    pub data_encryption_key_id: String,
    pub created_at: WrappedChronoDateTime,
}

// Repository trait for data encryption key management
#[async_trait::async_trait]
pub trait DataEncryptionKeyRepositoryLike: Send + Sync {
    async fn create_data_encryption_key(
        &self,
        data_encryption_key: &DataEncryptionKey,
    ) -> Result<(), CommonError>;

    async fn get_data_encryption_key_by_id(
        &self,
        id: &str,
    ) -> Result<Option<DataEncryptionKey>, CommonError>;

    async fn list_data_encryption_keys(
        &self,
        params: &PaginationRequest,
    ) -> Result<PaginatedResponse<DataEncryptionKeyListItem>, CommonError>;

    async fn delete_data_encryption_key(&self, id: &str) -> Result<(), CommonError>;

    // Alias management methods
    async fn create_data_encryption_key_alias(
        &self,
        alias: &DataEncryptionKeyAlias,
    ) -> Result<(), CommonError>;

    async fn get_data_encryption_key_alias_by_alias(
        &self,
        alias: &str,
    ) -> Result<Option<DataEncryptionKeyAlias>, CommonError>;

    async fn get_data_encryption_key_by_alias(
        &self,
        alias: &str,
    ) -> Result<Option<DataEncryptionKey>, CommonError>;

    async fn delete_data_encryption_key_alias(&self, alias: &str) -> Result<(), CommonError>;

    async fn list_aliases_for_dek(
        &self,
        dek_id: &str,
    ) -> Result<Vec<DataEncryptionKeyAlias>, CommonError>;

    async fn update_data_encryption_key_alias(
        &self,
        alias: &str,
        new_dek_id: &str,
    ) -> Result<(), CommonError>;
}
