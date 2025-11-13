// use karyon_jsonrpc::error::RPCError;
use serde::Serialize;

use crate::{
    errors::{A2aServerError, ErrorBuilder},
    types::Task,
};

// impl From<A2aServerError> for RPCError {
//     fn from(value: A2aServerError) -> Self {
//         // TODO: Implement conversion from A2aServerError to tonic::Status
//         match value {
//             A2aServerError::InternalError(val) => {
//                 RPCError::CustomError(-32603, val.message.clone())
//             }
//             A2aServerError::JsonParseError(val) => {
//                 RPCError::CustomError(-32700, val.message.clone())
//             }
//             A2aServerError::InvalidRequest(val) => {
//                 RPCError::CustomError(-32600, val.message.clone())
//             }
//             A2aServerError::MethodNotFoundError(val) => {
//                 RPCError::CustomError(-32601, val.message.clone())
//             }
//             A2aServerError::InvalidParamsError(val) => {
//                 RPCError::CustomError(-32602, val.message.clone())
//             }
//             A2aServerError::TaskNotFoundError(val) => {
//                 RPCError::CustomError(-32001, val.message.clone())
//             }
//             A2aServerError::TaskNotCancelableError(val) => {
//                 RPCError::CustomError(-32002, val.message.clone())
//             }
//             A2aServerError::PushNotificationNotSupportedError(val) => {
//                 RPCError::CustomError(-32003, val.message.clone())
//             }
//             A2aServerError::UnsupportedOperationError(val) => {
//                 RPCError::CustomError(-32004, val.message.clone())
//             }
//             A2aServerError::ContentTypeNotSupportedError(val) => {
//                 RPCError::CustomError(-32005, val.message.clone())
//             }
//             A2aServerError::InvalidAgentResponseError(val) => {
//                 RPCError::CustomError(-32006, val.message.clone())
//             }
//         }
//     }
// }

pub struct JsonRpcResponse<AppResponseType>(Result<AppResponseType, A2aServerError>)
where
    AppResponseType: Serialize;

impl<AppResponseType: Serialize> JsonRpcResponse<AppResponseType> {
    pub fn new(result: Result<AppResponseType, A2aServerError>) -> Self {
        JsonRpcResponse(result)
    }
}

// impl<AppResponseType> From<JsonRpcResponse<AppResponseType>> for Result<Value, RPCError>
// where
//     AppResponseType: Serialize,
// {
//     fn from(value: JsonRpcResponse<AppResponseType>) -> Self {
//         match value.0 {
//             Ok(app_response) => Ok(serde_json::to_value(app_response).unwrap()),
//             Err(err) => Err(err.into()),
//         }
//     }
// }

pub fn map_optional_task_to_not_found(task: Option<Task>) -> Result<Task, A2aServerError> {
    match task {
        Some(task) => Ok(task),
        None => Err(A2aServerError::TaskNotFoundError(
            ErrorBuilder::default()
                .message("Failed to find task")
                .build()
                .unwrap(),
        )),
    }
}

// TODO: duplciated from shared so we can remove crate dependency

use std::fmt::Debug;

use axum::{
    Json,
    response::{IntoResponse, Response},
};
use http::StatusCode;
use tracing::error;
use utoipa::IntoResponses;

pub struct JsonResponse<T: Serialize, E: Serialize>(Result<T, E>);

impl<T: Serialize, E: Serialize + IntoResponse> JsonResponse<T, E> {
    pub fn new_error(error: E) -> Self {
        Self(Err(error))
    }

    pub fn new_ok(value: T) -> Self {
        Self(Ok(value))
    }
}

impl<T: Serialize, E: Serialize + IntoResponse> IntoResponses for JsonResponse<T, E> {
    fn responses() -> std::collections::BTreeMap<
        String,
        utoipa::openapi::RefOr<utoipa::openapi::response::Response>,
    > {
        // responses.insert("200".to_string(), utoipa::openapi);
        std::collections::BTreeMap::new()
    }
}

impl<T: Serialize, E: Serialize + IntoResponse + Debug> IntoResponse for JsonResponse<T, E> {
    fn into_response(self) -> Response {
        match self.0 {
            Ok(value) => (StatusCode::OK, Json(value)).into_response(),
            Err(error) => {
                error!("Error: {:?}", error);

                error.into_response()
            }
        }
    }
}

impl<T: Serialize, E: Serialize + IntoResponse> From<Result<T, E>> for JsonResponse<T, E> {
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(value) => JsonResponse::new_ok(value),
            Err(error) => JsonResponse::new_error(error),
        }
    }
}
