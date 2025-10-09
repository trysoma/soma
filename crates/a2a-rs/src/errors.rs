use axum::{
    Json,
    response::{IntoResponse, Response},
};
// Type imports are used in the ErrorBuilder pattern below
use derive_builder::Builder;
use serde::{Serialize, Serializer};
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Debug, Builder, Serialize, ToSchema)]
#[builder(
    pattern = "owned",
    setter(into),
    build_fn(error = "std::convert::Infallible")
)]
pub struct Error {
    #[builder(default)]
    pub message: String,
    #[builder(default)]
    pub data: Option<serde_json::Value>,
    #[builder(default)]
    #[serde(skip_serializing)]
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

#[derive(Debug, Error, ToSchema)]
#[schema(as = Error)]
pub enum A2aServerError {
    #[error("JSON parse error")]
    JsonParseError(Error),
    #[error("Invalid request: {0:?}")]
    InvalidRequest(Error),
    #[error("Method not found: {0:?}")]
    MethodNotFoundError(Error),
    #[error("Invalid params: {0:?}")]
    InvalidParamsError(Error),
    #[error("Internal error: {0:?}")]
    InternalError(Error),
    #[error("Task not found error: {0:?}")]
    TaskNotFoundError(Error),
    #[error("Task not cancelable error: {0:?}")]
    TaskNotCancelableError(Error),
    #[error("Push notification not supported error: {0:?}")]
    PushNotificationNotSupportedError(Error),
    #[error("Unsupported operation error: {0:?}")]
    UnsupportedOperationError(Error),
    #[error("Content type not suppoerted error: {0:?}")]
    ContentTypeNotSupportedError(Error),
    #[error("Invalid agent response error: {0:?}")]
    InvalidAgentResponseError(Error),
}
impl Serialize for A2aServerError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            A2aServerError::InternalError(err)
            | A2aServerError::TaskNotFoundError(err)
            | A2aServerError::JsonParseError(err)
            | A2aServerError::InvalidRequest(err)
            | A2aServerError::MethodNotFoundError(err)
            | A2aServerError::InvalidParamsError(err)
            | A2aServerError::TaskNotCancelableError(err)
            | A2aServerError::PushNotificationNotSupportedError(err)
            | A2aServerError::UnsupportedOperationError(err)
            | A2aServerError::ContentTypeNotSupportedError(err)
            | A2aServerError::InvalidAgentResponseError(err) => err.serialize(serializer),
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    name: String,
    message: String,
}

impl IntoResponse for A2aServerError {
    fn into_response(self) -> Response {
        match self {
            A2aServerError::JsonParseError(err)
            | A2aServerError::InvalidRequest(err)
            | A2aServerError::MethodNotFoundError(err)
            | A2aServerError::InvalidParamsError(err)
            | A2aServerError::TaskNotCancelableError(err)
            | A2aServerError::PushNotificationNotSupportedError(err)
            | A2aServerError::UnsupportedOperationError(err)
            | A2aServerError::ContentTypeNotSupportedError(err)
            | A2aServerError::InternalError(err)
            | A2aServerError::TaskNotFoundError(err)
            | A2aServerError::InvalidAgentResponseError(err) => (
                http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    name: "UnknownError".into(),
                    message: err.message,
                }),
            )
                .into_response(),
        }
    }
}

impl A2aServerError {
    // TODO: map errors codes
    pub fn json_rpc_code(&self) -> i32 {
        match self {
            A2aServerError::JsonParseError(_) => -32700,
            A2aServerError::InvalidRequest(_) => -32600,
            A2aServerError::MethodNotFoundError(_) => -32601,
            A2aServerError::InvalidParamsError(_) => -32602,
            A2aServerError::InternalError(_) => -32603,
            A2aServerError::TaskNotFoundError(_) => -32001,
            A2aServerError::TaskNotCancelableError(_) => -32002,
            A2aServerError::PushNotificationNotSupportedError(_) => -32003,
            A2aServerError::UnsupportedOperationError(_) => -32004,
            A2aServerError::ContentTypeNotSupportedError(_) => -32005,
            A2aServerError::InvalidAgentResponseError(_) => -32006,
        }
    }

    pub fn message(&self) -> String {
        match &self {
            A2aServerError::JsonParseError(v) => v.message.clone(),
            A2aServerError::InvalidRequest(v) => v.message.clone(),
            A2aServerError::MethodNotFoundError(v) => v.message.clone(),
            A2aServerError::InvalidParamsError(v) => v.message.clone(),
            A2aServerError::InternalError(v) => v.message.clone(),
            A2aServerError::TaskNotFoundError(v) => v.message.clone(),
            A2aServerError::TaskNotCancelableError(v) => v.message.clone(),
            A2aServerError::PushNotificationNotSupportedError(v) => v.message.clone(),
            A2aServerError::UnsupportedOperationError(v) => v.message.clone(),
            A2aServerError::ContentTypeNotSupportedError(v) => v.message.clone(),
            A2aServerError::InvalidAgentResponseError(v) => v.message.clone(),
        }
    }

    pub fn data(&self) -> Option<serde_json::Value> {
        match &self {
            A2aServerError::JsonParseError(v) => v.data.clone(),
            A2aServerError::InvalidRequest(v) => v.data.clone(),
            A2aServerError::MethodNotFoundError(v) => v.data.clone(),
            A2aServerError::InvalidParamsError(v) => v.data.clone(),
            A2aServerError::InternalError(v) => v.data.clone(),
            A2aServerError::TaskNotFoundError(v) => v.data.clone(),
            A2aServerError::TaskNotCancelableError(v) => v.data.clone(),
            A2aServerError::PushNotificationNotSupportedError(v) => v.data.clone(),
            A2aServerError::UnsupportedOperationError(v) => v.data.clone(),
            A2aServerError::ContentTypeNotSupportedError(v) => v.data.clone(),
            A2aServerError::InvalidAgentResponseError(v) => v.data.clone(),
        }
    }
}
