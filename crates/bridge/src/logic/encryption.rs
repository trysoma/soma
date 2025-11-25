// // Re-export encryption crate types
// pub use encryption::*;

// use serde::{Deserialize, Serialize};
// use shared::{error::CommonError, primitives::WrappedJsonValue};
// use utoipa::ToSchema;

// use crate::logic::{
//     OnConfigChangeEvt, OnConfigChangeTx,
//     controller::{
//         WithCredentialControllerTypeId, WithProviderControllerTypeId, get_credential_controller,
//         get_provider_controller,
//     },
// };
// // Bridge-specific data encryption key management functions

// /// Bridge-specific version of CreateDataEncryptionKeyParams that includes envelope encryption key identifier
// #[derive(Serialize, Deserialize, Clone, ToSchema)]
// pub struct CreateDataEncryptionKeyParamsBridge {
//     /// Optional ID for the data encryption key (auto-generated if not provided)
//     pub id: Option<String>,
//     /// Optional pre-encrypted data envelope key (will be generated if not provided)
//     pub encrypted_data_envelope_key: Option<EncryptedDataEncryptionKey>,
//     /// Optional envelope encryption key identifier (ARN for AWS KMS, location for local)
//     /// If not provided, uses the default key from the bridge configuration
//     pub envelope_encryption_key_identifier: Option<String>,
//     /// Optional AWS region (only used when envelope_encryption_key_identifier is an AWS KMS ARN)
//     /// If not provided, region will be extracted from the ARN
//     pub aws_region: Option<String>,
// }

// pub async fn create_data_encryption_key<R>(
//     key_encryption_key: &EnvelopeEncryptionKeyContents,
//     on_config_change_tx: &OnConfigChangeTx,
//     repo: &R,
//     params: CreateDataEncryptionKeyParams,
//     publish_on_change_evt: bool,
// ) -> Result<CreateDataEncryptionKeyResponse, CommonError>
// where
//     R: crate::repository::ProviderRepositoryLike + DataEncryptionKeyRepositoryLike,
// {
//     let dek = encryption::create_data_encryption_key(key_encryption_key, repo, params).await?;

//     if publish_on_change_evt {
//         on_config_change_tx
//             .send(OnConfigChangeEvt::DataEncryptionKeyAdded(dek.clone()))
//             .map_err(|e| {
//                 CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
//             })?;
//     }

//     Ok(dek)
// }

// pub async fn delete_data_encryption_key<R>(
//     on_config_change_tx: &OnConfigChangeTx,
//     repo: &R,
//     id: DeleteDataEncryptionKeyParams,
//     publish_on_change_evt: bool,
// ) -> Result<DeleteDataEncryptionKeyResponse, CommonError>
// where
//     R: crate::repository::ProviderRepositoryLike + DataEncryptionKeyRepositoryLike,
// {
//     encryption::delete_data_encryption_key(repo, id.clone()).await?;

//     if publish_on_change_evt {
//         on_config_change_tx
//             .send(OnConfigChangeEvt::DataEncryptionKeyRemoved(id))
//             .map_err(|e| {
//                 CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
//             })?;
//     }

//     Ok(())
// }

// // Bridge-specific credential encryption functions

// #[derive(Serialize, Deserialize, Clone, ToSchema)]
// pub struct EncryptCredentialConfigurationParamsInner {
//     pub value: WrappedJsonValue,
//     pub data_encryption_key_id: String,
// }

// pub type EncryptedCredentialConfigurationResponse = WrappedJsonValue;

// pub type EncryptConfigurationParams = WithProviderControllerTypeId<
//     WithCredentialControllerTypeId<EncryptCredentialConfigurationParamsInner>,
// >;

// pub async fn encrypt_resource_server_configuration<R>(
//     envelope_encryption_key_contents: &EnvelopeEncryptionKeyContents,
//     repo: &R,
//     params: EncryptConfigurationParams,
// ) -> Result<EncryptedCredentialConfigurationResponse, CommonError>
// where
//     R: crate::repository::ProviderRepositoryLike + DataEncryptionKeyRepositoryLike,
// {
//     let crypto_service = encryption::get_crypto_service(
//         envelope_encryption_key_contents,
//         repo,
//         &params.inner.inner.data_encryption_key_id,
//     )
//     .await?;
//     let encryption_service = encryption::get_encryption_service(&crypto_service)?;
//     let provider_controller = get_provider_controller(&params.provider_controller_type_id)?;
//     let credential_controller = get_credential_controller(
//         &provider_controller,
//         &params.inner.credential_controller_type_id,
//     )?;
//     let resource_server_configuration = params.inner.inner.value;

//     let encrypted_resource_server_configuration = credential_controller
//         .encrypt_resource_server_configuration(&encryption_service, resource_server_configuration)
//         .await?;

//     Ok(encrypted_resource_server_configuration.value())
// }

// pub async fn encrypt_user_credential_configuration<R>(
//     envelope_encryption_key_contents: &EnvelopeEncryptionKeyContents,
//     repo: &R,
//     params: EncryptConfigurationParams,
// ) -> Result<EncryptedCredentialConfigurationResponse, CommonError>
// where
//     R: crate::repository::ProviderRepositoryLike + DataEncryptionKeyRepositoryLike,
// {
//     let crypto_service = encryption::get_crypto_service(
//         envelope_encryption_key_contents,
//         repo,
//         &params.inner.inner.data_encryption_key_id,
//     )
//     .await?;
//     let encryption_service = encryption::get_encryption_service(&crypto_service)?;
//     let provider_controller = get_provider_controller(&params.provider_controller_type_id)?;
//     let credential_controller = get_credential_controller(
//         &provider_controller,
//         &params.inner.credential_controller_type_id,
//     )?;
//     let user_credential_configuration = params.inner.inner.value;

//     let encrypted_user_credential_configuration = credential_controller
//         .encrypt_user_credential_configuration(&encryption_service, user_credential_configuration)
//         .await?;

//     Ok(encrypted_user_credential_configuration.value())
// }

// // Migration types and functions

// #[derive(Serialize, Deserialize, Clone, ToSchema)]
// pub struct MigrateEncryptionKeyParams {
//     pub from_envelope_encryption_key_id: EnvelopeEncryptionKey,
//     pub to_envelope_encryption_key_id: EnvelopeEncryptionKey,
// }

