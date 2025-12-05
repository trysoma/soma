#![allow(non_camel_case_types)]
#![allow(dead_code)]
mod raw_impl;

#[allow(clippy::all)]
pub mod generated {
    include!("raw.generated.rs");
}

pub use generated::*;

use crate::logic::dek::{DataEncryptionKey, DataEncryptionKeyListItem};
use crate::logic::envelope::EnvelopeEncryptionKey;
use crate::repository::{
    CreateDataEncryptionKey, CreateEnvelopeEncryptionKey, EncryptionKeyRepositoryLike,
};
use anyhow::Context;
use base64::Engine;
use shared::primitives::WrappedChronoDateTime;
use shared::{
    error::CommonError,
    primitives::{
        PaginatedResponse, PaginationRequest, SqlMigrationLoader, decode_pagination_token,
    },
};
use shared_macros::load_atlas_sql_migrations;
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct Repository {
    conn: shared::libsql::Connection,
}

impl Repository {
    pub fn new(conn: shared::libsql::Connection) -> Self {
        Self { conn }
    }

    /// Get all data encryption keys with their envelope encryption keys (using JOIN query)
    pub async fn get_all_data_encryption_keys_with_envelope_keys(
        &self,
    ) -> Result<Vec<DataEncryptionKey>, CommonError> {
        use crate::repository::sqlite::generated::get_all_data_encryption_keys_with_envelope_keys;

        let rows = get_all_data_encryption_keys_with_envelope_keys(&self.conn)
            .await
            .context("Failed to get all data encryption keys with envelope keys")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        let mut deks = Vec::new();
        for row in rows {
            let dek = row.try_into().map_err(|e: CommonError| e)?;
            deks.push(dek);
        }

        Ok(deks)
    }
}

