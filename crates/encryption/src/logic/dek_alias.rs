// Data encryption key alias management logic
// This module provides high-level operations for DEK alias management with event publishing

use serde::{Deserialize, Serialize};
use shared::{error::CommonError, primitives::WrappedChronoDateTime};
use utoipa::ToSchema;

use super::crypto_services::CryptoCache;
use super::{DataEncryptionKey, EncryptionKeyEvent, EncryptionKeyEventSender};
use crate::repository::{DataEncryptionKeyAlias, DataEncryptionKeyRepositoryLike};

// Parameter types following the pattern from dek.rs
pub struct WithDekId<T> {
    pub dek_id: String,
    pub inner: T,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct CreateAliasInnerParams {
    pub alias: String,
}

pub type CreateAliasParams = WithDekId<CreateAliasInnerParams>;
pub type CreateAliasResponse = DataEncryptionKeyAlias;
pub type DeleteAliasParams = String;
pub type DeleteAliasResponse = ();

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct UpdateAliasParams {
    pub new_dek_id: String,
}

pub type UpdateAliasResponse = DataEncryptionKeyAlias;

/// Create a new alias for a data encryption key
pub async fn create_alias<R>(
    on_change_tx: &EncryptionKeyEventSender,
    repo: &R,
    cache: &CryptoCache,
    params: CreateAliasParams,
) -> Result<CreateAliasResponse, CommonError>
where
    R: DataEncryptionKeyRepositoryLike,
{
    // Fetch the full DEK to include in the event
    let dek = repo
        .get_data_encryption_key_by_id(&params.dek_id)
        .await?
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!(
                "Data encryption key not found: {}",
                params.dek_id
            ))
        })?;

    let now = WrappedChronoDateTime::now();

    let alias_name = params.inner.alias.clone();
    let dek_id = params.dek_id.clone();

    let alias = DataEncryptionKeyAlias {
        alias: params.inner.alias,
        data_encryption_key_id: params.dek_id,
        created_at: now,
    };

    repo.create_data_encryption_key_alias(&alias).await?;

    // Invalidate cache entries for both the DEK ID and the new alias
    cache.invalidate_cache(&dek_id);
    cache.invalidate_cache(&alias_name);

    // Publish event with full DEK data
    let _ = on_change_tx.send(EncryptionKeyEvent::DataEncryptionKeyAliasAdded {
        alias: alias_name,
        dek,
    });

    Ok(alias)
}

/// Delete an alias by its name
pub async fn delete_alias<R>(
    on_change_tx: &EncryptionKeyEventSender,
    repo: &R,
    cache: &CryptoCache,
    alias_name: DeleteAliasParams,
) -> Result<DeleteAliasResponse, CommonError>
where
    R: DataEncryptionKeyRepositoryLike,
{
    // Verify the alias exists and get the DEK ID it points to
    let alias_record = repo
        .get_data_encryption_key_alias_by_alias(&alias_name)
        .await?
        .ok_or_else(|| CommonError::Unknown(anyhow::anyhow!("Alias not found: {alias_name}")))?;

    let dek_id = alias_record.data_encryption_key_id.clone();

    repo.delete_data_encryption_key_alias(&alias_name).await?;

    // Invalidate cache entries for both the alias and the DEK ID
    cache.invalidate_cache(&alias_name);
    cache.invalidate_cache(&dek_id);

    // Publish event to trigger cache refresh
    let _ =
        on_change_tx.send(EncryptionKeyEvent::DataEncryptionKeyAliasRemoved { alias: alias_name });

    Ok(())
}

/// Update an alias to point to a different DEK
pub async fn update_alias<R>(
    on_change_tx: &EncryptionKeyEventSender,
    repo: &R,
    cache: &CryptoCache,
    alias_name: String,
    params: UpdateAliasParams,
) -> Result<UpdateAliasResponse, CommonError>
where
    R: DataEncryptionKeyRepositoryLike,
{
    // Verify the alias exists and get the old DEK ID
    let existing_alias = repo
        .get_data_encryption_key_alias_by_alias(&alias_name)
        .await?
        .ok_or_else(|| CommonError::Unknown(anyhow::anyhow!("Alias not found: {alias_name}")))?;

    let old_dek_id = existing_alias.data_encryption_key_id.clone();

    // Fetch the full new DEK to include in the event
    let dek = repo
        .get_data_encryption_key_by_id(&params.new_dek_id)
        .await?
        .ok_or_else(|| {
            CommonError::Unknown(anyhow::anyhow!(
                "Data encryption key not found: {}",
                params.new_dek_id
            ))
        })?;

    let new_dek_id = params.new_dek_id.clone();

    // Update the alias
    repo.update_data_encryption_key_alias(&alias_name, &params.new_dek_id)
        .await?;

    // Invalidate cache entries for the alias and both old and new DEK IDs
    cache.invalidate_cache(&alias_name);
    cache.invalidate_cache(&old_dek_id);
    cache.invalidate_cache(&new_dek_id);

    // Publish event with full DEK data
    let _ = on_change_tx.send(EncryptionKeyEvent::DataEncryptionKeyAliasUpdated {
        alias: alias_name.clone(),
        dek,
    });

    let updated_alias = DataEncryptionKeyAlias {
        alias: alias_name,
        data_encryption_key_id: params.new_dek_id,
        created_at: existing_alias.created_at,
    };

    Ok(updated_alias)
}

/// Get a DEK by alias or ID
/// First tries to find a DEK by the provided string as an alias,
/// if not found, tries to look it up directly as a DEK ID
pub async fn get_by_alias_or_id<R>(
    repo: &R,
    alias_or_id: &str,
) -> Result<DataEncryptionKey, CommonError>
where
    R: DataEncryptionKeyRepositoryLike + ?Sized,
{
    // First, try to find by alias
    if let Some(dek) = repo.get_data_encryption_key_by_alias(alias_or_id).await? {
        return Ok(dek);
    }

    // If not found, try to find by direct ID
    if let Some(dek) = repo.get_data_encryption_key_by_id(alias_or_id).await? {
        return Ok(dek);
    }

    // Neither alias nor ID found
    Err(CommonError::NotFound {
        msg: "Data encryption key not found with alias or ID".to_string(),
        lookup_id: alias_or_id.to_string(),
        source: None,
    })
}

/// List all aliases for a specific DEK
pub async fn list_aliases_for_dek<R>(
    repo: &R,
    dek_id: &str,
) -> Result<Vec<DataEncryptionKeyAlias>, CommonError>
where
    R: DataEncryptionKeyRepositoryLike,
{
    repo.list_aliases_for_dek(dek_id).await
}