// /// Parameters for migrating encryption keys by ARN/location
// /// This allows passing just the identifier (ARN or location) and the bridge will look up the full key details
// #[derive(Serialize, Deserialize, Clone, ToSchema)]
// pub struct MigrateEncryptionKeyByIdentifierParams {
//     /// Source encryption key identifier (ARN for AWS KMS, location path for local)
//     pub from: String,
//     /// Target encryption key identifier (ARN for AWS KMS, location path for local)
//     pub to: String,
// }

// /// Parameters for deleting data encryption keys by envelope encryption key identifier
// #[derive(Serialize, Deserialize, Clone, ToSchema)]
// pub struct DeleteDataEncryptionKeyByIdentifierParams {
//     /// Encryption key identifier (ARN for AWS KMS, location path for local)
//     pub identifier: String,
// }

// #[derive(Serialize, Deserialize, Clone, ToSchema)]
// pub struct MigrateEncryptionKeyResponse {
//     pub migrated_resource_server_credentials: usize,
//     pub migrated_user_credentials: usize,
//     pub migrated_data_encryption_keys: usize,
// }

// /// Migrate encryption keys by identifier (ARN or location)
// /// Looks up the full envelope encryption key details from the database
// /// Constructs EnvelopeEncryptionKeyContents from the found keys (region extracted from DB for AWS KMS)
// pub async fn migrate_encryption_key_by_identifier<R>(
//     _bridge_envelope_key: &EnvelopeEncryptionKeyContents,
//     on_config_change_tx: &OnConfigChangeTx,
//     repo: &R,
//     params: MigrateEncryptionKeyByIdentifierParams,
// ) -> Result<MigrateEncryptionKeyResponse, CommonError>
// where
//     R: crate::repository::ProviderRepositoryLike + DataEncryptionKeyRepositoryLike,
// {
//     use tracing::info;

//     // Find the from envelope encryption key by identifier
//     let from_envelope_key_id = if params.from.starts_with("arn:aws:kms:") {
//         find_envelope_encryption_key_by_arn(repo, &params.from)
//             .await?
//             .ok_or_else(|| {
//                 CommonError::Unknown(anyhow::anyhow!(
//                     "No data encryption key found with ARN: {}",
//                     params.from
//                 ))
//             })?
//     } else {
//         find_envelope_encryption_key_by_location(repo, &params.from)
//             .await?
//             .ok_or_else(|| {
//                 CommonError::Unknown(anyhow::anyhow!(
//                     "No data encryption key found with location: {}",
//                     params.from
//                 ))
//             })?
//     };

//     // Find the to envelope encryption key by identifier
//     let to_envelope_key_id = if params.to.starts_with("arn:aws:kms:") {
//         find_envelope_encryption_key_by_arn(repo, &params.to)
//             .await?
//             .ok_or_else(|| {
//                 CommonError::Unknown(anyhow::anyhow!(
//                     "No data encryption key found with ARN: {}",
//                     params.to
//                 ))
//             })?
//     } else {
//         find_envelope_encryption_key_by_location(repo, &params.to)
//             .await?
//             .ok_or_else(|| {
//                 CommonError::Unknown(anyhow::anyhow!(
//                     "No data encryption key found with location: {}",
//                     params.to
//                 ))
//             })?
//     };

//     info!(
//         "Found envelope encryption keys - from: {:?}, to: {:?}",
//         from_envelope_key_id, to_envelope_key_id
//     );

//     // Extract region and construct EnvelopeEncryptionKeyContents from the found keys
//     // For AWS KMS, use the ARN and region from the found key
//     // For local keys, load the key bytes from the file
//     let (from_key_contents, to_key_contents) = match (&from_envelope_key_id, &to_envelope_key_id) {
//         (
//             EnvelopeEncryptionKey::AwsKms { arn: from_arn, region: from_region },
//             EnvelopeEncryptionKey::AwsKms { arn: to_arn, region: to_region },
//         ) => {
//             // For AWS KMS, use the ARN and region from the found key
//             (
//                 EnvelopeEncryptionKeyContents::AwsKms {
//                     arn: from_arn.clone(),
//                     region: from_region.clone(),
//                 },
//                 EnvelopeEncryptionKeyContents::AwsKms {
//                     arn: to_arn.clone(),
//                     region: to_region.clone(),
//                 },
//             )
//         }
//         (
//             EnvelopeEncryptionKey::Local { location: from_loc },
//             EnvelopeEncryptionKey::Local { location: to_loc },
//         ) => {
//             // For local keys, load the key bytes from the file
//             let from_key_contents = encryption::get_or_create_local_encryption_key(
//                 &std::path::PathBuf::from(from_loc),
//             )?;
//             let to_key_contents = encryption::get_or_create_local_encryption_key(
//                 &std::path::PathBuf::from(to_loc),
//             )?;
            
//             // Extract key_bytes from the loaded contents
//             let (from_key_bytes, to_key_bytes) = match (&from_key_contents, &to_key_contents) {
//                 (
//                     EnvelopeEncryptionKeyContents::Local { key_bytes: from_bytes, .. },
//                     EnvelopeEncryptionKeyContents::Local { key_bytes: to_bytes, .. },
//                 ) => (from_bytes.clone(), to_bytes.clone()),
//                 _ => {
//                     return Err(CommonError::Unknown(anyhow::anyhow!(
//                         "Failed to load local encryption keys"
//                     )));
//                 }
//             };
            
//             (
//                 EnvelopeEncryptionKeyContents::Local {
//                     location: from_loc.clone(),
//                     key_bytes: from_key_bytes,
//                 },
//                 EnvelopeEncryptionKeyContents::Local {
//                     location: to_loc.clone(),
//                     key_bytes: to_key_bytes,
//                 },
//             )
//         }
//         _ => {
//             return Err(CommonError::Unknown(anyhow::anyhow!(
//                 "Mismatched envelope encryption key types"
//             )));
//         }
//     };

//     // Call the existing migrate function with the resolved keys
//     migrate_encryption_key(
//         &from_key_contents,
//         &to_key_contents,
//         on_config_change_tx,
//         repo,
//         MigrateEncryptionKeyParams {
//             from_envelope_encryption_key_id: from_envelope_key_id,
//             to_envelope_encryption_key_id: to_envelope_key_id,
//         },
//     )
//     .await
// }

// /// Delete data encryption keys by envelope encryption key identifier (ARN or location)
// /// Finds all DEKs using the specified envelope encryption key and deletes them
// pub async fn delete_data_encryption_key_by_identifier<R>(
//     on_config_change_tx: &OnConfigChangeTx,
//     repo: &R,
//     params: DeleteDataEncryptionKeyByIdentifierParams,
// ) -> Result<usize, CommonError>
// where
//     R: crate::repository::ProviderRepositoryLike + DataEncryptionKeyRepositoryLike,
// {
//     use shared::primitives::PaginationRequest;
//     use tracing::info;

