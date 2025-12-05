use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::logic::token_mapping::template::JwtTokenMappingConfig;

pub mod dynamic;
pub mod template;

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum TokenMapping {
    JwtTemplate(JwtTokenMappingConfig),
}