impl EncryptionKeyRepositoryLike for Repository {
    async fn create_envelope_encryption_key(
        &self,
        params: &CreateEnvelopeEncryptionKey,
    ) -> Result<(), CommonError> {
        use crate::repository::sqlite::generated::create_envelope_encryption_key;
        use crate::repository::sqlite::generated::create_envelope_encryption_key_params;

        let sqlc_params = create_envelope_encryption_key_params {
            id: &params.id,
            key_type: &params.key_type,
            local_file_name: &params.local_file_name,
            aws_arn: &params.aws_arn,
            aws_region: &params.aws_region,
            created_at: &params.created_at,
            updated_at: &params.updated_at,
        };

        create_envelope_encryption_key(&self.conn, sqlc_params)
            .await
            .context("Failed to create envelope encryption key")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn get_envelope_encryption_key_by_id(
        &self,
        id: &str,
    ) -> Result<Option<EnvelopeEncryptionKey>, CommonError> {
        use crate::repository::sqlite::generated::get_envelope_encryption_key_by_id;
        use crate::repository::sqlite::generated::get_envelope_encryption_key_by_id_params;

        let sqlc_params = get_envelope_encryption_key_by_id_params {
            id: &id.to_string(),
        };

        let result = get_envelope_encryption_key_by_id(&self.conn, sqlc_params)
            .await
            .context("Failed to get envelope encryption key by id")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        result.map(|row| row.try_into()).transpose()
    }

    async fn list_envelope_encryption_keys(
        &self,
    ) -> Result<Vec<EnvelopeEncryptionKey>, CommonError> {
        use crate::repository::sqlite::generated::get_envelope_encryption_keys;

        let rows = get_envelope_encryption_keys(&self.conn)
            .await
            .context("Failed to list envelope encryption keys")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        rows.into_iter().map(|row| row.try_into()).collect()
    }

    async fn list_envelope_encryption_keys_paginated(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<EnvelopeEncryptionKey>, CommonError> {
        // Decode the base64 token to get the datetime cursor
        let cursor_datetime = if let Some(token) = &pagination.next_page_token {
            let decoded_parts =
                decode_pagination_token(token).map_err(|e| CommonError::Repository {
                    msg: format!("Invalid pagination token: {e}"),
                    source: Some(e.into()),
                })?;
            if decoded_parts.is_empty() {
                None
            } else {
                Some(
                    shared::primitives::WrappedChronoDateTime::try_from(decoded_parts[0].as_str())
                        .map_err(|e| CommonError::Repository {
                            msg: format!("Invalid datetime in pagination token: {e}"),
                            source: Some(e.into()),
                        })?,
                )
            }
        } else {
            None
        };

        use crate::repository::sqlite::generated::get_envelope_encryption_keys_paginated;
        use crate::repository::sqlite::generated::get_envelope_encryption_keys_paginated_params;

        let sqlc_params = get_envelope_encryption_keys_paginated_params {
            cursor: &cursor_datetime,
            page_size: &pagination.page_size,
        };

        let rows = get_envelope_encryption_keys_paginated(&self.conn, sqlc_params)
            .await
            .context("Failed to get envelope encryption keys")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        // Extract created_at values for pagination token generation
        let created_at_values: Vec<WrappedChronoDateTime> =
            rows.iter().map(|row| row.created_at).collect();

        // Convert rows to items using TryFrom
        let items: Result<Vec<EnvelopeEncryptionKey>, CommonError> = rows
            .into_iter()
            .map(EnvelopeEncryptionKey::try_from)
            .collect();
        let items = items?;

        // Check if we got more items than requested (page_size + 1)
        let has_more = items.len() as i64 > pagination.page_size;

        // If we have more items than page_size, remove the extra item
        let mut items = items;
        if has_more {
            items.pop();
        }

        // Generate next_page_token from the last item's created_at
        let next_page_token = if has_more && !items.is_empty() {
            // The last item corresponds to created_at at the same index
            created_at_values.get(items.len() - 1).map(|created_at| {
                let key_parts = [created_at.get_inner().to_rfc3339()];
                let composite_key = key_parts.join("__");
                base64::engine::general_purpose::STANDARD.encode(composite_key.as_bytes())
            })
        } else {
            None
        };

        Ok(PaginatedResponse {
            items,
            next_page_token,
        })
    }

    async fn delete_envelope_encryption_key(&self, id: &str) -> Result<(), CommonError> {
        use crate::repository::sqlite::generated::delete_envelope_encryption_key;
        use crate::repository::sqlite::generated::delete_envelope_encryption_key_params;

        let sqlc_params = delete_envelope_encryption_key_params {
            id: &id.to_string(),
        };

        delete_envelope_encryption_key(&self.conn, sqlc_params)
            .await
            .context("Failed to delete envelope encryption key")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn create_data_encryption_key(
        &self,
        params: &CreateDataEncryptionKey,
    ) -> Result<(), CommonError> {
        use crate::repository::sqlite::generated::create_data_encryption_key;
        use crate::repository::sqlite::generated::create_data_encryption_key_params;

        let sqlc_params = create_data_encryption_key_params {
            id: &params.id,
            envelope_encryption_key_id: &params.envelope_encryption_key_id,
            encryption_key: &params.encryption_key,
            created_at: &params.created_at,
            updated_at: &params.updated_at,
        };

        create_data_encryption_key(&self.conn, sqlc_params)
            .await
            .context("Failed to create data encryption key")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn get_data_encryption_key_by_id(
        &self,
        id: &str,
    ) -> Result<Option<DataEncryptionKey>, CommonError> {
        use crate::repository::sqlite::generated::get_data_encryption_key_by_id_with_envelope;
        use crate::repository::sqlite::generated::get_data_encryption_key_by_id_with_envelope_params;

        let sqlc_params = get_data_encryption_key_by_id_with_envelope_params {
            id: &id.to_string(),
        };

        let result = get_data_encryption_key_by_id_with_envelope(&self.conn, sqlc_params)
            .await
            .context("Failed to get data encryption key by id with envelope")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        // Convert the row to DataEncryptionKey using TryFrom
        // The JOIN ensures both DEK and envelope key exist, or returns None
        result.map(DataEncryptionKey::try_from).transpose()
    }

    async fn delete_data_encryption_key(&self, id: &str) -> Result<(), CommonError> {
        use crate::repository::sqlite::generated::delete_data_encryption_key;
        use crate::repository::sqlite::generated::delete_data_encryption_key_params;

        let sqlc_params = delete_data_encryption_key_params {
            id: &id.to_string(),
        };

        delete_data_encryption_key(&self.conn, sqlc_params)
            .await
            .context("Failed to delete data encryption key")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn list_data_encryption_keys(
        &self,
        pagination: &PaginationRequest,
    ) -> Result<PaginatedResponse<DataEncryptionKeyListItem>, CommonError> {
        // Decode the base64 token to get the datetime cursor
        let cursor_datetime = if let Some(token) = &pagination.next_page_token {
            let decoded_parts =
                decode_pagination_token(token).map_err(|e| CommonError::Repository {
                    msg: format!("Invalid pagination token: {e}"),
                    source: Some(e.into()),
                })?;
            if decoded_parts.is_empty() {
                None
            } else {
                Some(
                    shared::primitives::WrappedChronoDateTime::try_from(decoded_parts[0].as_str())
                        .map_err(|e| CommonError::Repository {
                            msg: format!("Invalid datetime in pagination token: {e}"),
                            source: Some(e.into()),
                        })?,
                )
            }
        } else {
            None
        };

        use crate::repository::sqlite::generated::get_data_encryption_keys;
        use crate::repository::sqlite::generated::get_data_encryption_keys_params;

        let sqlc_params = get_data_encryption_keys_params {
            cursor: &cursor_datetime,
            page_size: &pagination.page_size,
        };

        let rows = get_data_encryption_keys(&self.conn, sqlc_params)
            .await
            .context("Failed to get data encryption keys")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        // Convert envelope_encryption_key_id string to EnvelopeEncryptionKey for each item
        let mut items = Vec::new();
        for row in rows {
            // Get the envelope encryption key to determine the type
            use crate::repository::sqlite::generated::get_envelope_encryption_key_by_id;
            use crate::repository::sqlite::generated::get_envelope_encryption_key_by_id_params;

            let sqlc_params = get_envelope_encryption_key_by_id_params {
                id: &row.envelope_encryption_key_id,
            };

            let envelope_key_result = get_envelope_encryption_key_by_id(&self.conn, sqlc_params)
                .await
                .context("Failed to get envelope encryption key by id")
                .map_err(|e| CommonError::Repository {
                    msg: e.to_string(),
                    source: Some(e),
                })?;

            let envelope_key_id = match envelope_key_result {
                Some(key_row) => {
                    // Convert Row to EnvelopeEncryptionKey using TryFrom
                    EnvelopeEncryptionKey::try_from(key_row).map_err(|e| {
                        CommonError::Repository {
                            msg: format!("Failed to convert envelope key: {e}"),
                            source: Some(e.into()),
                        }
                    })?
                }
                None => {
                    return Err(CommonError::Repository {
                        msg: format!(
                            "Envelope encryption key {} not found",
                            row.envelope_encryption_key_id
                        ),
                        source: None,
                    });
                }
            };

            items.push(DataEncryptionKeyListItem {
                id: row.id,
                envelope_encryption_key_id: envelope_key_id,
                created_at: row.created_at,
                updated_at: row.updated_at,
            });
        }

        Ok(PaginatedResponse::from_items_with_extra(
            items,
            pagination,
            |item| vec![item.created_at.get_inner().to_rfc3339()],
        ))
    }
}

// Implement the encryption crate's DataEncryptionKeyRepositoryLike trait
#[async_trait::async_trait]
impl crate::repository::DataEncryptionKeyRepositoryLike for Repository {
    async fn create_data_encryption_key(
        &self,
        data_encryption_key: &DataEncryptionKey,
    ) -> Result<(), CommonError> {
        <Self as EncryptionKeyRepositoryLike>::create_data_encryption_key(
            self,
            &CreateDataEncryptionKey::from(data_encryption_key.clone()),
        )
        .await
    }

    async fn get_data_encryption_key_by_id(
        &self,
        id: &str,
    ) -> Result<Option<DataEncryptionKey>, CommonError> {
        <Self as EncryptionKeyRepositoryLike>::get_data_encryption_key_by_id(self, id).await
    }

    async fn list_data_encryption_keys(
        &self,
        params: &PaginationRequest,
    ) -> Result<PaginatedResponse<DataEncryptionKeyListItem>, CommonError> {
        <Self as EncryptionKeyRepositoryLike>::list_data_encryption_keys(self, params).await
    }

    async fn delete_data_encryption_key(&self, id: &str) -> Result<(), CommonError> {
        <Self as EncryptionKeyRepositoryLike>::delete_data_encryption_key(self, id).await
    }

    async fn create_data_encryption_key_alias(
        &self,
        alias: &crate::repository::DataEncryptionKeyAlias,
    ) -> Result<(), CommonError> {
        use crate::repository::sqlite::generated::create_data_encryption_key_alias;
        use crate::repository::sqlite::generated::create_data_encryption_key_alias_params;

        let sqlc_params = create_data_encryption_key_alias_params {
            alias: &alias.alias,
            data_encryption_key_id: &alias.data_encryption_key_id,
            created_at: &alias.created_at,
        };

        create_data_encryption_key_alias(&self.conn, sqlc_params)
            .await
            .context("Failed to create data encryption key alias")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn get_data_encryption_key_alias_by_alias(
        &self,
        alias: &str,
    ) -> Result<Option<crate::repository::DataEncryptionKeyAlias>, CommonError> {
        use crate::repository::sqlite::generated::get_data_encryption_key_alias_by_alias;
        use crate::repository::sqlite::generated::get_data_encryption_key_alias_by_alias_params;

        let sqlc_params = get_data_encryption_key_alias_by_alias_params {
            alias: &alias.to_string(),
        };

        let result = get_data_encryption_key_alias_by_alias(&self.conn, sqlc_params)
            .await
            .context("Failed to get data encryption key alias by alias")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        Ok(result.map(|row| crate::repository::DataEncryptionKeyAlias {
            alias: row.alias,
            data_encryption_key_id: row.data_encryption_key_id,
            created_at: row.created_at,
        }))
    }

    async fn get_data_encryption_key_by_alias(
        &self,
        alias: &str,
    ) -> Result<Option<DataEncryptionKey>, CommonError> {
        use crate::repository::sqlite::generated::get_data_encryption_key_by_alias;
        use crate::repository::sqlite::generated::get_data_encryption_key_by_alias_params;
        use crate::repository::sqlite::generated::get_envelope_encryption_key_by_id;
        use crate::repository::sqlite::generated::get_envelope_encryption_key_by_id_params;

        let sqlc_params = get_data_encryption_key_by_alias_params {
            alias: &alias.to_string(),
        };

        let result = get_data_encryption_key_by_alias(&self.conn, sqlc_params)
            .await
            .context("Failed to get data encryption key by alias")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        match result {
            None => Ok(None),
            Some(row) => {
                // Get the envelope encryption key to convert to EnvelopeEncryptionKey
                let envelope_sqlc_params = get_envelope_encryption_key_by_id_params {
                    id: &row.envelope_encryption_key_id,
                };

                let envelope_key_result =
                    get_envelope_encryption_key_by_id(&self.conn, envelope_sqlc_params)
                        .await
                        .context("Failed to get envelope encryption key by id")
                        .map_err(|e| CommonError::Repository {
                            msg: e.to_string(),
                            source: Some(e),
                        })?;

                let envelope_key_id = match envelope_key_result {
                    Some(key_row) => EnvelopeEncryptionKey::try_from(key_row).map_err(|e| {
                        CommonError::Repository {
                            msg: format!("Failed to convert envelope key: {e}"),
                            source: Some(e.into()),
                        }
                    })?,
                    None => {
                        return Err(CommonError::Repository {
                            msg: format!(
                                "Envelope encryption key {} not found for DEK",
                                row.envelope_encryption_key_id
                            ),
                            source: None,
                        });
                    }
                };

                Ok(Some(DataEncryptionKey {
                    id: row.id,
                    envelope_encryption_key_id: envelope_key_id,
                    encrypted_data_encryption_key: row.encryption_key,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                }))
            }
        }
    }

    async fn delete_data_encryption_key_alias(&self, alias: &str) -> Result<(), CommonError> {
        use crate::repository::sqlite::generated::delete_data_encryption_key_alias;
        use crate::repository::sqlite::generated::delete_data_encryption_key_alias_params;

        let sqlc_params = delete_data_encryption_key_alias_params {
            alias: &alias.to_string(),
        };

        delete_data_encryption_key_alias(&self.conn, sqlc_params)
            .await
            .context("Failed to delete data encryption key alias")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;
        Ok(())
    }

    async fn list_aliases_for_dek(
        &self,
        dek_id: &str,
    ) -> Result<Vec<crate::repository::DataEncryptionKeyAlias>, CommonError> {
        use crate::repository::sqlite::generated::list_aliases_for_dek;
        use crate::repository::sqlite::generated::list_aliases_for_dek_params;

        let sqlc_params = list_aliases_for_dek_params {
            data_encryption_key_id: &dek_id.to_string(),
        };

        let rows = list_aliases_for_dek(&self.conn, sqlc_params)
            .await
            .context("Failed to list aliases for DEK")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        Ok(rows
            .into_iter()
            .map(|row| crate::repository::DataEncryptionKeyAlias {
                alias: row.alias,
                data_encryption_key_id: row.data_encryption_key_id,
                created_at: row.created_at,
            })
            .collect())
    }

    async fn update_data_encryption_key_alias(
        &self,
        alias: &str,
        new_dek_id: &str,
    ) -> Result<(), CommonError> {
        use crate::repository::sqlite::generated::update_data_encryption_key_alias;
        use crate::repository::sqlite::generated::update_data_encryption_key_alias_params;

        let sqlc_params = update_data_encryption_key_alias_params {
            data_encryption_key_id: &new_dek_id.to_string(),
            alias: &alias.to_string(),
        };

        update_data_encryption_key_alias(&self.conn, sqlc_params)
            .await
            .context("Failed to update data encryption key alias")
            .map_err(|e| CommonError::Repository {
                msg: e.to_string(),
                source: Some(e),
            })?;

        Ok(())
    }
}

impl SqlMigrationLoader for Repository {
    fn load_sql_migrations() -> BTreeMap<&'static str, BTreeMap<&'static str, &'static str>> {
        load_atlas_sql_migrations!("dbs/encryption/migrations")
    }
}

#[cfg(all(test, feature = "unit_test"))]
mod unit_test {
    use super::*;
    use crate::logic::dek::EncryptedDataEncryptionKey;
    use crate::logic::envelope::EnvelopeEncryptionKey;
    use crate::repository::{
        CreateDataEncryptionKey, CreateEnvelopeEncryptionKey, DataEncryptionKeyAlias,
    };
    use shared::primitives::{PaginationRequest, SqlMigrationLoader, WrappedChronoDateTime};
    use shared::test_utils::repository::setup_in_memory_database;

    async fn create_test_envelope_key_aws(repo: &Repository, id: &str, now: WrappedChronoDateTime) {
        let params = CreateEnvelopeEncryptionKey {
            id: id.to_string(),
            key_type: crate::repository::EnvelopeEncryptionKeyType::AwsKms,
            local_file_name: None,
            aws_arn: Some("arn:aws:kms:eu-west-2:123456789012:key/test-key".to_string()),
            aws_region: Some("eu-west-2".to_string()),
            created_at: now,
            updated_at: now,
        };
        repo.create_envelope_encryption_key(&params).await.unwrap();
    }

    async fn create_test_envelope_key_local(
        repo: &Repository,
        id: &str,
        now: WrappedChronoDateTime,
    ) {
        let params = CreateEnvelopeEncryptionKey {
            id: id.to_string(),
            key_type: crate::repository::EnvelopeEncryptionKeyType::Local,
            local_file_name: Some("/path/to/key".to_string()),
            aws_arn: None,
            aws_region: None,
            created_at: now,
            updated_at: now,
        };
        repo.create_envelope_encryption_key(&params).await.unwrap();
    }

    async fn create_test_dek(
        repo: &Repository,
        dek_id: &str,
        envelope_key_id: &str,
        now: WrappedChronoDateTime,
    ) {
        let params = CreateDataEncryptionKey {
            id: dek_id.to_string(),
            envelope_encryption_key_id: envelope_key_id.to_string(),
            encryption_key: EncryptedDataEncryptionKey("test_encrypted_key".to_string()),
            created_at: now,
            updated_at: now,
        };
        repo.create_data_encryption_key(&params).await.unwrap();
    }

    #[tokio::test]
    async fn test_create_and_get_envelope_encryption_key_aws() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let key_id = "test-aws-key-1";

        // Create AWS KMS envelope key
        create_test_envelope_key_aws(&repo, key_id, now).await;

        // Retrieve it
        let retrieved = repo
            .get_envelope_encryption_key_by_id(key_id)
            .await
            .unwrap()
            .unwrap();

        match retrieved {
            EnvelopeEncryptionKey::AwsKms(aws_kms) => {
                assert_eq!(
                    aws_kms.arn,
                    "arn:aws:kms:eu-west-2:123456789012:key/test-key"
                );
                assert_eq!(aws_kms.region, "eu-west-2");
            }
            _ => panic!("Expected AWS KMS key"),
        }
    }

    #[tokio::test]
    async fn test_create_and_get_envelope_encryption_key_local() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let key_id = "test-local-key-1";

        // Create local envelope key
        create_test_envelope_key_local(&repo, key_id, now).await;

        // Retrieve it
        let retrieved = repo
            .get_envelope_encryption_key_by_id(key_id)
            .await
            .unwrap()
            .unwrap();

        match retrieved {
            EnvelopeEncryptionKey::Local(local) => {
                assert_eq!(local.file_name, "/path/to/key");
            }
            _ => panic!("Expected local key"),
        }
    }

    #[tokio::test]
    async fn test_list_envelope_encryption_keys() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();

        // Create multiple envelope keys
        create_test_envelope_key_aws(&repo, "aws-key-1", now).await;
        create_test_envelope_key_local(&repo, "local-key-1", now).await;
        create_test_envelope_key_aws(&repo, "aws-key-2", now).await;

        // List all keys
        let keys = repo.list_envelope_encryption_keys().await.unwrap();

        assert_eq!(keys.len(), 3);

        // We should have 2 AWS keys and 1 local key
        let aws_count = keys
            .iter()
            .filter(|k| matches!(k, EnvelopeEncryptionKey::AwsKms(_)))
            .count();
        let local_count = keys
            .iter()
            .filter(|k| matches!(k, EnvelopeEncryptionKey::Local(_)))
            .count();
        assert_eq!(aws_count, 2);
        assert_eq!(local_count, 1);
    }

    #[tokio::test]
    async fn test_delete_envelope_encryption_key() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let key_id = "test-key-to-delete";

        // Create key
        create_test_envelope_key_aws(&repo, key_id, now).await;

        // Verify it exists
        assert!(
            repo.get_envelope_encryption_key_by_id(key_id)
                .await
                .unwrap()
                .is_some()
        );

        // Delete it
        repo.delete_envelope_encryption_key(key_id).await.unwrap();

        // Verify it's gone
        assert!(
            repo.get_envelope_encryption_key_by_id(key_id)
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_get_nonexistent_envelope_encryption_key() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let result = repo
            .get_envelope_encryption_key_by_id("nonexistent-key")
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_create_and_get_data_encryption_key() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let envelope_key_id = "test-envelope-key";
        let dek_id = "test-dek-1";

        // Create envelope key first
        create_test_envelope_key_aws(&repo, envelope_key_id, now).await;

        // Create DEK
        create_test_dek(&repo, dek_id, envelope_key_id, now).await;

        // Retrieve DEK
        let retrieved = repo
            .get_data_encryption_key_by_id(dek_id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(retrieved.id, dek_id);
        assert_eq!(
            retrieved.encrypted_data_encryption_key.0,
            "test_encrypted_key"
        );

        // Verify envelope key ID matches
        match retrieved.envelope_encryption_key_id {
            EnvelopeEncryptionKey::AwsKms(aws_kms) => {
                assert_eq!(
                    aws_kms.arn,
                    "arn:aws:kms:eu-west-2:123456789012:key/test-key"
                );
                assert_eq!(aws_kms.region, "eu-west-2");
            }
            _ => panic!("Expected AWS KMS key"),
        }
    }

    #[tokio::test]
    async fn test_create_data_encryption_key_with_local_envelope() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let envelope_key_id = "test-local-envelope-key";
        let dek_id = "test-dek-local";

        // Create local envelope key
        create_test_envelope_key_local(&repo, envelope_key_id, now).await;

        // Create DEK with local envelope key
        create_test_dek(&repo, dek_id, envelope_key_id, now).await;

        // Retrieve DEK
        let retrieved = repo
            .get_data_encryption_key_by_id(dek_id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(retrieved.id, dek_id);

        // Verify envelope key ID matches
        match retrieved.envelope_encryption_key_id {
            EnvelopeEncryptionKey::Local(local) => {
                assert_eq!(local.file_name, "/path/to/key");
            }
            _ => panic!("Expected local key"),
        }
    }

    #[tokio::test]
    async fn test_list_data_encryption_keys() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let envelope_key_id = "test-envelope-key";

        // Create envelope key
        create_test_envelope_key_aws(&repo, envelope_key_id, now).await;

        // Create multiple DEKs
        create_test_dek(&repo, "dek-1", envelope_key_id, now).await;
        create_test_dek(&repo, "dek-2", envelope_key_id, now).await;
        create_test_dek(&repo, "dek-3", envelope_key_id, now).await;

        // List DEKs
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let result = repo.list_data_encryption_keys(&pagination).await.unwrap();

        assert_eq!(result.items.len(), 3);
        let ids: Vec<String> = result.items.iter().map(|item| item.id.clone()).collect();
        assert!(ids.contains(&"dek-1".to_string()));
        assert!(ids.contains(&"dek-2".to_string()));
        assert!(ids.contains(&"dek-3".to_string()));
    }

    #[tokio::test]
    async fn test_list_data_encryption_keys_pagination() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let mut now = WrappedChronoDateTime::now();
        let envelope_key_id = "test-envelope-key";

        // Create envelope key
        create_test_envelope_key_aws(&repo, envelope_key_id, now).await;

        // Create multiple DEKs with slight time differences to ensure ordering
        for i in 1..=5 {
            // Add small delay to ensure different timestamps
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            now = WrappedChronoDateTime::now();
            create_test_dek(&repo, &format!("dek-{i}"), envelope_key_id, now).await;
        }

        // First page
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: None,
        };
        let result = repo.list_data_encryption_keys(&pagination).await.unwrap();

        assert_eq!(result.items.len(), 2, "First page should have 2 items");
        assert!(
            result.next_page_token.is_some(),
            "Should have next page token"
        );

        // Second page
        let pagination = PaginationRequest {
            page_size: 2,
            next_page_token: result.next_page_token.clone(),
        };
        let result = repo.list_data_encryption_keys(&pagination).await.unwrap();

        assert_eq!(result.items.len(), 2, "Second page should have 2 items");
        assert!(
            result.next_page_token.is_some(),
            "Should have next page token for third page"
        );
    }

