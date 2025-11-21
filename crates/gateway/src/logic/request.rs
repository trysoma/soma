// AI Gateway request handling logic
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GatewayRequest {
    pub model: String,
    pub prompt: String,
    pub max_tokens: Option<u32>,
}
