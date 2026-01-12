//! A2A Protocol Types
//!
//! This module contains the generated types from the A2A JSON schema specification,
//! plus custom wrapper types for better Rust ergonomics.

#![allow(irrefutable_let_patterns)]

use axum::{Json, response::IntoResponse};
use serde::{Deserialize, Serialize};
use utoipa::{PartialSchema, ToSchema};

use super::errors::A2aServerError;

// Include the generated types from build.rs
include!(concat!(env!("OUT_DIR"), "/a2a_types.rs"));

pub type TaskId = String;
pub type ContextId = String;

impl ToSchema for AgentCard {}

impl PartialSchema for AgentCard {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::RefOr::T(utoipa::openapi::Schema::Object(
            utoipa::openapi::ObjectBuilder::new().build(),
        ))
    }
}

impl ToSchema for JsonrpcRequest {}

impl PartialSchema for JsonrpcRequest {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::RefOr::T(utoipa::openapi::Schema::Object(
            utoipa::openapi::ObjectBuilder::new().build(),
        ))
    }
}

impl ToSchema for JsonrpcRequestId {}

impl PartialSchema for JsonrpcRequestId {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::RefOr::T(utoipa::openapi::Schema::Object(
            utoipa::openapi::ObjectBuilder::new().build(),
        ))
    }
}

/// Custom JSON-RPC error type with better serialization
#[derive(Deserialize, Serialize, Clone, Debug, ToSchema)]
pub struct CustomJsonrpcError {
    /// A Number that indicates the error type that occurred.
    pub code: i32,
    /// A Primitive or Structured value that contains additional information about the error.
    /// This may be omitted.
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub data: ::std::option::Option<::serde_json::Value>,
    /// A String providing a short description of the error.
    pub message: ::std::string::String,
}

impl From<A2aServerError> for CustomJsonrpcError {
    fn from(value: A2aServerError) -> Self {
        CustomJsonrpcError {
            code: value.json_rpc_code(),
            data: value.data().cloned(),
            message: value.message().to_string(),
        }
    }
}

/// Custom JSON-RPC payload that can be either success or error
#[derive(Serialize, Clone, Debug, ToSchema)]
pub enum CustomJsonRpcPayload<Data> {
    #[serde(rename = "error")]
    Err(CustomJsonrpcError),
    #[serde(rename = "result")]
    Ok(Data),
}

/// Custom JSON-RPC response wrapper
#[derive(Serialize, Clone, Debug, ToSchema)]
pub struct CustomJsonrpcResponse<Data> {
    /// An identifier established by the Client that MUST contain a String, Number.
    /// Numbers SHOULD NOT contain fractional parts.
    pub id: Option<JsonrpcRequestId>,
    /// Specifies the version of the JSON-RPC protocol. MUST be exactly "2.0".
    pub jsonrpc: ::std::string::String,
    #[serde(flatten)]
    pub data: CustomJsonRpcPayload<Data>,
}

impl<Data> CustomJsonrpcResponse<Data> {
    pub fn new_err(id: Option<JsonrpcRequestId>, error: CustomJsonrpcError) -> Self {
        CustomJsonrpcResponse {
            id,
            jsonrpc: "2.0".to_string(),
            data: CustomJsonRpcPayload::Err(error),
        }
    }

    pub fn new_ok(id: Option<JsonrpcRequestId>, data: Data) -> Self {
        CustomJsonrpcResponse {
            id,
            jsonrpc: "2.0".to_string(),
            data: CustomJsonRpcPayload::Ok(data),
        }
    }

    pub fn new(id: Option<JsonrpcRequestId>, data: CustomJsonRpcPayload<Data>) -> Self {
        CustomJsonrpcResponse {
            id,
            jsonrpc: "2.0".to_string(),
            data,
        }
    }
}

impl<Result: Serialize> IntoResponse for CustomJsonrpcResponse<Result> {
    fn into_response(self) -> axum::response::Response {
        (http::StatusCode::OK, Json(self)).into_response()
    }
}

impl<Data> From<Result<Data, A2aServerError>> for CustomJsonRpcPayload<Data> {
    fn from(result: Result<Data, A2aServerError>) -> Self {
        match result {
            Ok(data) => CustomJsonRpcPayload::Ok(data),
            Err(err) => CustomJsonRpcPayload::Err(err.into()),
        }
    }
}

/// Convert an Event result to a CustomJsonRpcPayload for SSE streaming
impl From<Result<super::events::Event, A2aServerError>>
    for CustomJsonRpcPayload<SendStreamingMessageSuccessResponseResult>
{
    fn from(result: Result<super::events::Event, A2aServerError>) -> Self {
        match result {
            Ok(event) => {
                let result: SendStreamingMessageSuccessResponseResult = match event {
                    super::events::Event::Message(msg) => {
                        SendStreamingMessageSuccessResponseResult::Message(msg)
                    }
                    super::events::Event::Task(task) => {
                        SendStreamingMessageSuccessResponseResult::Task(task)
                    }
                    super::events::Event::TaskStatusUpdate(update) => {
                        SendStreamingMessageSuccessResponseResult::TaskStatusUpdateEvent(update)
                    }
                    super::events::Event::TaskArtifactUpdate(update) => {
                        SendStreamingMessageSuccessResponseResult::TaskArtifactUpdateEvent(update)
                    }
                };
                CustomJsonRpcPayload::Ok(result)
            }
            Err(err) => CustomJsonRpcPayload::Err(err.into()),
        }
    }
}