//     // Find the envelope encryption key by identifier
//     let envelope_key_id = if params.identifier.starts_with("arn:aws:kms:") {
//         find_envelope_encryption_key_by_arn(repo, &params.identifier)
//             .await?
//             .ok_or_else(|| {
//                 CommonError::Unknown(anyhow::anyhow!(
//                     "No data encryption key found with ARN: {}",
//                     params.identifier
//                 ))
//             })?
//     } else {
//         find_envelope_encryption_key_by_location(repo, &params.identifier)
//             .await?
//             .ok_or_else(|| {
//                 CommonError::Unknown(anyhow::anyhow!(
//                     "No data encryption key found with location: {}",
//                     params.identifier
//                 ))
//             })?
//     };

//     info!("Found envelope encryption key: {:?}", envelope_key_id);

//     // Find all DEKs using this envelope encryption key and delete them
//     let mut deleted_count = 0;
//     let mut page_token = None;
//     loop {
//         let deks = encryption::list_data_encryption_keys(
//             repo,
//             PaginationRequest {
//                 page_size: 100,
//                 next_page_token: page_token.clone(),
//             },
//         )
//         .await?;

//         for dek_item in &deks.items {
//             if matches_envelope_key_id(&dek_item.envelope_encryption_key_id, &envelope_key_id) {
//                 info!("Deleting data encryption key: {}", dek_item.id);
//                 delete_data_encryption_key(
//                     on_config_change_tx,
//                     repo,
//                     dek_item.id.clone(), // DeleteDataEncryptionKeyParams is a String
//                     true,
//                 )
//                 .await?;
//                 deleted_count += 1;
//             }
//         }

//         if deks.next_page_token.is_none() {
//             break;
//         }
//         page_token = deks.next_page_token;
//     }

//     info!("Deleted {} data encryption key(s)", deleted_count);
//     Ok(deleted_count)
// }

// pub async fn migrate_encryption_key<R>(
//     from_envelope_key: &EnvelopeEncryptionKeyContents,
//     to_envelope_key: &EnvelopeEncryptionKeyContents,
//     on_config_change_tx: &OnConfigChangeTx,
//     repo: &R,
//     params: MigrateEncryptionKeyParams,
// ) -> Result<MigrateEncryptionKeyResponse, CommonError>
// where
//     R: crate::repository::ProviderRepositoryLike + DataEncryptionKeyRepositoryLike,
// {
//     use shared::primitives::PaginationRequest;
//     use tracing::info;

//     info!(
//         "Starting migration from {:?} to {:?}",
//         params.from_envelope_encryption_key_id, params.to_envelope_encryption_key_id
//     );

//     let mut migrated_resource_server_credentials = 0;
//     let mut migrated_user_credentials = 0;
//     let mut migrated_data_encryption_keys = 0;

//     // Step 1: Get all data encryption keys that use the "from" envelope encryption key
//     let mut page_token = None;
//     loop {
//         let deks = encryption::list_data_encryption_keys(
//             repo,
//             PaginationRequest {
//                 page_size: 100,
//                 next_page_token: page_token.clone(),
//             },
//         )
//         .await?;

//         for dek_item in &deks.items {
//             // Check if this DEK uses the "from" envelope encryption key
//             if !matches_envelope_key_id(
//                 &dek_item.envelope_encryption_key_id,
//                 &params.from_envelope_encryption_key_id,
//             ) {
//                 continue;
//             }

//             info!(
//                 "Processing DEK {} for migration",
//                 dek_item.id
//             );

//             // Get the full DEK
//             let dek = encryption::DataEncryptionKeyRepositoryLike::get_data_encryption_key_by_id(repo, &dek_item.id)
//                 .await?
//                 .ok_or_else(|| {
//                     CommonError::Unknown(anyhow::anyhow!("DEK {} not found", dek_item.id))
//                 })?;

//             // Step 2: Create a new DEK with the "to" envelope encryption key
//             let new_dek = encryption::create_data_encryption_key(
//                 to_envelope_key,
//                 repo,
//                 CreateDataEncryptionKeyParams {
//                     id: None,
//                     encrypted_data_envelope_key: None,
//                 },
//             )
//             .await?;

//             info!(
//                 "Created new DEK {} for migrated data",
//                 new_dek.id
//             );

//             // Step 3: Get old and new crypto services
//             let old_crypto_service =
//                 encryption::get_crypto_service(from_envelope_key, repo, &dek.id).await?;
//             let old_decryption_service = encryption::get_decryption_service(&old_crypto_service)?;

//             let new_crypto_service =
//                 encryption::get_crypto_service(to_envelope_key, repo, &new_dek.id).await?;
//             let new_encryption_service = encryption::get_encryption_service(&new_crypto_service)?;

//             // Step 4: Migrate all resource server credentials using this DEK
//             let (migrated_rs, _) = migrate_resource_server_credentials(
//                 repo,
//                 &dek.id,
//                 &new_dek.id,
//                 &old_decryption_service,
//                 &new_encryption_service,
//             )
//             .await?;
//             migrated_resource_server_credentials += migrated_rs;

//             // Step 5: Migrate all user credentials using this DEK
//             let (migrated_uc, _) = migrate_user_credentials(
//                 repo,
//                 &dek.id,
//                 &new_dek.id,
//                 &old_decryption_service,
//                 &new_encryption_service,
//             )
//             .await?;
//             migrated_user_credentials += migrated_uc;

//             // Step 6: Delete the old DEK
//             encryption::delete_data_encryption_key(repo, dek.id.clone()).await?;
//             migrated_data_encryption_keys += 1;

//             info!(
//                 "Successfully migrated DEK {} to {}",
//                 dek.id, new_dek.id
//             );
//         }

//         if deks.next_page_token.is_none() {
//             break;
//         }
//         page_token = deks.next_page_token;
//     }

//     // Trigger bridge on change
//     on_config_change_tx
//         .send(OnConfigChangeEvt::DataEncryptionKeyAdded(
//             // Send a dummy event just to trigger the bridge sync
//             DataEncryptionKey {
//                 id: "migration-completed".to_string(),
//                 envelope_encryption_key_id: params.to_envelope_encryption_key_id,
//                 encrypted_data_encryption_key: EncryptedDataEncryptionKey(String::new()),
//                 created_at: shared::primitives::WrappedChronoDateTime::now(),
//                 updated_at: shared::primitives::WrappedChronoDateTime::now(),
//             },
//         ))
//         .map_err(|e| {
//             CommonError::Unknown(anyhow::anyhow!("Failed to send config change event: {e}"))
//         })?;

