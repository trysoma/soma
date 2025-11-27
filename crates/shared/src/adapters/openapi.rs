use std::fmt::Debug;

use axum::{
    Json,
    response::{IntoResponse, Redirect, Response},
};
use http::StatusCode;
use serde::Serialize;
use tracing::error;
use utoipa::IntoResponses;

pub const API_VERSION_TAG: &str = "v1";

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

pub struct RedirectResponse<E: Serialize + ToOwned + IntoResponse>(Result<Redirect, E>);

impl<E: Serialize + ToOwned + IntoResponse> RedirectResponse<E> {
    pub fn new_error(error: E) -> Self {
        Self(Err(error))
    }

    pub fn new_ok(redirect: Redirect) -> Self {
        Self(Ok(redirect))
    }
}

impl<E: Serialize + ToOwned + IntoResponse + Debug> IntoResponse for RedirectResponse<E> {
    fn into_response(self) -> Response {
        match self.0 {
            Ok(redirect) => redirect.into_response(),
            Err(error) => {
                error!("Error: {:?}", error);

                error.into_response()
            }
        }
    }
}

impl<E: Serialize + ToOwned + IntoResponse + Debug> From<Result<Redirect, E>>
    for RedirectResponse<E>
{
    fn from(result: Result<Redirect, E>) -> Self {
        match result {
            Ok(redirect) => RedirectResponse::new_ok(redirect),
            Err(error) => RedirectResponse::new_error(error),
        }
    }
}
