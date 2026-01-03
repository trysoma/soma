// use std::sync::Arc;

// use crate::adapters::jsonrpc::utils::{JsonRpcResponse, map_optional_task_to_not_found};
// use crate::service::{A2aServiceLike, RequestContext};
// use karyon_jsonrpc::{
//     error::RPCError, rpc_impl, rpc_method, rpc_pubsub_impl,
//     server::Channel,
// };
// use serde_json::{Value, json};

// // impl From<A2aServerError> for ErrorObject<'static> {
// //     fn from(err: A2aServerError) -> Self {
// //         ErrorObject::owned(err.json_rpc_code(), err.message(), err.data())
// //     }
// // }

// pub struct JsonRpcService {
//     service: Arc<dyn A2aServiceLike + Send + Sync>,
// }

// #[rpc_impl]
// impl JsonRpcService {
//     #[rpc_method(name = "tasks/get")]
//     async fn rpc_on_get_task(
//         &self,
//         request_context: RequestContext,
//         _method: String,
//         params: Value,
//     ) -> Result<Value, RPCError> {
//         let params = serde_json::from_value(params)?;
//         let res = self
//             .service
//             .request_handler(request_context)
//             .on_get_task(params)
//             .await
//             .and_then(map_optional_task_to_not_found);
//         let rpc_res = JsonRpcResponse::new(res);
//         rpc_res.into()
//     }

//     #[rpc_method(name = "tasks/cancel")]
//     async fn rpc_on_cancel_task(
//         &self,
//         request_context: RequestContext,
//         _method: String,
//         params: Value,
//     ) -> Result<Value, RPCError> {
//         let params = serde_json::from_value(params)?;
//         let res = self
//             .service
//             .request_handler(request_context)
//             .on_cancel_task(params)
//             .await
//             .and_then(map_optional_task_to_not_found);
//         let rpc_res = JsonRpcResponse::new(res);
//         rpc_res.into()
//     }

//     #[rpc_method(name = "message/send")]
//     async fn rpc_on_message_send(
//         &self,
//         request_context: RequestContext,
//         _method: String,
//         params: Value,
//     ) -> Result<Value, RPCError> {
//         let params = serde_json::from_value(params)?;
//         let res = self.service.request_handler(request_context).on_message_send(params).await;
//         let rpc_res = JsonRpcResponse::new(res);
//         rpc_res.into()
//     }

//     #[rpc_method(name = "tasks/pushNotificationConfig/set")]
//     async fn rpc_on_set_task_push_notification_config(
//         &self,
//         request_context: RequestContext,
//         params: Value,
//     ) -> Result<Value, RPCError> {
//         let params = serde_json::from_value(params)?;
//         let res = self
//             .service
//             .request_handler(request_context)
//             .on_set_task_push_notification_config(params)
//             .await;
//         let rpc_res = JsonRpcResponse::new(res);
//         rpc_res.into()
//     }

//     #[rpc_method(name = "tasks/pushNotificationConfig/delete")]
//     async fn rpc_on_delete_task_push_notification_config(
//         &self,
//         request_context: RequestContext,
//         params: Value,
//     ) -> Result<Value, RPCError> {
//         let params = serde_json::from_value(params)?;
//         let res = self
//             .service
//             .request_handler(request_context)
//             .on_delete_task_push_notification_config(params)
//             .await;
//         let rpc_res = JsonRpcResponse::new(res);
//         rpc_res.into()
//     }

//     #[rpc_method(name = "tasks/pushNotificationConfig/list")]
//     async fn rpc_on_list_task_push_notification_config(
//         &self,
//         request_context: RequestContext,
//         params: Value,
//     ) -> Result<Value, RPCError> {
//         let params = serde_json::from_value(params)?;
//         let res = self
//             .service
//             .request_handler(request_context    )
//             .on_list_task_push_notification_config(params)
//             .await;
//         let rpc_res = JsonRpcResponse::new(res);
//         rpc_res.into()
//     }

//     #[rpc_method(name = "tasks/pushNotificationConfig/get")]
//     async fn rpc_on_get_task_push_notification_config(
//         &self,
//         request_context: RequestContext,
//         params: Value,
//     ) -> Result<Value, RPCError> {
//         let params = serde_json::from_value(params)?;
//         let res = self
//             .service
//             .request_handler(request_context)
//             .on_get_task_push_notification_config(params)
//             .await;
//         let rpc_res = JsonRpcResponse::new(res);
//         rpc_res.into()
//     }
// }

// #[rpc_pubsub_impl]
// impl JsonRpcService {
//     #[rpc_method(name = "tasks/resubscribe")]
//     async fn rpc_on_resubscribe_to_task(
//         &self,
//         chan: Arc<Channel>,
//         request_context: RequestContext,
//         method: String,
//         params: Value,
//     ) -> Result<Value, RPCError> {
//         // let sub_id: SubscriptionID = serde_json::from_value(params)?;
//         // chan.remove_subscription(&sub_id).await.unwrap();
//         // info!("Unsubscribed to log events");
//         let params = serde_json::from_value(params)?;
//         let stream = self
//             .service
//             .request_handler(request_context)
//             .on_resubscribe_to_task(params)?;
//         // tokio:spawn and deal with stream processing
//         Ok(json!({}))
//     }
//     #[rpc_method(name = "message/stream")]
//     async fn rpc_on_message_send_stream(
//         &self,
//         chan: Arc<Channel>,
//         _method: String,
//         params: Value,
//     ) -> Result<Value, RPCError> {
//         // let sub_id: SubscriptionID = serde_json::from_value(params)?;
//         // chan.remove_subscription(&sub_id).await.unwrap();
//         // info!("Unsubscribed to log events");
//         let params = serde_json::from_value(params)?;
//         let stream = self
//             .service
//             .request_handler(request_context)
//             .on_message_send_stream(params)?;
//         // tokio:spawn and deal with stream processing
//         Ok(json!({}))
//     }
// }