    #[tokio::test]
    async fn test_delete_data_encryption_key() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let envelope_key_id = "test-envelope-key";
        let dek_id = "test-dek-to-delete";

        // Create envelope key
        create_test_envelope_key_aws(&repo, envelope_key_id, now).await;

        // Create DEK
        create_test_dek(&repo, dek_id, envelope_key_id, now).await;

        // Verify it exists
        assert!(
            repo.get_data_encryption_key_by_id(dek_id)
                .await
                .unwrap()
                .is_some()
        );

        // Delete it
        repo.delete_data_encryption_key(dek_id).await.unwrap();

        // Verify it's gone
        assert!(
            repo.get_data_encryption_key_by_id(dek_id)
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_get_nonexistent_data_encryption_key() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let result = repo
            .get_data_encryption_key_by_id("nonexistent-dek")
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_data_encryption_key_by_id_with_envelope_joins_correctly() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn.clone());

        let now = WrappedChronoDateTime::now();
        let envelope_key_id = "test-envelope-key";
        let dek_id = "test-dek-1";

        // Create envelope key first
        create_test_envelope_key_aws(&repo, envelope_key_id, now).await;

        // Create DEK
        create_test_dek(&repo, dek_id, envelope_key_id, now).await;

        // Retrieve DEK - should work with single query JOIN
        let retrieved = repo
            .get_data_encryption_key_by_id(dek_id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(retrieved.id, dek_id);
        assert_eq!(
            retrieved.encrypted_data_encryption_key.0,
            "test_encrypted_key"
        );

        // Verify envelope key ID matches (from JOIN)
        match retrieved.envelope_encryption_key_id {
            EnvelopeEncryptionKey::AwsKms(aws_kms) => {
                assert_eq!(
                    aws_kms.arn,
                    "arn:aws:kms:eu-west-2:123456789012:key/test-key"
                );
                assert_eq!(aws_kms.region, "eu-west-2");
            }
            _ => panic!("Expected AWS KMS key"),
        }

        // Test that non-existent DEK returns None
        let result = repo
            .get_data_encryption_key_by_id("nonexistent-dek")
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_create_data_encryption_key_with_nonexistent_envelope_key() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let dek_id = "test-dek";

        // Try to create DEK with nonexistent envelope key
        let params = CreateDataEncryptionKey {
            id: dek_id.to_string(),
            envelope_encryption_key_id: "nonexistent-envelope-key".to_string(),
            encryption_key: EncryptedDataEncryptionKey("test_encrypted_key".to_string()),
            created_at: now,
            updated_at: now,
        };

        // This should fail due to foreign key constraint or when retrieving
        let result = repo.create_data_encryption_key(&params).await;

        // The creation might succeed (if FK constraint is not enforced),
        // but retrieval should fail
        if result.is_ok() {
            let retrieve_result = repo.get_data_encryption_key_by_id(dek_id).await;
            assert!(retrieve_result.is_err());
        } else {
            // Creation failed as expected
            assert!(result.is_err());
        }
    }

