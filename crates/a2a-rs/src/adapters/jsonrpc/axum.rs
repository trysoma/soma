use crate::adapters::jsonrpc::utils::map_optional_task_to_not_found;
use crate::errors::A2aServerError;
use crate::service::{A2aServiceLike, RequestContext};
use crate::types::{
    AgentCard, CustomJsonRpcPayload, CustomJsonrpcResponse, JsonrpcRequest,
    SendStreamingMessageSuccessResponseResult,
};

use axum::response::IntoResponse;
use axum::{
    Json,
    extract::State,
    response::sse::{Event, Sse},
};
use http::{HeaderMap, Uri};
use shared::adapters::openapi::JsonResponse;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use tokio_stream::StreamExt as TokioStreamExt;
use tracing::info;
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn create_router<S: A2aServiceLike + Send + Sync + 'static>() -> OpenApiRouter<Arc<S>> {
    OpenApiRouter::new()
        .routes(routes!(json_rpc))
        .routes(routes!(agent_card))
        .routes(routes!(extended_agent_card))
}

macro_rules! require_request_context {
    ($uri:expr, $headers:expr) => {
        RequestContext {
            request_uri: $uri.clone(),
            headers: $headers.clone(),
        }
    };
}

#[utoipa::path(
    get,
    path = "/.well-known/agent.json",
    responses(
        (status = 200, description = "Successful response", body = AgentCard),
        (status = 500, description = "Internal Server Error", body = A2aServerError),
    )
)]
async fn agent_card<S: A2aServiceLike + Send + Sync + 'static>(
    State(ctx): State<Arc<S>>,
    uri: Uri,
    headers: HeaderMap,
) -> JsonResponse<AgentCard, A2aServerError> {
    info!("Received agent card request");
    let request_context = require_request_context!(uri, headers);
    let res = ctx.agent_card(request_context);
    JsonResponse::from(Ok(res.clone()))
}

