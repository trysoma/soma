use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use shared::primitives::{WrappedChronoDateTime, WrappedUuidV4};
use once_cell::sync::Lazy;
use utoipa::ToSchema;
use std::sync::{Arc, RwLock};

// use enum_dispatch::enum_dispatch;
// use serde::{Deserialize, Serialize};
use crate::logic::{FunctionControllerLike, ProviderControllerLike, ProviderInstanceLike};

pub mod google_mail;


