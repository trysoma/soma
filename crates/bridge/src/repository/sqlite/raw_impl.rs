// Note: Since the generated code doesn't have query functions that return rows yet,
// we're not implementing TryFrom for row types here. The type conversions are primarily
// handled in the repository trait implementation using the CreateResourceServerCredential
// and CreateUserCredential structs in mod.rs, which convert domain types to database types.
//
// If you add query functions to provider.sql that return rows (e.g., get_resource_server_credential_by_id),
// you would add TryFrom implementations here similar to the soma crate pattern:
//
// impl TryFrom<Row_get_resource_server_credential_by_id> for ResourceServerCredential {
//     type Error = CommonError;
//     fn try_from(row: Row_get_resource_server_credential_by_id) -> Result<Self, Self::Error> {
//         let metadata: Metadata = serde_json::from_value(row.metadata.get_inner().clone())?;
//         let inner: ResourceServerCredentialVariant = serde_json::from_value(row.credential_data.get_inner().clone())?;
//         Ok(ResourceServerCredential {
//             id: row.id,
//             inner,
//             metadata,
//             created_at: row.created_at,
//             updated_at: row.updated_at,
//         })
//     }
// }