#[utoipa::path(
    get,
    path = "/agent/authenticatedExtendedCard",
    responses(
        (status = 200, description = "Successful response", body = AgentCard),
        (status = 500, description = "Internal Server Error", body = A2aServerError),
    )
)]
async fn extended_agent_card<S: A2aServiceLike + Send + Sync + 'static>(
    State(ctx): State<Arc<S>>,
    uri: Uri,
    headers: HeaderMap,
) -> impl IntoResponse {
    let request_context = require_request_context!(uri, headers);
    let res = ctx.extended_agent_card(request_context);
    match res.as_ref() {
        Some(card) => (http::StatusCode::OK, Json(card.clone())).into_response(),
        None => (http::StatusCode::NOT_FOUND).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/",
    responses(
        (status = 200, description = "Successful response"),
        (status = 500, description = "Internal Server Error", body = A2aServerError),
    )
)]
async fn json_rpc<S: A2aServiceLike + Send + Sync + 'static>(
    State(ctx): State<Arc<S>>,
    uri: Uri,
    headers: HeaderMap,
    Json(body): Json<JsonrpcRequest>,
) -> impl IntoResponse {
    let request_context = require_request_context!(uri, headers);
    let id = body.id.clone();
    info!("Received JSON-RPC request");

    macro_rules! respond {
        ($expr:expr) => {{
            let data = $expr.into();
            let res = CustomJsonrpcResponse::new(id.clone(), data);
            res.into_response()
        }};
    }

    match body.method.as_str() {
        "tasks/get" => {
            info!("Received tasks/get request");
            let params = serde_json::from_value(serde_json::Value::Object(body.params)).unwrap();
            respond!(
                ctx.request_handler(request_context)
                    .on_get_task(params)
                    .await
                    .and_then(map_optional_task_to_not_found)
            )
        }
        "tasks/cancel" => {
            info!("Received tasks/cancel request");
            let params = serde_json::from_value(serde_json::Value::Object(body.params)).unwrap();
            respond!(
                ctx.request_handler(request_context)
                    .on_cancel_task(params)
                    .await
                    .and_then(map_optional_task_to_not_found)
            )
        }
        "message/send" => {
            info!("Received message/send request");
            let params = serde_json::from_value(serde_json::Value::Object(body.params)).unwrap();
            respond!(
                ctx.request_handler(request_context)
                    .on_message_send(params)
                    .await
            )
        }
        "tasks/pushNotificationConfig/get" => {
            info!("Received tasks/pushNotificationConfig/get request");
            let params = serde_json::from_value(serde_json::Value::Object(body.params)).unwrap();
            respond!(
                ctx.request_handler(request_context)
                    .on_get_task_push_notification_config(params)
                    .await
            )
        }
        "tasks/pushNotificationConfig/list" => {
            info!("Received tasks/pushNotificationConfig/list request");
            let params = serde_json::from_value(serde_json::Value::Object(body.params)).unwrap();
            respond!(
                ctx.request_handler(request_context)
                    .on_list_task_push_notification_config(params)
                    .await
            )
        }
        "tasks/pushNotificationConfig/delete" => {
            info!("Received tasks/pushNotificationConfig/delete request");
            let params = serde_json::from_value(serde_json::Value::Object(body.params)).unwrap();
            respond!(
                ctx.request_handler(request_context)
                    .on_delete_task_push_notification_config(params)
                    .await
            )
        }
        "message/stream" => {
            info!("Received message/stream request");
            let params = serde_json::from_value(serde_json::Value::Object(body.params)).unwrap();
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            let id_for_task = id.clone();

            tokio::spawn(async move {
                let handler = ctx
                    .request_handler(request_context);
                let stream_res = handler
                    .on_message_send_stream(params).await;

                match stream_res {
                    Ok(mut stream) => {
                        while let Some(item) = stream.next().await {
                            info!("Sending message stream item 1");
                            if tx.send(item).is_err() {
                                break;
                            }
                        }
                    }
                    Err(err) => {
                        let _ = tx.send(Err(err));
                    }
                }
            });

            let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
            let stream = TokioStreamExt::map(stream, move |item| {
                info!("Sending message stream item 2");

                let data: CustomJsonRpcPayload<SendStreamingMessageSuccessResponseResult> =
                    item.into();
                let res = CustomJsonrpcResponse::new(id_for_task.clone(), data);
                info!(
                    "Sending message stream item {:?}",
                    serde_json::to_string(&res).unwrap()
                );
                Event::default().json_data(res)
            });

            Sse::new(stream)
                .keep_alive(
                    axum::response::sse::KeepAlive::new()
                        .interval(Duration::from_secs(1))
                        .text("keep-alive"),
                )
                .into_response()
        }
        "tasks/resubscribe" => {
            info!("Received tasks/resubscribe request");
            let params = serde_json::from_value(serde_json::Value::Object(body.params)).unwrap();
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            let id_for_task = id.clone();

            tokio::spawn(async move {
                let handler = ctx
                    .request_handler(request_context);
                let stream_res = handler
                    .on_message_send_stream(params).await;

                match stream_res {
                    Ok(mut stream) => {
                        while let Some(item) = stream.next().await {
                            if tx.send(item).is_err() {
                                break;
                            }
                        }
                    }
                    Err(err) => {
                        let _ = tx.send(Err(err));
                    }
                }
            });

            let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
            let stream = TokioStreamExt::map(stream, move |item| {
                let data: CustomJsonRpcPayload<SendStreamingMessageSuccessResponseResult> =
                    item.into();
                let res = CustomJsonrpcResponse::new(id_for_task.clone(), data);
                Event::default().json_data(res)
            });

            Sse::new(stream)
                .keep_alive(
                    axum::response::sse::KeepAlive::new()
                        .interval(Duration::from_secs(1))
                        .text("keep-alive"),
                )
                .into_response()
        }
        _ => {
            info!("Received unknown request");
            (http::StatusCode::NOT_FOUND).into_response()
        }
    }
}