//     info!(
//         "Migration completed: {} resource server credentials, {} user credentials, {} DEKs",
//         migrated_resource_server_credentials, migrated_user_credentials, migrated_data_encryption_keys
//     );

//     Ok(MigrateEncryptionKeyResponse {
//         migrated_resource_server_credentials,
//         migrated_user_credentials,
//         migrated_data_encryption_keys,
//     })
// }

// pub fn matches_envelope_key_id(
//     id1: &EnvelopeEncryptionKey,
//     id2: &EnvelopeEncryptionKey,
// ) -> bool {
//     match (id1, id2) {
//         (EnvelopeEncryptionKey::AwsKms { arn: arn1, region: region1 }, EnvelopeEncryptionKey::AwsKms { arn: arn2, region: region2 }) => {
//             arn1 == arn2 && region1 == region2
//         }
//         (EnvelopeEncryptionKey::Local { location: loc1 }, EnvelopeEncryptionKey::Local { location: loc2 }) => {
//             loc1 == loc2
//         }
//         _ => false,
//     }
// }

// /// Find envelope encryption key by ARN (for AWS KMS keys)
// /// Returns the full EnvelopeEncryptionKey with region extracted from stored data
// pub async fn find_envelope_encryption_key_by_arn<R>(
//     repo: &R,
//     arn: &str,
// ) -> Result<Option<EnvelopeEncryptionKey>, CommonError>
// where
//     R: DataEncryptionKeyRepositoryLike,
// {
//     use shared::primitives::PaginationRequest;

//     let mut page_token = None;
//     loop {
//         let deks = encryption::list_data_encryption_keys(
//             repo,
//             PaginationRequest {
//                 page_size: 100,
//                 next_page_token: page_token.clone(),
//             },
//         )
//         .await?;

//         for dek_item in &deks.items {
//             if let EnvelopeEncryptionKey::AwsKms { arn: stored_arn, .. } = &dek_item.envelope_encryption_key_id {
//                 if stored_arn == arn {
//                     return Ok(Some(dek_item.envelope_encryption_key_id.clone()));
//                 }
//             }
//         }

//         if deks.next_page_token.is_none() {
//             break;
//         }
//         page_token = deks.next_page_token;
//     }

//     Ok(None)
// }

// /// Find envelope encryption key by location (for local keys)
// /// Returns the full EnvelopeEncryptionKey
// pub async fn find_envelope_encryption_key_by_location<R>(
//     repo: &R,
//     location: &str,
// ) -> Result<Option<EnvelopeEncryptionKey>, CommonError>
// where
//     R: DataEncryptionKeyRepositoryLike,
// {
//     use shared::primitives::PaginationRequest;

//     let mut page_token = None;
//     loop {
//         let deks = encryption::list_data_encryption_keys(
//             repo,
//             PaginationRequest {
//                 page_size: 100,
//                 next_page_token: page_token.clone(),
//             },
//         )
//         .await?;

//         for dek_item in &deks.items {
//             if let EnvelopeEncryptionKey::Local { location: stored_location } = &dek_item.envelope_encryption_key_id {
//                 if stored_location == location {
//                     return Ok(Some(dek_item.envelope_encryption_key_id.clone()));
//                 }
//             }
//         }

//         if deks.next_page_token.is_none() {
//             break;
//         }
//         page_token = deks.next_page_token;
//     }

//     Ok(None)
// }

// async fn migrate_resource_server_credentials<R>(
//     repo: &R,
//     old_dek_id: &str,
//     _new_dek_id: &str,
//     old_decryption_service: &DecryptionService,
//     new_encryption_service: &EncryptionService,
// ) -> Result<(usize, Vec<String>), CommonError>
// where
//     R: crate::repository::ProviderRepositoryLike,
// {
//     use shared::primitives::PaginationRequest;
//     use tracing::info;

//     let mut migrated_count = 0;
//     let mut migrated_ids = Vec::new();
//     let mut page_token = None;

//     loop {
//         let creds = repo
//             .list_resource_server_credentials(&PaginationRequest {
//                 page_size: 100,
//                 next_page_token: page_token.clone(),
//             })
//             .await?;

//         for cred in &creds.items {
//             // Only migrate credentials using the old DEK
//             if cred.data_encryption_key_id != old_dek_id {
//                 continue;
//             }

//             // Get the provider controller for this credential
//             let provider_controller = match get_provider_controller_from_credential_type(&cred.type_id) {
//                 Ok(controller) => controller,
//                 Err(e) => {
//                     info!("Skipping credential {} (type {}): {}", cred.id, cred.type_id, e);
//                     continue;
//                 }
//             };

//             // Decrypt the credential
//             let decrypted_value = decrypt_credential_value(
//                 &provider_controller,
//                 old_decryption_service,
//                 &cred.value,
//             )
//             .await?;

//             // Re-encrypt with the new key
//             let encrypted_value = encrypt_credential_value(
//                 &provider_controller,
//                 new_encryption_service,
//                 decrypted_value,
//             )
//             .await?;

//             // Update the credential in the database
//             repo.update_resource_server_credential(
//                 &cred.id,
//                 Some(&encrypted_value),
//                 None,
//                 None,
//                 Some(&shared::primitives::WrappedChronoDateTime::now()),
//             )
//             .await?;

//             // Update the DEK ID (we need to add this method to the repository)
//             // For now, we'll leave it as a TODO
//             // TODO: Add update_resource_server_credential_dek_id method

//             migrated_count += 1;
//             migrated_ids.push(cred.id.to_string());
//             info!("Migrated resource server credential {}", cred.id);
//         }

//         if creds.next_page_token.is_none() {
//             break;
//         }
//         page_token = creds.next_page_token;
//     }

//     Ok((migrated_count, migrated_ids))
// }

// async fn migrate_user_credentials<R>(
//     repo: &R,
//     old_dek_id: &str,
//     _new_dek_id: &str,
//     old_decryption_service: &DecryptionService,
//     new_encryption_service: &EncryptionService,
// ) -> Result<(usize, Vec<String>), CommonError>
// where
//     R: crate::repository::ProviderRepositoryLike,
// {
//     use shared::primitives::PaginationRequest;
//     use tracing::info;

