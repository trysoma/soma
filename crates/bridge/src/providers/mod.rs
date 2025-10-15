// use enum_dispatch::enum_dispatch;
// use serde::{Deserialize, Serialize};
// use crate::logic::ProviderControllerLike;

pub mod google_mail;

pub const MAIL_CATEGORY: &str = "mail";

// #[enum_dispatch(ProviderControllerLike)]
pub enum ProviderController {
    GoogleMail(google_mail::GoogleMailController),
}


// #[macro_export]
// macro_rules! define_provider {
//     (
//         $provider_id:ident {
//         id: $id:literal,
//         name: $name:literal,
//         docs: $docs:literal,
//         flows: [$( $flow:ident ),+ $(,)?],
//         default_scopes: [$( $scope:literal ),* $(,)?]
//         }
//     ) => {
//         paste::paste! {
//             use serde::{Serialize, Deserialize};
//             use crate::logic::*;
//             use shared::{error::CommonError, primitives::{WrappedChronoDateTime, WrappedUuidV4}};

//             // ------------------------------
//             // Provider controller
//             // ------------------------------
//             pub struct [<$provider_id:camel Controller>];

//             impl ProviderControllerLike for [<$provider_id:camel Controller>] {
//                 type ProviderInstance = [<$provider_id:camel Instance>];

//                 async fn save_resource_server_credential(
//                     input: ResourceServerCredentialVariant,
//                 ) -> Result<ResourceServerCredential, CommonError> {
//                     match input {
//                         $(
//                             ResourceServerCredentialVariant::$flow(_) => (),
//                         )+
//                         _ => return Err(CommonError::InvalidRequest {
//                             msg: concat!("Unsupported credential type for ", stringify!($provider_id)).into(),
//                             source: None,
//                         }),
//                     };
//                     Ok(ResourceServerCredential {
//                         id: WrappedUuidV4::new(),
//                         created_at: WrappedChronoDateTime::now(),
//                         updated_at: WrappedChronoDateTime::now(),
//                         inner: input,
//                         metadata: Metadata::new(),
//                     })
//                 }

//                 async fn save_user_credential(
//                     input: UserCredentialVariant,
//                 ) -> Result<UserCredential, CommonError> {
//                     match input {
//                         $(
//                             UserCredentialVariant::$flow(_) => (),
//                         )+
//                         _ => return Err(CommonError::InvalidRequest {
//                             msg: concat!("Unsupported user credential type for ", stringify!($provider_id)).into(),
//                             source: None,
//                         }),
//                     };
//                     Ok(UserCredential {
//                         id: WrappedUuidV4::new(),
//                         created_at: WrappedChronoDateTime::now(),
//                         updated_at: WrappedChronoDateTime::now(),
//                         inner: input,
//                         metadata: Metadata::new(),
//                     })
//                 }

//                 async fn get_static_credentials(variant: StaticCredentialConfigurationType) -> Result<StaticCredentialConfiguration, CommonError> {
//                     match variant {
//                         $(
//                             StaticCredentialConfigurationType::$flow => (),
//                         )+
//                         _ => return Err(CommonError::InvalidRequest {
//                             msg: concat!("Unsupported static credential configuration type for ", stringify!($provider_id)).into(),
//                             source: None,
//                         }),
//                     };
//                     Ok(StaticCredentialConfiguration {
//                         inner: variant,
//                         metadata: Metadata::new(),
//                     })
//                 }

//                 fn id() -> String { $id.to_string() }
//                 fn name() -> String { $name.to_string() }
//                 fn documentation_url() -> String { $docs.to_string() }
//             }

            

//             // ------------------------------
//             // Flow variant enum
//             // ------------------------------
//             #[derive(Serialize, Deserialize)]
//             #[serde(tag = "type", rename_all = "snake_case")]
//             pub enum [<$provider_id:camel Variant>] {
//                 $(
//                     $flow(
//                         [<$flow FullCredential>]
//                     ),
//                 )+
//             }

//             #[derive(Serialize, Deserialize)]
//             #[serde(transparent)]
//             pub struct [<$provider_id:camel Instance>](pub [<$provider_id:camel Variant>]);

            
//         }
//     };
// }