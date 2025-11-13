use crate::adapters::mcp::McpErrorMsg;
use a2a_rs::errors::A2aServerError;
use axum::{
    Json,
    response::{IntoResponse, Response},
};
use http::StatusCode;
use rmcp::ErrorData;
use serde::Serialize;
use thiserror::Error;
use utoipa::{IntoResponses, PartialSchema, ToSchema};

pub type DynError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Error, Debug, Serialize)]
pub enum CommonError {
    #[error("user is not authenticated to perform this action.")]
    Authentication {
        msg: String,
        #[serde(skip)]
        #[source]
        source: Option<anyhow::Error>,
    },
    #[error("user is not authorized to perform this action.")]
    Authorization {
        msg: String,
        #[serde(skip)]
        #[source]
        source: anyhow::Error,
    },
    #[error("could not find resource")]
    NotFound {
        msg: String,
        lookup_id: String,
        #[serde(skip)]
        #[source]
        source: Option<anyhow::Error>,
    },
    #[error("unknown error")]
    Unknown(
        #[serde(skip)]
        #[from]
        anyhow::Error,
    ),
    #[error("invalid request")]
    InvalidRequest {
        msg: String,
        #[serde(skip)]
        #[source]
        source: Option<anyhow::Error>,
    },
    #[error("invalid response")]
    InvalidResponse {
        msg: String,
        #[serde(skip)]
        #[source]
        source: Option<anyhow::Error>,
    },
    #[error("repository error")]
    Repository {
        msg: String,
        #[serde(skip)]
        #[source]
        source: Option<anyhow::Error>,
    },
    #[error("sqlite database error")]
    SqliteError {
        #[serde(skip)]
        #[from]
        #[source]
        source: libsql::Error,
    },
    #[error("tokio channel error")]
    TokioChannelError {
        #[serde(skip)]
        #[source]
        source: DynError,
    },
    #[error("io error")]
    IoError {
        #[serde(skip)]
        #[from]
        #[source]
        source: std::io::Error,
    },
    #[error("url parse error")]
    UrlParseError {
        #[serde(skip)]
        #[from]
        #[source]
        source: url::ParseError,
    },
    #[error("serde json error")]
    SerdeSerializationError {
        #[serde(skip)]
        #[from]
        #[source]
        source: serde_json::Error,
    },
    #[error("axum error")]
    AxumError {
        #[serde(skip)]
        #[from]
        #[source]
        source: axum::Error,
    },

    #[error("address parse error")]
    AddrParseError {
        #[serde(skip)]
        #[from]
        #[source]
        source: std::net::AddrParseError,
    },
    #[error("libsql migration error")]
    LibsqlMigrationError {
        #[serde(skip)]
        #[from]
        #[source]
        source: libsql_migration::errors::LibsqlDirMigratorError,
    },
    #[error("var error")]
    VarError {
        #[serde(skip)]
        #[from]
        #[source]
        source: std::env::VarError,
    },
    #[error("glob set error")]
    GlobSetError {
        #[serde(skip)]
        #[from]
        #[source]
        source: globset::Error,
    },
    #[error("notify error")]
    NotifyError {
        #[serde(skip)]
        #[from]
        #[source]
        source: notify::Error,
    },
    #[error("reqwest error")]
    ReqwestError {
        #[serde(skip)]
        #[from]
        #[source]
        source: reqwest::Error,
    },
}

impl From<CommonError> for A2aServerError {
    fn from(e: CommonError) -> Self {
        A2aServerError::InternalError(a2a_rs::errors::Error {
            message: e.to_string(),
            data: None,
            source: Some(Box::new(e)),
        })
    }
}

impl<T: Send + Sync + 'static> From<tokio::sync::mpsc::error::SendError<T>> for CommonError {
    fn from(e: tokio::sync::mpsc::error::SendError<T>) -> Self {
        CommonError::TokioChannelError {
            source: Box::new(e),
        }
    }
}

impl From<tokio::sync::oneshot::error::RecvError> for CommonError {
    fn from(e: tokio::sync::oneshot::error::RecvError) -> Self {
        CommonError::TokioChannelError {
            source: Box::new(e),
        }
    }
}

impl<T: Send + Sync + 'static + std::fmt::Debug> From<tokio::sync::broadcast::error::SendError<T>>
    for CommonError
{
    fn from(e: tokio::sync::broadcast::error::SendError<T>) -> Self {
        CommonError::TokioChannelError {
            source: Box::new(e),
        }
    }
}

impl From<tokio::sync::broadcast::error::RecvError> for CommonError {
    fn from(e: tokio::sync::broadcast::error::RecvError) -> Self {
        CommonError::TokioChannelError {
            source: Box::new(e),
        }
    }
}

impl From<rustls::Error> for CommonError {
    fn from(err: rustls::Error) -> Self {
        CommonError::InvalidRequest {
            msg: "TLS error".to_string(),
            source: Some(anyhow::Error::from(err)),
        }
    }
}

impl ToSchema for CommonError {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("Error")
    }

    fn schemas(
        _schemas: &mut Vec<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>,
    ) {
        // nothing by default
    }
}

impl PartialSchema for CommonError {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .property(
                "name",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::String),
            )
            .required("name")
            .property(
                "message",
                utoipa::openapi::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::String),
            )
            .required("message")
            .into()
    }
}