//     let mut migrated_count = 0;
//     let mut migrated_ids = Vec::new();
//     let mut page_token = None;

//     loop {
//         let creds = repo
//             .list_user_credentials(&PaginationRequest {
//                 page_size: 100,
//                 next_page_token: page_token.clone(),
//             })
//             .await?;

//         for cred in &creds.items {
//             // Only migrate credentials using the old DEK
//             if cred.data_encryption_key_id != old_dek_id {
//                 continue;
//             }

//             // Get the provider controller for this credential
//             let provider_controller = match get_provider_controller_from_credential_type(&cred.type_id) {
//                 Ok(controller) => controller,
//                 Err(e) => {
//                     info!("Skipping credential {} (type {}): {}", cred.id, cred.type_id, e);
//                     continue;
//                 }
//             };

//             // Decrypt the credential
//             let decrypted_value = decrypt_credential_value(
//                 &provider_controller,
//                 old_decryption_service,
//                 &cred.value,
//             )
//             .await?;

//             // Re-encrypt with the new key
//             let encrypted_value = encrypt_credential_value(
//                 &provider_controller,
//                 new_encryption_service,
//                 decrypted_value,
//             )
//             .await?;

//             // Update the credential in the database
//             repo.update_user_credential(
//                 &cred.id,
//                 Some(&encrypted_value),
//                 None,
//                 None,
//                 Some(&shared::primitives::WrappedChronoDateTime::now()),
//             )
//             .await?;

//             // Update the DEK ID (we need to add this method to the repository)
//             // For now, we'll leave it as a TODO
//             // TODO: Add update_user_credential_dek_id method

//             migrated_count += 1;
//             migrated_ids.push(cred.id.to_string());
//             info!("Migrated user credential {}", cred.id);
//         }

//         if creds.next_page_token.is_none() {
//             break;
//         }
//         page_token = creds.next_page_token;
//     }

//     Ok((migrated_count, migrated_ids))
// }

// // Helper functions to extract provider controller from credential type_id
// fn get_provider_controller_from_credential_type(
//     type_id: &str,
// ) -> Result<std::sync::Arc<dyn crate::logic::ProviderControllerLike>, CommonError> {
//     // The type_id typically follows patterns like "resource_server_oauth", "user_oauth", etc.
//     // We need to extract the provider type from this

//     // For now, this is a simplified implementation
//     // In a real implementation, we'd need to properly map type_ids to provider controllers

//     // Common patterns:
//     // - resource_server_oauth -> OAuth provider
//     // - resource_server_api_key -> API Key provider
//     // - user_oauth -> OAuth provider
//     // - user_api_key -> API Key provider

//     let provider_type = if type_id.contains("oauth") {
//         "oauth"
//     } else if type_id.contains("api_key") {
//         "api_key"
//     } else if type_id.contains("no_auth") {
//         "no_auth"
//     } else {
//         return Err(CommonError::Unknown(anyhow::anyhow!(
//             "Unknown credential type: {}",
//             type_id
//         )));
//     };

//     get_provider_controller(provider_type)
// }

// async fn decrypt_credential_value(
//     _provider_controller: &std::sync::Arc<dyn crate::logic::ProviderControllerLike>,
//     _decryption_service: &DecryptionService,
//     encrypted_value: &WrappedJsonValue,
// ) -> Result<WrappedJsonValue, CommonError> {
//     // This is a simplified implementation
//     // In a real implementation, we'd need to:
//     // 1. Deserialize the encrypted_value based on the credential type
//     // 2. Use the provider controller to decrypt specific fields
//     // 3. Return the decrypted value

//     // For now, we'll just return the encrypted value as-is
//     // This is a placeholder that would need proper implementation
//     // based on the specific credential controller's decrypt methods

//     // The actual decryption would use methods like:
//     // - controller.decrypt_api_key_credentials(decryption_service, credential)
//     // - controller.decrypt_oauth_credentials(decryption_service, credential)
//     // etc.

//     Ok(encrypted_value.clone())
// }

// async fn encrypt_credential_value(
//     _provider_controller: &std::sync::Arc<dyn crate::logic::ProviderControllerLike>,
//     _encryption_service: &EncryptionService,
//     decrypted_value: WrappedJsonValue,
// ) -> Result<WrappedJsonValue, CommonError> {
//     // This is a placeholder implementation
//     // In a real implementation, we'd need to use the provider controller
//     // to properly encrypt the credential based on its type

//     // For now, just return the decrypted value as-is
//     // This is a placeholder that would need proper implementation
//     Ok(decrypted_value)
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::logic::OnConfigChangeTx;
//     use crate::repository::Repository;
//     use rand::RngCore;
//     use shared::primitives::{PaginationRequest, SqlMigrationLoader};
//     use shared::test_utils::repository::setup_in_memory_database;

//     /// Helper function to create a temporary local key file
//     fn create_temp_local_key() -> (tempfile::NamedTempFile, EnvelopeEncryptionKeyContents) {
//         let mut kek_bytes = [0u8; 32];
//         rand::thread_rng().fill_bytes(&mut kek_bytes);

//         let temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
//         std::fs::write(temp_file.path(), kek_bytes).expect("Failed to write KEK to temp file");

//         let location = temp_file.path().to_string_lossy().to_string();

//         let contents = EnvelopeEncryptionKeyContents::Local {
//             location: location.clone(),
//             key_bytes: kek_bytes.to_vec(),
//         };

//         (temp_file, contents)
//     }

//     /// Helper function to get AWS KMS key by alias
//     #[allow(dead_code)]
//     fn get_aws_kms_key_by_alias() -> EnvelopeEncryptionKeyContents {
//         let alias = "alias/unsafe-github-action-soma-test-key".to_string();
//         let region = "eu-west-2".to_string();

//         EnvelopeEncryptionKeyContents::AwsKms {
//             arn: alias, // Using alias as ARN - the encryption library should handle this
//             region,
//         }
//     }

//     #[tokio::test]
//     async fn test_create_data_encryption_key_with_local_key() {
//         shared::setup_test!();

//         // Setup in-memory database
//         let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
//             .await
//             .unwrap();
//         let repo = Repository::new(conn);
//         let (tx, _rx): (OnConfigChangeTx, _) = tokio::sync::broadcast::channel(100);

//         // Create a local key
//         let (_temp_file, local_key) = create_temp_local_key();

