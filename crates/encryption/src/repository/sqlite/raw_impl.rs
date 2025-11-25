use shared::error::CommonError;

// Import generated Row types from parent module
use super::{
    Row_get_envelope_encryption_key_by_id,
    Row_get_envelope_encryption_keys,
    Row_get_envelope_encryption_keys_paginated,
    Row_get_all_data_encryption_keys_with_envelope_keys,
    Row_get_data_encryption_key_by_id_with_envelope,
};

// Conversion from repository EnvelopeEncryptionKey row types to logic EnvelopeEncryptionKey enum
use crate::logic::envelope::EnvelopeEncryptionKey as LogicEnvelopeEncryptionKey;

impl TryFrom<Row_get_envelope_encryption_key_by_id> for LogicEnvelopeEncryptionKey {
    type Error = CommonError;

    fn try_from(row: Row_get_envelope_encryption_key_by_id) -> Result<Self, Self::Error> {
        match row.key_type {
            crate::repository::EnvelopeEncryptionKeyType::AwsKms => {
                let arn = row.aws_arn.ok_or_else(|| {
                    CommonError::Unknown(anyhow::anyhow!("AWS KMS key missing ARN"))
                })?;
                let region = row.aws_region.ok_or_else(|| {
                    CommonError::Unknown(anyhow::anyhow!("AWS KMS key missing region"))
                })?;
                Ok(LogicEnvelopeEncryptionKey::AwsKms { arn, region })
            }
            crate::repository::EnvelopeEncryptionKeyType::Local => {
                let location = row.local_location.ok_or_else(|| {
                    CommonError::Unknown(anyhow::anyhow!("Local key missing location"))
                })?;
                Ok(LogicEnvelopeEncryptionKey::Local { location })
            }
        }
    }
}

impl TryFrom<Row_get_envelope_encryption_keys> for LogicEnvelopeEncryptionKey {
    type Error = CommonError;

    fn try_from(row: Row_get_envelope_encryption_keys) -> Result<Self, Self::Error> {
        match row.key_type {
            crate::repository::EnvelopeEncryptionKeyType::AwsKms => {
                let arn = row.aws_arn.ok_or_else(|| {
                    CommonError::Unknown(anyhow::anyhow!("AWS KMS key missing ARN"))
                })?;
                let region = row.aws_region.ok_or_else(|| {
                    CommonError::Unknown(anyhow::anyhow!("AWS KMS key missing region"))
                })?;
                Ok(LogicEnvelopeEncryptionKey::AwsKms { arn, region })
            }
            crate::repository::EnvelopeEncryptionKeyType::Local => {
                let location = row.local_location.ok_or_else(|| {
                    CommonError::Unknown(anyhow::anyhow!("Local key missing location"))
                })?;
                Ok(LogicEnvelopeEncryptionKey::Local { location })
            }
        }
    }
}

impl TryFrom<Row_get_envelope_encryption_keys_paginated> for LogicEnvelopeEncryptionKey {
    type Error = CommonError;

    fn try_from(row: Row_get_envelope_encryption_keys_paginated) -> Result<Self, Self::Error> {
        match row.key_type {
            crate::repository::EnvelopeEncryptionKeyType::AwsKms => {
                let arn = row.aws_arn.ok_or_else(|| {
                    CommonError::Unknown(anyhow::anyhow!("AWS KMS key missing ARN"))
                })?;
                let region = row.aws_region.ok_or_else(|| {
                    CommonError::Unknown(anyhow::anyhow!("AWS KMS key missing region"))
                })?;
                Ok(LogicEnvelopeEncryptionKey::AwsKms { arn, region })
            }
            crate::repository::EnvelopeEncryptionKeyType::Local => {
                let location = row.local_location.ok_or_else(|| {
                    CommonError::Unknown(anyhow::anyhow!("Local key missing location"))
                })?;
                Ok(LogicEnvelopeEncryptionKey::Local { location })
            }
        }
    }
}

// Conversion from repository row with DEK and envelope key to logic struct
use crate::logic::dek::DataEncryptionKey as LogicDataEncryptionKey;

impl TryFrom<Row_get_all_data_encryption_keys_with_envelope_keys> for LogicDataEncryptionKey {
    type Error = CommonError;

    fn try_from(row: Row_get_all_data_encryption_keys_with_envelope_keys) -> Result<Self, Self::Error> {
        // Convert envelope encryption key from row fields
        let envelope_encryption_key_id = match row.key_type {
            crate::repository::EnvelopeEncryptionKeyType::AwsKms => {
                let arn = row.aws_arn.ok_or_else(|| {
                    CommonError::Unknown(anyhow::anyhow!("AWS KMS key missing ARN"))
                })?;
                let region = row.aws_region.ok_or_else(|| {
                    CommonError::Unknown(anyhow::anyhow!("AWS KMS key missing region"))
                })?;
                LogicEnvelopeEncryptionKey::AwsKms { arn, region }
            }
            crate::repository::EnvelopeEncryptionKeyType::Local => {
                let location = row.local_location.ok_or_else(|| {
                    CommonError::Unknown(anyhow::anyhow!("Local key missing location"))
                })?;
                LogicEnvelopeEncryptionKey::Local { location }
            }
        };

        Ok(LogicDataEncryptionKey {
            id: row.id,
            envelope_encryption_key_id,
            encrypted_data_encryption_key: row.encryption_key,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl TryFrom<Row_get_data_encryption_key_by_id_with_envelope> for LogicDataEncryptionKey {
    type Error = CommonError;

    fn try_from(row: Row_get_data_encryption_key_by_id_with_envelope) -> Result<Self, Self::Error> {
        // Convert envelope encryption key from row fields
        let envelope_encryption_key_id = match row.key_type {
            crate::repository::EnvelopeEncryptionKeyType::AwsKms => {
                let arn = row.aws_arn.ok_or_else(|| {
                    CommonError::Unknown(anyhow::anyhow!("AWS KMS key missing ARN"))
                })?;
                let region = row.aws_region.ok_or_else(|| {
                    CommonError::Unknown(anyhow::anyhow!("AWS KMS key missing region"))
                })?;
                LogicEnvelopeEncryptionKey::AwsKms { arn, region }
            }
            crate::repository::EnvelopeEncryptionKeyType::Local => {
                let location = row.local_location.ok_or_else(|| {
                    CommonError::Unknown(anyhow::anyhow!("Local key missing location"))
                })?;
                LogicEnvelopeEncryptionKey::Local { location }
            }
        };

        Ok(LogicDataEncryptionKey {
            id: row.id,
            envelope_encryption_key_id,
            encrypted_data_encryption_key: row.encryption_key,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

