use crate::adapters::grpc::proto;
use crate::types::{self, SendStreamingMessageSuccessResponseResult};

/// Convert from proto TaskStatusUpdateEvent to internal TaskStatusUpdateEvent
impl From<proto::TaskStatusUpdateEvent> for types::TaskStatusUpdateEvent {
    fn from(proto_event: proto::TaskStatusUpdateEvent) -> Self {
        types::TaskStatusUpdateEvent {
            task_id: proto_event.task_id,
            context_id: proto_event.context_id,
            kind: "status-update".to_string(), // This field doesn't exist in proto, using default
            status: proto_event
                .status
                .expect("TaskStatusUpdateEvent must have a status")
                .into(),
            final_: proto_event.r#final,
            metadata: proto_event
                .metadata
                .map(super::impl_task::struct_to_json_map)
                .unwrap_or_default(),
        }
    }
}

/// Convert from internal TaskStatusUpdateEvent to proto TaskStatusUpdateEvent
impl From<types::TaskStatusUpdateEvent> for proto::TaskStatusUpdateEvent {
    fn from(event: types::TaskStatusUpdateEvent) -> Self {
        proto::TaskStatusUpdateEvent {
            task_id: event.task_id,
            context_id: event.context_id,
            status: Some(event.status.into()),
            r#final: event.final_,
            metadata: if event.metadata.is_empty() {
                None
            } else {
                Some(super::impl_task::json_map_to_struct(event.metadata))
            },
        }
    }
}

/// Convert from proto TaskArtifactUpdateEvent to internal TaskArtifactUpdateEvent
impl From<proto::TaskArtifactUpdateEvent> for types::TaskArtifactUpdateEvent {
    fn from(proto_event: proto::TaskArtifactUpdateEvent) -> Self {
        types::TaskArtifactUpdateEvent {
            task_id: proto_event.task_id,
            context_id: proto_event.context_id,
            kind: "artifact-update".to_string(), // This field doesn't exist in proto, using default
            artifact: proto_event
                .artifact
                .expect("TaskArtifactUpdateEvent must have an artifact")
                .into(),
            append: Some(proto_event.append),
            last_chunk: Some(proto_event.last_chunk),
            metadata: proto_event
                .metadata
                .map(super::impl_task::struct_to_json_map)
                .unwrap_or_default(),
        }
    }
}

/// Convert from internal TaskArtifactUpdateEvent to proto TaskArtifactUpdateEvent
impl From<types::TaskArtifactUpdateEvent> for proto::TaskArtifactUpdateEvent {
    fn from(event: types::TaskArtifactUpdateEvent) -> Self {
        proto::TaskArtifactUpdateEvent {
            task_id: event.task_id,
            context_id: event.context_id,
            artifact: Some(event.artifact.into()),
            append: event.append.unwrap_or(false),
            last_chunk: event.last_chunk.unwrap_or(false),
            metadata: if event.metadata.is_empty() {
                None
            } else {
                Some(super::impl_task::json_map_to_struct(event.metadata))
            },
        }
    }
}

/// Convert TaskEvent to proto StreamResponse (for streaming)
impl From<SendStreamingMessageSuccessResponseResult> for proto::StreamResponse {
    fn from(event: SendStreamingMessageSuccessResponseResult) -> Self {
        use proto::stream_response::Payload;

        let payload = match event {
            SendStreamingMessageSuccessResponseResult::Task(task) => Payload::Task(task.into()),
            SendStreamingMessageSuccessResponseResult::TaskStatusUpdateEvent(status_update) => {
                Payload::StatusUpdate(status_update.into())
            }
            SendStreamingMessageSuccessResponseResult::TaskArtifactUpdateEvent(artifact_update) => {
                Payload::ArtifactUpdate(artifact_update.into())
            }
            SendStreamingMessageSuccessResponseResult::Message(message) => {
                Payload::Msg(message.into())
            }
        };

        proto::StreamResponse {
            payload: Some(payload),
        }
    }
}

/// Convert from proto StreamResponse to TaskEvent
impl TryFrom<proto::StreamResponse> for SendStreamingMessageSuccessResponseResult {
    type Error = String;

    fn try_from(stream_resp: proto::StreamResponse) -> Result<Self, Self::Error> {
        use proto::stream_response::Payload;

        match stream_resp.payload {
            Some(Payload::Task(task)) => {
                Ok(SendStreamingMessageSuccessResponseResult::Task(task.into()))
            }
            Some(Payload::StatusUpdate(update)) => {
                Ok(SendStreamingMessageSuccessResponseResult::TaskStatusUpdateEvent(update.into()))
            }
            Some(Payload::ArtifactUpdate(update)) => Ok(
                SendStreamingMessageSuccessResponseResult::TaskArtifactUpdateEvent(update.into()),
            ),
            Some(Payload::Msg(msg)) => Ok(SendStreamingMessageSuccessResponseResult::Message(
                msg.into(),
            )),
            None => Err("StreamResponse has no payload".to_string()),
        }
    }
}

/// Convert from SendMessageSuccessResponseResult to appropriate proto response
impl From<types::SendMessageSuccessResponseResult> for proto::SendMessageResponse {
    fn from(result: types::SendMessageSuccessResponseResult) -> Self {
        use proto::send_message_response::Payload;

        let payload = match result {
            types::SendMessageSuccessResponseResult::Task(task) => Payload::Task(task.into()),
            types::SendMessageSuccessResponseResult::Message(message) => {
                Payload::Msg(message.into())
            }
        };

        proto::SendMessageResponse {
            payload: Some(payload),
        }
    }
}

/// Convert from proto SendMessageResponse to SendMessageSuccessResponseResult
impl TryFrom<proto::SendMessageResponse> for types::SendMessageSuccessResponseResult {
    type Error = String;

    fn try_from(proto_resp: proto::SendMessageResponse) -> Result<Self, Self::Error> {
        use proto::send_message_response::Payload;

        match proto_resp.payload {
            Some(Payload::Task(task)) => {
                Ok(types::SendMessageSuccessResponseResult::Task(task.into()))
            }
            Some(Payload::Msg(message)) => Ok(types::SendMessageSuccessResponseResult::Message(
                message.into(),
            )),
            None => Err("SendMessageResponse has no payload".to_string()),
        }
    }
}

impl From<proto::StreamResponse> for crate::events::Event {
    fn from(response: proto::StreamResponse) -> Self {
        match response.payload.unwrap() {
            proto::stream_response::Payload::Msg(msg) => crate::events::Event::Message(msg.into()),
            proto::stream_response::Payload::Task(task) => crate::events::Event::Task(task.into()),
            proto::stream_response::Payload::StatusUpdate(update) => {
                crate::events::Event::TaskStatusUpdate(update.into())
            }
            proto::stream_response::Payload::ArtifactUpdate(art) => {
                crate::events::Event::TaskArtifactUpdate(art.into())
            }
        }
    }
}