//         // Create a data encryption key
//         let dek = create_data_encryption_key(
//             &local_key,
//             &tx,
//             &repo,
//             CreateDataEncryptionKeyParams {
//                 id: Some("test-dek-local".to_string()),
//                 encrypted_data_envelope_key: None,
//             },
//             false,
//         )
//         .await
//         .unwrap();

//         assert_eq!(dek.id, "test-dek-local");
//         assert!(matches!(
//             dek.envelope_encryption_key_id,
//             EnvelopeEncryptionKey::Local { .. }
//         ));

//         // Verify the DEK exists in the database
//         let retrieved_dek =
//             DataEncryptionKeyRepositoryLike::get_data_encryption_key_by_id(&repo, &dek.id)
//                 .await
//                 .unwrap();
//         assert!(retrieved_dek.is_some());
//     }

//     #[tokio::test]
//     async fn test_delete_data_encryption_key_with_local_key() {
//         shared::setup_test!();

//         // Setup in-memory database
//         let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
//             .await
//             .unwrap();
//         let repo = Repository::new(conn);
//         let (tx, _rx): (OnConfigChangeTx, _) = tokio::sync::broadcast::channel(100);

//         // Create a local key
//         let (_temp_file, local_key) = create_temp_local_key();

//         // Create a data encryption key
//         let dek = create_data_encryption_key(
//             &local_key,
//             &tx,
//             &repo,
//             CreateDataEncryptionKeyParams {
//                 id: Some("test-dek-local-delete".to_string()),
//                 encrypted_data_envelope_key: None,
//             },
//             false,
//         )
//         .await
//         .unwrap();

//         // Delete the DEK
//         delete_data_encryption_key(&tx, &repo, dek.id.clone(), false)
//             .await
//             .unwrap();

//         // Verify the DEK is deleted
//         let deleted_dek =
//             DataEncryptionKeyRepositoryLike::get_data_encryption_key_by_id(&repo, &dek.id)
//                 .await
//                 .unwrap();
//         assert!(deleted_dek.is_none());
//     }

//     #[tokio::test]
//     async fn test_create_multiple_local_keys() {
//         shared::setup_test!();

//         let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
//             .await
//             .unwrap();
//         let repo = Repository::new(conn);
//         let (tx, _rx): (OnConfigChangeTx, _) = tokio::sync::broadcast::channel(100);

//         // Create first local key
//         let (_temp_file1, local_key1) = create_temp_local_key();
//         let dek1 = create_data_encryption_key(
//             &local_key1,
//             &tx,
//             &repo,
//             CreateDataEncryptionKeyParams {
//                 id: Some("test-dek-local-1".to_string()),
//                 encrypted_data_envelope_key: None,
//             },
//             false,
//         )
//         .await
//         .unwrap();

//         // Create second local key
//         let (_temp_file2, local_key2) = create_temp_local_key();
//         let dek2 = create_data_encryption_key(
//             &local_key2,
//             &tx,
//             &repo,
//             CreateDataEncryptionKeyParams {
//                 id: Some("test-dek-local-2".to_string()),
//                 encrypted_data_envelope_key: None,
//             },
//             false,
//         )
//         .await
//         .unwrap();

//         // Verify both DEKs exist
//         assert_eq!(dek1.id, "test-dek-local-1");
//         assert_eq!(dek2.id, "test-dek-local-2");

//         // List DEKs
//         let deks = encryption::list_data_encryption_keys(
//             &repo,
//             PaginationRequest {
//                 page_size: 100,
//                 next_page_token: None,
//             },
//         )
//         .await
//         .unwrap();

//         assert!(deks.items.len() >= 2);
//     }

//     #[tokio::test]
//     async fn test_create_data_encryption_key_with_aws_kms() {
//         shared::setup_test!();

//         let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
//             .await
//             .unwrap();
//         let repo = Repository::new(conn);
//         let (tx, _rx): (OnConfigChangeTx, _) = tokio::sync::broadcast::channel(100);

//         // Get AWS KMS key by alias
//         let aws_key = get_aws_kms_key_by_alias();

//         // Create a data encryption key
//         let dek = create_data_encryption_key(
//             &aws_key,
//             &tx,
//             &repo,
//             CreateDataEncryptionKeyParams {
//                 id: Some("test-dek-aws".to_string()),
//                 encrypted_data_envelope_key: None,
//             },
//             false,
//         )
//         .await
//         .unwrap();

//         assert_eq!(dek.id, "test-dek-aws");
//         assert!(matches!(
//             dek.envelope_encryption_key_id,
//             EnvelopeEncryptionKey::AwsKms { .. }
//         ));

//         // Verify the DEK exists in the database
//         let retrieved_dek =
//             DataEncryptionKeyRepositoryLike::get_data_encryption_key_by_id(&repo, &dek.id)
//                 .await
//                 .unwrap();
//         assert!(retrieved_dek.is_some());
//     }

//     #[tokio::test]
//     async fn test_delete_data_encryption_key_with_aws_kms() {
//         shared::setup_test!();

//         let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
//             .await
//             .unwrap();
//         let repo = Repository::new(conn);
//         let (tx, _rx): (OnConfigChangeTx, _) = tokio::sync::broadcast::channel(100);

//         // Get AWS KMS key by alias
//         let aws_key = get_aws_kms_key_by_alias();

//         // Create a data encryption key
//         let dek = create_data_encryption_key(
//             &aws_key,
//             &tx,
//             &repo,
//             CreateDataEncryptionKeyParams {
//                 id: Some("test-dek-aws-delete".to_string()),
//                 encrypted_data_envelope_key: None,
//             },
//             false,
//         )
//         .await
//         .unwrap();

//         // Delete the DEK
//         delete_data_encryption_key(&tx, &repo, dek.id.clone(), false)
//             .await
//             .unwrap();

//         // Verify the DEK is deleted
//         let deleted_dek =
//             DataEncryptionKeyRepositoryLike::get_data_encryption_key_by_id(&repo, &dek.id)
//                 .await
//                 .unwrap();
//         assert!(deleted_dek.is_none());
//     }

//     #[tokio::test]
//     async fn test_migrate_encryption_key_between_local_keys() {
//         shared::setup_test!();

//         let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
//             .await
//             .unwrap();
//         let repo = Repository::new(conn);
//         let (tx, _rx): (OnConfigChangeTx, _) = tokio::sync::broadcast::channel(100);

//         // Create two local keys
//         let (_temp_file1, local_key1) = create_temp_local_key();
//         let local_key1_id =
//             if let EnvelopeEncryptionKeyContents::Local { location, .. } = &local_key1 {
//                 EnvelopeEncryptionKey::Local {
//                     location: location.clone(),
//                 }
//             } else {
//                 panic!("Expected local key");
//             };