    #[tokio::test]
    async fn test_multiple_deks_with_same_envelope_key() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let envelope_key_id = "shared-envelope-key";

        // Create envelope key
        create_test_envelope_key_aws(&repo, envelope_key_id, now).await;

        // Create multiple DEKs with same envelope key
        create_test_dek(&repo, "dek-1", envelope_key_id, now).await;
        create_test_dek(&repo, "dek-2", envelope_key_id, now).await;
        create_test_dek(&repo, "dek-3", envelope_key_id, now).await;

        // List all DEKs
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let result = repo.list_data_encryption_keys(&pagination).await.unwrap();

        assert_eq!(result.items.len(), 3);

        // Verify all DEKs reference the same envelope key
        for item in result.items {
            match item.envelope_encryption_key_id {
                EnvelopeEncryptionKey::AwsKms(aws_kms) => {
                    assert_eq!(
                        aws_kms.arn,
                        "arn:aws:kms:eu-west-2:123456789012:key/test-key"
                    );
                }
                _ => panic!("Expected AWS KMS key"),
            }
        }
    }

    #[tokio::test]
    async fn test_deks_with_different_envelope_keys() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let envelope_key_1 = "envelope-key-1";
        let envelope_key_2 = "envelope-key-2";

        // Create two different envelope keys
        create_test_envelope_key_aws(&repo, envelope_key_1, now).await;
        create_test_envelope_key_local(&repo, envelope_key_2, now).await;

        // Create DEKs with different envelope keys
        create_test_dek(&repo, "dek-1", envelope_key_1, now).await;
        create_test_dek(&repo, "dek-2", envelope_key_2, now).await;

        // List all DEKs
        let pagination = PaginationRequest {
            page_size: 10,
            next_page_token: None,
        };
        let result = repo.list_data_encryption_keys(&pagination).await.unwrap();

        assert_eq!(result.items.len(), 2);
    }

    #[tokio::test]
    async fn test_get_all_data_encryption_keys_with_envelope_keys() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let envelope_key_id_aws = "test-envelope-key-aws";
        let envelope_key_id_local = "test-envelope-key-local";
        let dek_id_1 = "test-dek-1";
        let dek_id_2 = "test-dek-2";

        // Create AWS envelope key
        create_test_envelope_key_aws(&repo, envelope_key_id_aws, now).await;

        // Create local envelope key
        create_test_envelope_key_local(&repo, envelope_key_id_local, now).await;

        // Create DEKs
        create_test_dek(&repo, dek_id_1, envelope_key_id_aws, now).await;
        create_test_dek(&repo, dek_id_2, envelope_key_id_local, now).await;

        // Get all DEKs with envelope keys using the new query
        let deks = repo
            .get_all_data_encryption_keys_with_envelope_keys()
            .await
            .unwrap();

        assert_eq!(deks.len(), 2);

        // Verify first DEK (AWS)
        let dek_1 = deks.iter().find(|d| d.id == dek_id_1).unwrap();
        assert_eq!(dek_1.id, dek_id_1);
        match &dek_1.envelope_encryption_key_id {
            EnvelopeEncryptionKey::AwsKms(aws_kms) => {
                assert_eq!(
                    aws_kms.arn,
                    "arn:aws:kms:eu-west-2:123456789012:key/test-key"
                );
                assert_eq!(aws_kms.region, "eu-west-2");
            }
            _ => panic!("Expected AWS KMS key"),
        }

        // Verify second DEK (Local)
        let dek_2 = deks.iter().find(|d| d.id == dek_id_2).unwrap();
        assert_eq!(dek_2.id, dek_id_2);
        match &dek_2.envelope_encryption_key_id {
            EnvelopeEncryptionKey::Local(local) => {
                assert_eq!(local.file_name, "/path/to/key");
            }
            _ => panic!("Expected local key"),
        }
    }

    // Alias system tests
    #[tokio::test]
    async fn test_create_and_get_data_encryption_key_alias() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let envelope_key_id = "test-envelope-key";
        let dek_id = "test-dek";

        // Create envelope key and DEK
        create_test_envelope_key_aws(&repo, envelope_key_id, now).await;
        create_test_dek(&repo, dek_id, envelope_key_id, now).await;

        // Create alias
        let alias = DataEncryptionKeyAlias {
            alias: "my-app-key".to_string(),
            data_encryption_key_id: dek_id.to_string(),
            created_at: now,
        };
        crate::repository::DataEncryptionKeyRepositoryLike::create_data_encryption_key_alias(
            &repo, &alias,
        )
        .await
        .unwrap();

        // Retrieve alias
        let retrieved = crate::repository::DataEncryptionKeyRepositoryLike::get_data_encryption_key_alias_by_alias(
            &repo,
            "my-app-key",
        )
        .await
        .unwrap()
        .unwrap();

        assert_eq!(retrieved.alias, "my-app-key");
        assert_eq!(retrieved.data_encryption_key_id, dek_id);
    }

    #[tokio::test]
    async fn test_get_data_encryption_key_by_alias() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let envelope_key_id = "test-envelope-key";
        let dek_id = "test-dek";

        // Create envelope key and DEK
        create_test_envelope_key_aws(&repo, envelope_key_id, now).await;
        create_test_dek(&repo, dek_id, envelope_key_id, now).await;

        // Create alias
        let alias = DataEncryptionKeyAlias {
            alias: "my-app-key".to_string(),
            data_encryption_key_id: dek_id.to_string(),
            created_at: now,
        };
        crate::repository::DataEncryptionKeyRepositoryLike::create_data_encryption_key_alias(
            &repo, &alias,
        )
        .await
        .unwrap();

        // Get DEK by alias
        let dek =
            crate::repository::DataEncryptionKeyRepositoryLike::get_data_encryption_key_by_alias(
                &repo,
                "my-app-key",
            )
            .await
            .unwrap()
            .unwrap();

        assert_eq!(dek.id, dek_id);
        assert_eq!(dek.encrypted_data_encryption_key.0, "test_encrypted_key");

        // Verify envelope key ID matches
        match dek.envelope_encryption_key_id {
            EnvelopeEncryptionKey::AwsKms(aws_kms) => {
                assert_eq!(
                    aws_kms.arn,
                    "arn:aws:kms:eu-west-2:123456789012:key/test-key"
                );
                assert_eq!(aws_kms.region, "eu-west-2");
            }
            _ => panic!("Expected AWS KMS key"),
        }
    }

    #[tokio::test]
    async fn test_list_aliases_for_dek() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let envelope_key_id = "test-envelope-key";
        let dek_id = "test-dek";

        // Create envelope key and DEK
        create_test_envelope_key_aws(&repo, envelope_key_id, now).await;
        create_test_dek(&repo, dek_id, envelope_key_id, now).await;

        // Create multiple aliases for the same DEK
        let alias1 = DataEncryptionKeyAlias {
            alias: "alias-1".to_string(),
            data_encryption_key_id: dek_id.to_string(),
            created_at: now,
        };
        let alias2 = DataEncryptionKeyAlias {
            alias: "alias-2".to_string(),
            data_encryption_key_id: dek_id.to_string(),
            created_at: now,
        };
        let alias3 = DataEncryptionKeyAlias {
            alias: "alias-3".to_string(),
            data_encryption_key_id: dek_id.to_string(),
            created_at: now,
        };

        crate::repository::DataEncryptionKeyRepositoryLike::create_data_encryption_key_alias(
            &repo, &alias1,
        )
        .await
        .unwrap();
        crate::repository::DataEncryptionKeyRepositoryLike::create_data_encryption_key_alias(
            &repo, &alias2,
        )
        .await
        .unwrap();
        crate::repository::DataEncryptionKeyRepositoryLike::create_data_encryption_key_alias(
            &repo, &alias3,
        )
        .await
        .unwrap();

        // List all aliases for the DEK
        let aliases =
            crate::repository::DataEncryptionKeyRepositoryLike::list_aliases_for_dek(&repo, dek_id)
                .await
                .unwrap();

        assert_eq!(aliases.len(), 3);
        let alias_names: Vec<String> = aliases.iter().map(|a| a.alias.clone()).collect();
        assert!(alias_names.contains(&"alias-1".to_string()));
        assert!(alias_names.contains(&"alias-2".to_string()));
        assert!(alias_names.contains(&"alias-3".to_string()));
    }

    #[tokio::test]
    async fn test_delete_data_encryption_key_alias() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let envelope_key_id = "test-envelope-key";
        let dek_id = "test-dek";

        // Create envelope key and DEK
        create_test_envelope_key_aws(&repo, envelope_key_id, now).await;
        create_test_dek(&repo, dek_id, envelope_key_id, now).await;

        // Create alias
        let alias = DataEncryptionKeyAlias {
            alias: "my-app-key".to_string(),
            data_encryption_key_id: dek_id.to_string(),
            created_at: now,
        };
        crate::repository::DataEncryptionKeyRepositoryLike::create_data_encryption_key_alias(
            &repo, &alias,
        )
        .await
        .unwrap();

        // Verify it exists
        assert!(crate::repository::DataEncryptionKeyRepositoryLike::get_data_encryption_key_alias_by_alias(
            &repo,
            "my-app-key"
        )
        .await
        .unwrap()
        .is_some());

        // Delete it
        crate::repository::DataEncryptionKeyRepositoryLike::delete_data_encryption_key_alias(
            &repo,
            "my-app-key",
        )
        .await
        .unwrap();

        // Verify it's gone
        assert!(crate::repository::DataEncryptionKeyRepositoryLike::get_data_encryption_key_alias_by_alias(
            &repo,
            "my-app-key"
        )
        .await
        .unwrap()
        .is_none());
    }

    #[tokio::test]
    async fn test_cascade_delete_aliases_when_dek_deleted() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let envelope_key_id = "test-envelope-key";
        let dek_id = "test-dek";

        // Create envelope key and DEK
        create_test_envelope_key_aws(&repo, envelope_key_id, now).await;
        create_test_dek(&repo, dek_id, envelope_key_id, now).await;

        // Create multiple aliases
        let alias1 = DataEncryptionKeyAlias {
            alias: "alias-1".to_string(),
            data_encryption_key_id: dek_id.to_string(),
            created_at: now,
        };
        let alias2 = DataEncryptionKeyAlias {
            alias: "alias-2".to_string(),
            data_encryption_key_id: dek_id.to_string(),
            created_at: now,
        };

        crate::repository::DataEncryptionKeyRepositoryLike::create_data_encryption_key_alias(
            &repo, &alias1,
        )
        .await
        .unwrap();
        crate::repository::DataEncryptionKeyRepositoryLike::create_data_encryption_key_alias(
            &repo, &alias2,
        )
        .await
        .unwrap();

        // Verify aliases exist
        assert!(crate::repository::DataEncryptionKeyRepositoryLike::get_data_encryption_key_alias_by_alias(
            &repo, "alias-1"
        )
        .await
        .unwrap()
        .is_some());
        assert!(crate::repository::DataEncryptionKeyRepositoryLike::get_data_encryption_key_alias_by_alias(
            &repo, "alias-2"
        )
        .await
        .unwrap()
        .is_some());

        // Delete the DEK
        crate::repository::DataEncryptionKeyRepositoryLike::delete_data_encryption_key(
            &repo, dek_id,
        )
        .await
        .unwrap();

        // Verify aliases are automatically deleted due to CASCADE
        assert!(crate::repository::DataEncryptionKeyRepositoryLike::get_data_encryption_key_alias_by_alias(
            &repo, "alias-1"
        )
        .await
        .unwrap()
        .is_none());
        assert!(crate::repository::DataEncryptionKeyRepositoryLike::get_data_encryption_key_alias_by_alias(
            &repo, "alias-2"
        )
        .await
        .unwrap()
        .is_none());
    }

    #[tokio::test]
    async fn test_get_nonexistent_alias() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let result = crate::repository::DataEncryptionKeyRepositoryLike::get_data_encryption_key_alias_by_alias(
            &repo,
            "nonexistent-alias",
        )
        .await
        .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_dek_by_nonexistent_alias() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let result =
            crate::repository::DataEncryptionKeyRepositoryLike::get_data_encryption_key_by_alias(
                &repo,
                "nonexistent-alias",
            )
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_create_duplicate_alias_fails() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let envelope_key_id = "test-envelope-key";
        let dek_id_1 = "test-dek-1";
        let dek_id_2 = "test-dek-2";

        // Create envelope key and two DEKs
        create_test_envelope_key_aws(&repo, envelope_key_id, now).await;
        create_test_dek(&repo, dek_id_1, envelope_key_id, now).await;
        create_test_dek(&repo, dek_id_2, envelope_key_id, now).await;

        // Create alias for first DEK
        let alias1 = DataEncryptionKeyAlias {
            alias: "my-alias".to_string(),
            data_encryption_key_id: dek_id_1.to_string(),
            created_at: now,
        };
        crate::repository::DataEncryptionKeyRepositoryLike::create_data_encryption_key_alias(
            &repo, &alias1,
        )
        .await
        .unwrap();

        // Try to create the same alias for second DEK
        let alias2 = DataEncryptionKeyAlias {
            alias: "my-alias".to_string(),
            data_encryption_key_id: dek_id_2.to_string(),
            created_at: now,
        };
        let result =
            crate::repository::DataEncryptionKeyRepositoryLike::create_data_encryption_key_alias(
                &repo, &alias2,
            )
            .await;

        // Should fail due to unique constraint on alias
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_multiple_aliases_for_different_deks() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let envelope_key_id = "test-envelope-key";
        let dek_id_1 = "test-dek-1";
        let dek_id_2 = "test-dek-2";

        // Create envelope key and two DEKs
        create_test_envelope_key_aws(&repo, envelope_key_id, now).await;
        create_test_dek(&repo, dek_id_1, envelope_key_id, now).await;
        create_test_dek(&repo, dek_id_2, envelope_key_id, now).await;

        // Create aliases for both DEKs
        let alias1 = DataEncryptionKeyAlias {
            alias: "dek-1-alias".to_string(),
            data_encryption_key_id: dek_id_1.to_string(),
            created_at: now,
        };
        let alias2 = DataEncryptionKeyAlias {
            alias: "dek-2-alias".to_string(),
            data_encryption_key_id: dek_id_2.to_string(),
            created_at: now,
        };

        crate::repository::DataEncryptionKeyRepositoryLike::create_data_encryption_key_alias(
            &repo, &alias1,
        )
        .await
        .unwrap();
        crate::repository::DataEncryptionKeyRepositoryLike::create_data_encryption_key_alias(
            &repo, &alias2,
        )
        .await
        .unwrap();

        // Get DEKs by their aliases
        let dek1 =
            crate::repository::DataEncryptionKeyRepositoryLike::get_data_encryption_key_by_alias(
                &repo,
                "dek-1-alias",
            )
            .await
            .unwrap()
            .unwrap();
        let dek2 =
            crate::repository::DataEncryptionKeyRepositoryLike::get_data_encryption_key_by_alias(
                &repo,
                "dek-2-alias",
            )
            .await
            .unwrap()
            .unwrap();

        assert_eq!(dek1.id, dek_id_1);
        assert_eq!(dek2.id, dek_id_2);
    }

    #[tokio::test]
    async fn test_list_aliases_for_dek_with_no_aliases() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let envelope_key_id = "test-envelope-key";
        let dek_id = "test-dek";

        // Create envelope key and DEK
        create_test_envelope_key_aws(&repo, envelope_key_id, now).await;
        create_test_dek(&repo, dek_id, envelope_key_id, now).await;

        // List aliases (should be empty)
        let aliases =
            crate::repository::DataEncryptionKeyRepositoryLike::list_aliases_for_dek(&repo, dek_id)
                .await
                .unwrap();
        assert_eq!(aliases.len(), 0);
    }

    #[tokio::test]
    async fn test_update_data_encryption_key_alias() {
        let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
            .await
            .unwrap();
        let repo = Repository::new(conn);

        let now = WrappedChronoDateTime::now();
        let envelope_key_id = "test-envelope-key";
        let dek_id_1 = "test-dek-1";
        let dek_id_2 = "test-dek-2";

        // Create envelope key and two DEKs
        create_test_envelope_key_aws(&repo, envelope_key_id, now).await;
        create_test_dek(&repo, dek_id_1, envelope_key_id, now).await;
        create_test_dek(&repo, dek_id_2, envelope_key_id, now).await;

        // Create alias for first DEK
        let alias = DataEncryptionKeyAlias {
            alias: "my-alias".to_string(),
            data_encryption_key_id: dek_id_1.to_string(),
            created_at: now,
        };
        crate::repository::DataEncryptionKeyRepositoryLike::create_data_encryption_key_alias(
            &repo, &alias,
        )
        .await
        .unwrap();

        // Verify it points to dek_id_1
        let retrieved = crate::repository::DataEncryptionKeyRepositoryLike::get_data_encryption_key_alias_by_alias(
            &repo,
            "my-alias",
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(retrieved.data_encryption_key_id, dek_id_1);

        // Update alias to point to dek_id_2
        crate::repository::DataEncryptionKeyRepositoryLike::update_data_encryption_key_alias(
            &repo, "my-alias", dek_id_2,
        )
        .await
        .unwrap();

        // Verify it now points to dek_id_2
        let updated = crate::repository::DataEncryptionKeyRepositoryLike::get_data_encryption_key_alias_by_alias(
            &repo,
            "my-alias",
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(updated.data_encryption_key_id, dek_id_2);
        assert_eq!(updated.alias, "my-alias");
    }
}