impl IntoResponses for CommonError {
    fn responses() -> std::collections::BTreeMap<
        String,
        utoipa::openapi::RefOr<utoipa::openapi::response::Response>,
    > {
        let mut responses = std::collections::BTreeMap::new();

        let error_content = utoipa::openapi::ContentBuilder::new()
            .schema(Some(CommonError::schema()))
            .build();

        // Authentication Error - 401
        responses.insert(
            "401".to_string(),
            utoipa::openapi::ResponseBuilder::new()
                .description("Authentication error")
                .content("application/json", error_content.clone())
                .into(),
        );

        // Authorization Error - 403
        responses.insert(
            "403".to_string(),
            utoipa::openapi::ResponseBuilder::new()
                .description("Authorization error")
                .content("application/json", error_content.clone())
                .into(),
        );

        // Not Found Error - 404
        responses.insert(
            "404".to_string(),
            utoipa::openapi::ResponseBuilder::new()
                .description("Resource not found")
                .content("application/json", error_content.clone())
                .into(),
        );

        // Invalid Request - 400
        responses.insert(
            "400".to_string(),
            utoipa::openapi::ResponseBuilder::new()
                .description("Invalid request")
                .content("application/json", error_content.clone())
                .into(),
        );

        // Invalid Response - 500
        responses.insert(
            "500".to_string(),
            utoipa::openapi::ResponseBuilder::new()
                .description("Server error")
                .content("application/json", error_content)
                .into(),
        );

        responses
    }
}

impl IntoResponse for CommonError {
    fn into_response(self) -> Response {
        let status = match self {
            CommonError::Authentication { .. } => StatusCode::UNAUTHORIZED,
            CommonError::Authorization { .. } => StatusCode::FORBIDDEN,
            CommonError::NotFound { .. } => StatusCode::NOT_FOUND,
            CommonError::InvalidRequest { .. } => StatusCode::BAD_REQUEST,
            CommonError::InvalidResponse { .. }
            | CommonError::Unknown(_)
            | CommonError::Repository { .. }
            | CommonError::SqliteError { .. }
            | CommonError::TokioChannelError { .. }
            | CommonError::IoError { .. }
            | CommonError::SerdeSerializationError { .. }
            | CommonError::UrlParseError { .. }
            | CommonError::AxumError { .. }
            | CommonError::LibsqlMigrationError { .. }
            | CommonError::VarError { .. }
            | CommonError::GlobSetError { .. }
            | CommonError::NotifyError { .. }
            | CommonError::ReqwestError { .. }
            | CommonError::AddrParseError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(ErrorResponse {
            name: match self {
                CommonError::Authentication { .. } => "Authentication",
                CommonError::Authorization { .. } => "Authorization",
                CommonError::NotFound { .. } => "NotFound",
                CommonError::InvalidRequest { .. } => "InvalidRequest",
                CommonError::InvalidResponse { .. } => "InvalidResponse",
                CommonError::Repository { .. } => "Repository",
                CommonError::SqliteError { .. } => "InternalServerError",
                CommonError::Unknown(_) => "InternalServerError",
                CommonError::TokioChannelError { .. } => "InternalServerError",
                CommonError::IoError { .. } => "InternalServerError",
                CommonError::SerdeSerializationError { .. } => "InternalServerError",
                CommonError::UrlParseError { .. } => "InternalServerError",
                CommonError::AxumError { .. } => "InternalServerError",
                CommonError::AddrParseError { .. } => "InternalServerError",
                CommonError::LibsqlMigrationError { .. } => "InternalServerError",
                CommonError::VarError { .. } => "InternalServerError",
                CommonError::GlobSetError { .. } => "InternalServerError",
                CommonError::NotifyError { .. } => "InternalServerError",
                CommonError::ReqwestError { .. } => "InternalServerError",
            }
            .to_string(),
            message: self.to_string(),
        });

        (status, body).into_response()
    }
}

#[derive(Serialize)]
pub struct ErrorResponse {
    name: String,
    message: String,
}

impl From<CommonError> for ErrorData {
    fn from(error: CommonError) -> ErrorData {
        match error {
            CommonError::NotFound {
                msg,
                lookup_id: _,
                source: _,
            } => ErrorData::resource_not_found(msg, None),
            CommonError::InvalidRequest { msg, source: _ } => ErrorData::invalid_request(msg, None),
            CommonError::Authentication { .. }
            | CommonError::Authorization { .. }
            | CommonError::InvalidResponse { .. }
            | CommonError::Unknown(_)
            | CommonError::Repository { .. }
            | CommonError::SqliteError { .. }
            | CommonError::TokioChannelError { .. }
            | CommonError::IoError { .. }
            | CommonError::SerdeSerializationError { .. }
            | CommonError::AxumError { .. }
            | CommonError::UrlParseError { .. }
            | CommonError::AddrParseError { .. }
            | CommonError::LibsqlMigrationError { .. }
            | CommonError::VarError { .. }
            | CommonError::GlobSetError { .. }
            | CommonError::NotifyError { .. }
            | CommonError::ReqwestError { .. } => {
                ErrorData::internal_error(error.to_string(), None)
            }
        }
    }
}

impl McpErrorMsg for CommonError {
    fn to_mcp_error(&self) -> String {
        self.to_string()
    }
}
