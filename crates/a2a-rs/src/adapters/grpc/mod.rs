mod impl_artifact;
mod impl_events;
mod impl_message;
mod impl_params;
mod impl_task;

mod utils;

use futures::{Stream, StreamExt};
use http::Request;
use hyper::body::Incoming;
use std::pin::Pin;
use std::sync::Arc;
use tower::ServiceBuilder;

use crate::adapters::grpc::utils::convert_stream_to_grpc;
use crate::adapters::grpc::utils::{GrpcResponse, map_optional_task_to_not_found};
use crate::service::{A2aServiceLike, RequestContext};
use crate::spawn_stream_to_grpc;

#[allow(clippy::all)]
pub mod proto {
    pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/service.bin"));

    tonic::include_proto!("a2a.v1");
}

type SendStreamingMessageStream =
    Pin<Box<dyn Stream<Item = Result<proto::StreamResponse, tonic::Status>> + Send + 'static>>;

pub struct GrpcService {
    service: Arc<dyn A2aServiceLike + Send + Sync>,
}

macro_rules! require_request_context {
    ($request:expr) => {
        match $request.extensions().get::<RequestContext>() {
            Some(ctx) => ctx.clone(),
            None => panic!("RequestContext not found, ensure"),
        }
    };
}

#[tonic::async_trait]
impl proto::a2a_service_server::A2aService for GrpcService {
    type SendStreamingMessageStream = SendStreamingMessageStream;
    type TaskSubscriptionStream = SendStreamingMessageStream;

    /// Send a message to the agent. This is a blocking call that will return the
    /// task once it is completed, or a LRO if requested.
    async fn send_message(
        self: std::sync::Arc<Self>,
        request: tonic::Request<proto::SendMessageRequest>,
    ) -> Result<tonic::Response<proto::SendMessageResponse>, tonic::Status> {
        let request_context = require_request_context!(request);
        GrpcResponse::new(
            self.service
                .request_handler(request_context)
                .on_message_send(request.into_inner().into())
                .await,
        )
        .into()
    }

    /// SendStreamingMessage is a streaming call that will return a stream of
    /// task update events until the Task is in an interrupted or terminal state.
    async fn send_streaming_message(
        self: std::sync::Arc<Self>,
        request: tonic::Request<proto::SendMessageRequest>,
    ) -> Result<tonic::Response<Self::SendStreamingMessageStream>, tonic::Status> {
        let request_context = require_request_context!(request);
        let params = request.into_inner().into();

        let result_stream = self
            .service
            .request_handler(request_context)
            .on_message_send_stream(params)
            .await
            .unwrap();
        let stream = spawn_stream_to_grpc!(result_stream, 32);
        let mapped_stream: SendStreamingMessageStream = convert_stream_to_grpc(stream);

        Ok(tonic::Response::new(mapped_stream))
    }
    /// Get the current state of a task from the agent.
    async fn get_task(
        self: std::sync::Arc<Self>,
        request: tonic::Request<proto::GetTaskRequest>,
    ) -> Result<tonic::Response<proto::Task>, tonic::Status> {
        let request_context = require_request_context!(request);
        let res = self
            .service
            .request_handler(request_context)
            .on_get_task(request.into_inner().into())
            .await
            .and_then(map_optional_task_to_not_found);

        GrpcResponse::new(res).into()
    }
    /// Cancel a task from the agent. If supported one should expect no
    /// more task updates for the task.
    async fn cancel_task(
        self: std::sync::Arc<Self>,
        request: tonic::Request<proto::CancelTaskRequest>,
    ) -> Result<tonic::Response<proto::Task>, tonic::Status> {
        let request_context = require_request_context!(request);
        let res = self
            .service
            .request_handler(request_context)
            .on_cancel_task(request.into_inner().into())
            .await
            .and_then(map_optional_task_to_not_found);
        GrpcResponse::new(res).into()
    }

    /// TaskSubscription is a streaming call that will return a stream of task
    /// update events. This attaches the stream to an existing in process task.
    /// If the task is complete the stream will return the completed task (like
    /// GetTask) and close the stream.
    async fn task_subscription(
        self: std::sync::Arc<Self>,
        request: tonic::Request<proto::TaskSubscriptionRequest>,
    ) -> Result<tonic::Response<Self::TaskSubscriptionStream>, tonic::Status> {
        let request_context = require_request_context!(request);
        let handler = self.service.request_handler(request_context).clone();
        let params = request.into_inner().into();

        let stream = spawn_stream_to_grpc!(handler.on_resubscribe_to_task(params).unwrap(), 32);
        let mapped_stream: SendStreamingMessageStream = convert_stream_to_grpc(stream);

        Ok(tonic::Response::new(mapped_stream))
    }
    /// Set a push notification config for a task.
    async fn create_task_push_notification_config(
        self: std::sync::Arc<Self>,
        request: tonic::Request<proto::CreateTaskPushNotificationConfigRequest>,
    ) -> Result<tonic::Response<proto::TaskPushNotificationConfig>, tonic::Status> {
        let request_context = require_request_context!(request);
        GrpcResponse::new(
            self.service
                .request_handler(request_context)
                .on_set_task_push_notification_config(request.into_inner().into())
                .await,
        )
        .into()
    }
    /// Get a push notification config for a task.
    async fn get_task_push_notification_config(
        self: std::sync::Arc<Self>,
        request: tonic::Request<proto::GetTaskPushNotificationConfigRequest>,
    ) -> Result<tonic::Response<proto::TaskPushNotificationConfig>, tonic::Status> {
        let request_context = require_request_context!(request);
        GrpcResponse::new(
            self.service
                .request_handler(request_context)
                .on_get_task_push_notification_config(request.into_inner().into())
                .await,
        )
        .into()
    }
    /// Get a list of push notifications configured for a task.
    async fn list_task_push_notification_config(
        self: std::sync::Arc<Self>,
        request: tonic::Request<proto::ListTaskPushNotificationConfigRequest>,
    ) -> Result<tonic::Response<proto::ListTaskPushNotificationConfigResponse>, tonic::Status> {
        let request_context = require_request_context!(request);
        GrpcResponse::new(
            self.service
                .request_handler(request_context)
                .on_list_task_push_notification_config(request.into_inner().into())
                .await,
        )
        .into()
    }
    /// GetAgentCard returns the agent card for the agent.
    async fn get_agent_card(
        self: std::sync::Arc<Self>,
        request: tonic::Request<proto::GetAgentCardRequest>,
    ) -> Result<tonic::Response<proto::AgentCard>, tonic::Status> {
        let request_context = require_request_context!(request);
        let card = self
            .service
            .agent_card(request_context)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        Ok(tonic::Response::new(card.into()))
    }
    /// Delete a push notification config for a task.
    async fn delete_task_push_notification_config(
        self: std::sync::Arc<Self>,
        request: tonic::Request<proto::DeleteTaskPushNotificationConfigRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let request_context = require_request_context!(request);
        GrpcResponse::new(
            self.service
                .request_handler(request_context)
                .on_delete_task_push_notification_config(request.into_inner().into())
                .await,
        )
        .into()
    }
}

pub fn build_request_context_layer()
-> impl tower::layer::Layer<tower::util::Either<Request<Incoming>, Request<Incoming>>> {
    ServiceBuilder::new().map_request(|mut req: Request<Incoming>| {
        // clone what you need from the raw HTTP request
        let ctx = RequestContext {
            request_uri: req.uri().clone(),
            headers: req.headers().clone(),
        };
        // stash it into request extensions so tonic can carry it into your gRPC handler
        req.extensions_mut().insert(ctx);
        req
    })
}
