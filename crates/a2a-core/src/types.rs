#![allow(irrefutable_let_patterns)]

use axum::{Json, response::IntoResponse};
use serde::{Deserialize, Serialize};
use utoipa::{PartialSchema, ToSchema};

use crate::errors::A2aServerError;
include!(concat!(env!("OUT_DIR"), "/a2a_types.rs"));

pub type TaskId = String;
pub type ContextId = String;
// The code generator we are using, typify, to generate the rust types from JSON schema specification.
// Unfortunately it doesn't deal with "anyOf" or "oneOf" types very well and generates structs with Optional fields rather than
// enums with values. so for a few types, we need to manually define the enum to provide a better rust representation. The same applies for Errors
// (see errors.rs)
// pub enum TaskEvent {
//     Task(Task),
//     TaskStatusUpdate(TaskStatusUpdateEvent),
//     TaskArtifactUpdate(TaskArtifactUpdateEvent),
// }

// pub enum TaskIdParamsOrGetParams {
//     TaskId(TaskIdParams),
//     GetParams(GetTaskPushNotificationConfigParams),
// }

impl ToSchema for AgentCard {}

impl PartialSchema for AgentCard {
    // TODO: Implement schema generation for AgentCard
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::RefOr::T(utoipa::openapi::Schema::Object(
            utoipa::openapi::ObjectBuilder::new().build(),
        ))
    }
}

impl ToSchema for JsonrpcRequest {}

impl PartialSchema for JsonrpcRequest {
    // TODO: Implement schema generation for AgentCard
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::RefOr::T(utoipa::openapi::Schema::Object(
            utoipa::openapi::ObjectBuilder::new().build(),
        ))
    }
}

impl ToSchema for JsonrpcRequestId {}

impl PartialSchema for JsonrpcRequestId {
    // TODO: Implement schema generation for AgentCard
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::RefOr::T(utoipa::openapi::Schema::Object(
            utoipa::openapi::ObjectBuilder::new().build(),
        ))
    }
}

// We manually redefine a few types because the subtype_ generated fields would be
// problematic.
//
#[derive(Deserialize, Serialize, Clone, Debug, ToSchema)]
pub struct CustomJsonrpcError {
    ///A Number that indicates the error type that occurred.
    pub code: i32,
    /**A Primitive or Structured value that contains additional information about the error.
    This may be omitted.*/
    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
    pub data: ::std::option::Option<::serde_json::Value>,
    ///A String providing a short description of the error.
    pub message: ::std::string::String,
}

// #[derive(Deserialize, Serialize, Clone, Debug, ToSchema)]
// pub struct CustomJsonrpcErrorResponse {
//     pub error: CustomJsonrpcError,
//     /**An identifier established by the Client that MUST contain a String, Number.
//     Numbers SHOULD NOT contain fractional parts.*/
//     pub id: Option<JsonrpcRequestId>,
//     ///Specifies the version of the JSON-RPC protocol. MUST be exactly "2.0".
//     pub jsonrpc: ::std::string::String,
// }

// impl IntoResponse for CustomJsonrpcErrorResponse {
//     fn into_response(self) -> axum::response::Response {
//         (http::StatusCode::INTERNAL_SERVER_ERROR, Json(self)).into_response()
//     }
// }

impl From<A2aServerError> for CustomJsonrpcError {
    fn from(value: A2aServerError) -> Self {
        CustomJsonrpcError {
            code: value.json_rpc_code(),
            data: value.data(),
            message: value.message().to_string(),
        }
    }
}

#[derive(Serialize, Clone, Debug, ToSchema)]
pub enum CustomJsonRpcPayload<Data> {
    #[serde(rename = "error")]
    Err(CustomJsonrpcError),
    #[serde(rename = "result")]
    Ok(Data),
}

#[derive(Serialize, Clone, Debug, ToSchema)]
pub struct CustomJsonrpcResponse<Data> {
    /**An identifier established by the Client that MUST contain a String, Number.
    Numbers SHOULD NOT contain fractional parts.*/
    pub id: Option<JsonrpcRequestId>,
    ///Specifies the version of the JSON-RPC protocol. MUST be exactly "2.0".
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