//         let (_temp_file2, local_key2) = create_temp_local_key();
//         let local_key2_id =
//             if let EnvelopeEncryptionKeyContents::Local { location, .. } = &local_key2 {
//                 EnvelopeEncryptionKey::Local {
//                     location: location.clone(),
//                 }
//             } else {
//                 panic!("Expected local key");
//             };

//         // Create a DEK with the first key
//         let dek1 = create_data_encryption_key(
//             &local_key1,
//             &tx,
//             &repo,
//             CreateDataEncryptionKeyParams {
//                 id: Some("test-dek-migration-1".to_string()),
//                 encrypted_data_envelope_key: None,
//             },
//             false,
//         )
//         .await
//         .unwrap();

//         // Perform migration from local_key1 to local_key2
//         let migration_result = migrate_encryption_key(
//             &local_key1,
//             &local_key2,
//             &tx,
//             &repo,
//             MigrateEncryptionKeyParams {
//                 from_envelope_encryption_key_id: local_key1_id.clone(),
//                 to_envelope_encryption_key_id: local_key2_id.clone(),
//             },
//         )
//         .await
//         .unwrap();

//         // Verify migration results
//         assert_eq!(migration_result.migrated_data_encryption_keys, 1);

//         // Verify the old DEK is gone
//         let old_dek =
//             DataEncryptionKeyRepositoryLike::get_data_encryption_key_by_id(&repo, &dek1.id)
//                 .await
//                 .unwrap();
//         assert!(old_dek.is_none());

//         // Verify a new DEK was created with the new envelope key
//         let deks = encryption::list_data_encryption_keys(
//             &repo,
//             PaginationRequest {
//                 page_size: 100,
//                 next_page_token: None,
//             },
//         )
//         .await
//         .unwrap();

//         // Should have at least one DEK with the new envelope key
//         assert!(deks.items.len() >= 1);
//     }

//     #[tokio::test]
//     async fn test_migrate_with_no_credentials() {
//         shared::setup_test!();

//         let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
//             .await
//             .unwrap();
//         let repo = Repository::new(conn);
//         let (tx, _rx): (OnConfigChangeTx, _) = tokio::sync::broadcast::channel(100);

//         // Create two local keys
//         let (_temp_file1, local_key1) = create_temp_local_key();
//         let local_key1_id =
//             if let EnvelopeEncryptionKeyContents::Local { location, .. } = &local_key1 {
//                 EnvelopeEncryptionKey::Local {
//                     location: location.clone(),
//                 }
//             } else {
//                 panic!("Expected local key");
//             };

//         let (_temp_file2, local_key2) = create_temp_local_key();
//         let local_key2_id =
//             if let EnvelopeEncryptionKeyContents::Local { location, .. } = &local_key2 {
//                 EnvelopeEncryptionKey::Local {
//                     location: location.clone(),
//                 }
//             } else {
//                 panic!("Expected local key");
//             };

//         // Create a DEK but no credentials
//         let _dek = create_data_encryption_key(
//             &local_key1,
//             &tx,
//             &repo,
//             CreateDataEncryptionKeyParams {
//                 id: Some("test-dek-no-creds".to_string()),
//                 encrypted_data_envelope_key: None,
//             },
//             false,
//         )
//         .await
//         .unwrap();

//         // Perform migration
//         let migration_result = migrate_encryption_key(
//             &local_key1,
//             &local_key2,
//             &tx,
//             &repo,
//             MigrateEncryptionKeyParams {
//                 from_envelope_encryption_key_id: local_key1_id.clone(),
//                 to_envelope_encryption_key_id: local_key2_id.clone(),
//             },
//         )
//         .await
//         .unwrap();

//         // Verify no credentials were migrated, but the DEK was
//         assert_eq!(migration_result.migrated_resource_server_credentials, 0);
//         assert_eq!(migration_result.migrated_user_credentials, 0);
//         assert_eq!(migration_result.migrated_data_encryption_keys, 1);
//     }

//     #[tokio::test]
//     async fn test_migrate_from_local_to_aws_kms() {
//         shared::setup_test!();

//         let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
//             .await
//             .unwrap();
//         let repo = Repository::new(conn);
//         let (tx, _rx): (OnConfigChangeTx, _) = tokio::sync::broadcast::channel(100);

//         // Create a local key
//         let (_temp_file, local_key) = create_temp_local_key();
//         let local_key_id =
//             if let EnvelopeEncryptionKeyContents::Local { location, .. } = &local_key {
//                 EnvelopeEncryptionKey::Local {
//                     location: location.clone(),
//                 }
//             } else {
//                 panic!("Expected local key");
//             };

//         // Get AWS KMS key
//         let aws_key = get_aws_kms_key_by_alias();
//         let aws_key_id = if let EnvelopeEncryptionKeyContents::AwsKms { arn, region, .. } = &aws_key {
//             EnvelopeEncryptionKey::AwsKms { arn: arn.clone(), region: region.clone() }
//         } else {
//             panic!("Expected AWS KMS key");
//         };

//         // Create a DEK with the local key
//         let _dek = create_data_encryption_key(
//             &local_key,
//             &tx,
//             &repo,
//             CreateDataEncryptionKeyParams {
//                 id: Some("test-dek-local-to-aws".to_string()),
//                 encrypted_data_envelope_key: None,
//             },
//             false,
//         )
//         .await
//         .unwrap();

//         // Perform migration from local to AWS KMS
//         let migration_result = migrate_encryption_key(
//             &local_key,
//             &aws_key,
//             &tx,
//             &repo,
//             MigrateEncryptionKeyParams {
//                 from_envelope_encryption_key_id: local_key_id.clone(),
//                 to_envelope_encryption_key_id: aws_key_id.clone(),
//             },
//         )
//         .await
//         .unwrap();

//         // Verify migration results
//         assert_eq!(migration_result.migrated_data_encryption_keys, 1);

//         // Verify new DEKs were created with AWS KMS
//         let deks = encryption::list_data_encryption_keys(
//             &repo,
//             PaginationRequest {
//                 page_size: 100,
//                 next_page_token: None,
//             },
//         )
//         .await
//         .unwrap();

//         // Should have at least one DEK with the AWS KMS envelope key
//         let aws_deks: Vec<_> = deks
//             .items
//             .iter()
//             .filter(|dek| {
//                 matches!(
//                     dek.envelope_encryption_key_id,
//                     EnvelopeEncryptionKey::AwsKms { .. }
//                 )
//             })
//             .collect();
//         assert!(aws_deks.len() >= 1);
//     }

