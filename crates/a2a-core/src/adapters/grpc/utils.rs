use std::{marker::PhantomData, pin::Pin};

use crate::{
    errors::{A2aServerError, ErrorBuilder},
    types::Task,
};
use futures::StreamExt;
use futures::{Stream, TryStreamExt};

impl From<A2aServerError> for tonic::Status {
    fn from(value: A2aServerError) -> Self {
        // TODO: Implement conversion from A2aServerError to tonic::Status
        tonic::Status::internal(value.to_string())
    }
}

// TODO: resolve this implementationt oo.
impl From<tonic::Status> for A2aServerError {
    fn from(value: tonic::Status) -> Self {
        A2aServerError::InternalError(
            ErrorBuilder::default()
                .message(format!("tonic failed: {}", value.code()))
                .build()
                .unwrap(),
        )
    }
}

pub struct GrpcResponse<InnerType, AppResponseType>(
    Result<AppResponseType, A2aServerError>,
    PhantomData<InnerType>,
)
where
    AppResponseType: Into<InnerType>;

impl<InnerType, AppResponseType: Into<InnerType>> GrpcResponse<InnerType, AppResponseType> {
    pub fn new(result: Result<AppResponseType, A2aServerError>) -> Self {
        GrpcResponse(result, PhantomData)
    }
}

impl<InnerType, AppResponseType> From<GrpcResponse<InnerType, AppResponseType>>
    for Result<tonic::Response<InnerType>, tonic::Status>
where
    AppResponseType: Into<InnerType>,
{
    fn from(value: GrpcResponse<InnerType, AppResponseType>) -> Self {
        match value.0 {
            Ok(app_response) => Ok(tonic::Response::new(app_response.into())),
            Err(err) => Err(err.into()),
        }
    }
}

#[allow(dead_code)]
pub type BoxedStream<T> = Pin<Box<dyn Stream<Item = Result<T, A2aServerError>> + Send>>;

#[allow(dead_code)]
pub fn convert_stream_to_internal<Pb, Domain>(stream: tonic::Streaming<Pb>) -> BoxedStream<Domain>
where
    Domain: TryFrom<Pb, Error = A2aServerError> + Send + 'static,
    Pb: Send + 'static,
{
    Box::pin(
        stream
            .map_err(A2aServerError::from) // tonic::Status -> CommonError
            .and_then(|pb| async move {
                Domain::try_from(pb) // TryFrom<Pb> -> Domain
            }),
    )
}

#[allow(dead_code)]
pub fn convert_stream_to_grpc<Pb, Domain>(
    stream: impl Stream<Item = Result<Domain, A2aServerError>> + Send + 'static,
) -> Pin<Box<dyn Stream<Item = Result<Pb, tonic::Status>> + Send + 'static>>
where
    Domain: Into<Pb> + 'static,
{
    Box::pin(stream.map(|res| res.map(Into::into).map_err(tonic::Status::from)))
}

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

/// Spawns a non-'static stream into a `'static` gRPC stream response using mpsc and tokio::spawn.
#[macro_export]
macro_rules! spawn_stream_to_grpc {
    ($stream_expr:expr, $buffer:expr) => {{
        let (tx, rx) = ::tokio::sync::mpsc::channel($buffer);

        ::tokio::spawn(async move {
            let mut stream = $stream_expr;
            while let Some(item) = stream.next().await {
                if tx.send(item).await.is_err() {
                    break;
                }
            }
        });

        ::tokio_stream::wrappers::ReceiverStream::new(rx)
    }};
}
