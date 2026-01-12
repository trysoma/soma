//! A2A Protocol Error Types
//!
//! Defines the error types for A2A server operations, including JSON-RPC error codes.

use axum::{
    Json,
    response::{IntoResponse, Response},
};
use serde::{Serialize, Serializer};
use thiserror::Error;
use utoipa::ToSchema;

/// Internal error structure with optional source
#[derive(Debug, Serialize, ToSchema)]
pub struct A2aError {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing)]
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl A2aError {
    /// Create a new A2aError with just a message
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            data: None,
            source: None,
        }
    }

    /// Create a new A2aError with message and data
    pub fn with_data(message: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            message: message.into(),
            data: Some(data),
            source: None,
        }
    }

    /// Create a new A2aError with message and source error
    pub fn with_source(
        message: impl Into<String>,
        source: Box<dyn std::error::Error + Send + Sync>,
    ) -> Self {
        Self {
            message: message.into(),
            data: None,
            source: Some(source),
        }
    }
}

impl Default for A2aError {
    fn default() -> Self {
        Self {
            message: String::new(),
            data: None,
            source: None,
        }
    }
}

/// A2A Server Error enum covering all possible error types
#[derive(Debug, Error, ToSchema)]
#[schema(as = A2aError)]
pub enum A2aServerError {
    #[error("JSON parse error")]
    JsonParseError(A2aError),
    #[error("Invalid request: {0:?}")]
    InvalidRequest(A2aError),
    #[error("Method not found: {0:?}")]
    MethodNotFoundError(A2aError),
    #[error("Invalid params: {0:?}")]
    InvalidParamsError(A2aError),
    #[error("Internal error: {0:?}")]
    InternalError(A2aError),
    #[error("Task not found error: {0:?}")]
    TaskNotFoundError(A2aError),
    #[error("Task not cancelable error: {0:?}")]
    TaskNotCancelableError(A2aError),
    #[error("Push notification not supported error: {0:?}")]
    PushNotificationNotSupportedError(A2aError),
    #[error("Unsupported operation error: {0:?}")]
    UnsupportedOperationError(A2aError),
    #[error("Content type not supported error: {0:?}")]
    ContentTypeNotSupportedError(A2aError),
    #[error("Invalid agent response error: {0:?}")]
    InvalidAgentResponseError(A2aError),
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
    /// Returns the JSON-RPC error code for this error type
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

    /// Returns the error message
    pub fn message(&self) -> &str {
        match &self {
            A2aServerError::JsonParseError(v) => &v.message,
            A2aServerError::InvalidRequest(v) => &v.message,
            A2aServerError::MethodNotFoundError(v) => &v.message,
            A2aServerError::InvalidParamsError(v) => &v.message,
            A2aServerError::InternalError(v) => &v.message,
            A2aServerError::TaskNotFoundError(v) => &v.message,
            A2aServerError::TaskNotCancelableError(v) => &v.message,
            A2aServerError::PushNotificationNotSupportedError(v) => &v.message,
            A2aServerError::UnsupportedOperationError(v) => &v.message,
            A2aServerError::ContentTypeNotSupportedError(v) => &v.message,
            A2aServerError::InvalidAgentResponseError(v) => &v.message,
        }
    }

    /// Returns additional error data if present
    pub fn data(&self) -> Option<&serde_json::Value> {
        match &self {
            A2aServerError::JsonParseError(v) => v.data.as_ref(),
            A2aServerError::InvalidRequest(v) => v.data.as_ref(),
            A2aServerError::MethodNotFoundError(v) => v.data.as_ref(),
            A2aServerError::InvalidParamsError(v) => v.data.as_ref(),
            A2aServerError::InternalError(v) => v.data.as_ref(),
            A2aServerError::TaskNotFoundError(v) => v.data.as_ref(),
            A2aServerError::TaskNotCancelableError(v) => v.data.as_ref(),
            A2aServerError::PushNotificationNotSupportedError(v) => v.data.as_ref(),
            A2aServerError::UnsupportedOperationError(v) => v.data.as_ref(),
            A2aServerError::ContentTypeNotSupportedError(v) => v.data.as_ref(),
            A2aServerError::InvalidAgentResponseError(v) => v.data.as_ref(),
        }
    }
}

impl From<shared::error::CommonError> for A2aServerError {
    fn from(e: shared::error::CommonError) -> Self {
        A2aServerError::InternalError(A2aError::with_source(e.to_string(), Box::new(e)))
    }
}