//     #[tokio::test]
//     async fn test_migrate_from_local_to_kms_managed_key() {
//         shared::setup_test!();

//         let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
//             .await
//             .unwrap();
//         let repo = Repository::new(conn);
//         let (tx, _rx): (OnConfigChangeTx, _) = tokio::sync::broadcast::channel(100);

//         // Create a local key
//         let (_temp_file, local_key) = create_temp_local_key();
//         let local_key_id =
//             if let EnvelopeEncryptionKeyContents::Local { location, .. } = &local_key {
//                 EnvelopeEncryptionKey::Local {
//                     location: location.clone(),
//                 }
//             } else {
//                 panic!("Expected local key");
//             };

//         // Get AWS KMS managed key (using alias)
//         let kms_managed_key = get_aws_kms_key_by_alias();
//         let kms_managed_key_id =
//             if let EnvelopeEncryptionKeyContents::AwsKms { arn, region, .. } = &kms_managed_key {
//                 EnvelopeEncryptionKey::AwsKms { arn: arn.clone(), region: region.clone() }
//             } else {
//                 panic!("Expected AWS KMS key");
//             };

//         // Create a DEK with the local key and some test credentials
//         let dek = create_data_encryption_key(
//             &local_key,
//             &tx,
//             &repo,
//             CreateDataEncryptionKeyParams {
//                 id: Some("test-dek-local-to-kms-managed".to_string()),
//                 encrypted_data_envelope_key: None,
//             },
//             false,
//         )
//         .await
//         .unwrap();

//         // Verify the DEK was created with local key
//         assert!(matches!(
//             dek.envelope_encryption_key_id,
//             EnvelopeEncryptionKey::Local { .. }
//         ));

//         // Perform migration from local to KMS managed key
//         let migration_result = migrate_encryption_key(
//             &local_key,
//             &kms_managed_key,
//             &tx,
//             &repo,
//             MigrateEncryptionKeyParams {
//                 from_envelope_encryption_key_id: local_key_id.clone(),
//                 to_envelope_encryption_key_id: kms_managed_key_id.clone(),
//             },
//         )
//         .await
//         .unwrap();

//         // Verify migration results
//         assert_eq!(migration_result.migrated_data_encryption_keys, 1);
//         assert_eq!(migration_result.migrated_resource_server_credentials, 0);
//         assert_eq!(migration_result.migrated_user_credentials, 0);

//         // Verify the old DEK with local key is deleted
//         let old_dek =
//             DataEncryptionKeyRepositoryLike::get_data_encryption_key_by_id(&repo, &dek.id)
//                 .await
//                 .unwrap();
//         assert!(old_dek.is_none());

//         // Verify new DEKs were created with AWS KMS
//         let deks = encryption::list_data_encryption_keys(
//             &repo,
//             PaginationRequest {
//                 page_size: 100,
//                 next_page_token: None,
//             },
//         )
//         .await
//         .unwrap();

//         // Should have at least one DEK with the AWS KMS envelope key
//         let aws_deks: Vec<_> = deks
//             .items
//             .iter()
//             .filter(|dek| {
//                 matches!(
//                     dek.envelope_encryption_key_id,
//                     EnvelopeEncryptionKey::AwsKms { .. }
//                 )
//             })
//             .collect();
//         assert!(aws_deks.len() >= 1);

//         // Verify the new DEK has the correct KMS managed key ID
//         let new_dek = &aws_deks[0];
//         if let EnvelopeEncryptionKey::AwsKms { arn, region: _ } = &new_dek.envelope_encryption_key_id {
//             assert_eq!(arn, "alias/unsafe-github-action-soma-test-key");
//         } else {
//             panic!("Expected AWS KMS key");
//         }
//     }

//     #[tokio::test]
//     async fn test_migrate_from_aws_kms_to_local() {
//         shared::setup_test!();

//         let (_db, conn) = setup_in_memory_database(vec![Repository::load_sql_migrations()])
//             .await
//             .unwrap();
//         let repo = Repository::new(conn);
//         let (tx, _rx): (OnConfigChangeTx, _) = tokio::sync::broadcast::channel(100);

//         // Get AWS KMS key
//         let aws_key = get_aws_kms_key_by_alias();
//         let aws_key_id = if let EnvelopeEncryptionKeyContents::AwsKms { arn, region, .. } = &aws_key {
//             EnvelopeEncryptionKey::AwsKms { arn: arn.clone(), region: region.clone() }
//         } else {
//             panic!("Expected AWS KMS key");
//         };

//         // Create a local key
//         let (_temp_file, local_key) = create_temp_local_key();
//         let local_key_id =
//             if let EnvelopeEncryptionKeyContents::Local { location, .. } = &local_key {
//                 EnvelopeEncryptionKey::Local {
//                     location: location.clone(),
//                 }
//             } else {
//                 panic!("Expected local key");
//             };

//         // Create a DEK with the AWS KMS key
//         let _dek = create_data_encryption_key(
//             &aws_key,
//             &tx,
//             &repo,
//             CreateDataEncryptionKeyParams {
//                 id: Some("test-dek-aws-to-local".to_string()),
//                 encrypted_data_envelope_key: None,
//             },
//             false,
//         )
//         .await
//         .unwrap();

//         // Perform migration from AWS KMS to local
//         let migration_result = migrate_encryption_key(
//             &aws_key,
//             &local_key,
//             &tx,
//             &repo,
//             MigrateEncryptionKeyParams {
//                 from_envelope_encryption_key_id: aws_key_id.clone(),
//                 to_envelope_encryption_key_id: local_key_id.clone(),
//             },
//         )
//         .await
//         .unwrap();

//         // Verify migration results
//         assert_eq!(migration_result.migrated_data_encryption_keys, 1);

//         // Verify new DEKs were created with local key
//         let deks = encryption::list_data_encryption_keys(
//             &repo,
//             PaginationRequest {
//                 page_size: 100,
//                 next_page_token: None,
//             },
//         )
//         .await
//         .unwrap();

//         // Should have at least one DEK with the local envelope key
//         let local_deks: Vec<_> = deks
//             .items
//             .iter()
//             .filter(|dek| {
//                 matches!(
//                     dek.envelope_encryption_key_id,
//                     EnvelopeEncryptionKey::Local { .. }
//                 )
//             })
//             .collect();
//         assert!(local_deks.len() >= 1);
//     }
// }
