use crate::adapters::grpc::proto::{self};
use crate::types::{self, TaskState};
use chrono::{DateTime, Utc};
use std::collections::BTreeMap;

/// Convert from proto Task to internal Task
impl From<proto::Task> for types::Task {
    fn from(proto_task: proto::Task) -> Self {
        types::Task {
            id: proto_task.id,
            context_id: proto_task.context_id,
            kind: "task".to_string(), // This field doesn't exist in proto, using default
            status: proto_task
                .status
                .map(|s| s.into())
                .unwrap_or_else(|| types::TaskStatus {
                    state: TaskState::Unknown,
                    message: None,
                    timestamp: None,
                }),
            artifacts: proto_task.artifacts.into_iter().map(|a| a.into()).collect(),
            history: proto_task.history.into_iter().map(|m| m.into()).collect(),
            metadata: proto_task
                .metadata
                .map(struct_to_json_map)
                .unwrap_or_default(),
        }
    }
}

/// Convert from internal Task to proto Task
impl From<types::Task> for proto::Task {
    fn from(task: types::Task) -> Self {
        proto::Task {
            id: task.id,
            context_id: task.context_id,
            status: Some(task.status.into()),
            artifacts: task.artifacts.into_iter().map(|a| a.into()).collect(),
            history: task.history.into_iter().map(|m| m.into()).collect(),
            metadata: if task.metadata.is_empty() {
                None
            } else {
                Some(json_map_to_struct(task.metadata))
            },
        }
    }
}

/// Convert from proto TaskStatus to internal TaskStatus
impl From<proto::TaskStatus> for types::TaskStatus {
    fn from(proto_status: proto::TaskStatus) -> Self {
        types::TaskStatus {
            state: match proto::TaskState::try_from(proto_status.state) {
                Ok(state) => state.into(),
                Err(_) => TaskState::Unknown,
            },
            message: proto_status.update.map(|m| m.into()),
            timestamp: proto_status
                .timestamp
                .and_then(|ts| {
                    let seconds = ts.seconds;
                    let nanos = ts.nanos as u32;
                    DateTime::<Utc>::from_timestamp(seconds, nanos)
                })
                .map(|dt| dt.to_rfc3339()),
        }
    }
}

/// Convert from internal TaskStatus to proto TaskStatus
impl From<types::TaskStatus> for proto::TaskStatus {
    fn from(status: types::TaskStatus) -> Self {
        proto::TaskStatus {
            state: proto::TaskState::from(status.state).into(),
            update: status.message.map(|m| m.into()),
            timestamp: status.timestamp.and_then(|ts| {
                chrono::DateTime::parse_from_rfc3339(&ts)
                    .ok()
                    .map(|dt| prost_types::Timestamp {
                        seconds: dt.timestamp(),
                        nanos: dt.timestamp_subsec_nanos() as i32,
                    })
            }),
        }
    }
}

/// Convert from proto TaskState to internal TaskState
impl From<proto::TaskState> for types::TaskState {
    fn from(proto_state: proto::TaskState) -> Self {
        match proto_state {
            proto::TaskState::Unspecified => TaskState::Unknown,
            proto::TaskState::Submitted => TaskState::Submitted,
            proto::TaskState::Working => TaskState::Working,
            proto::TaskState::InputRequired => TaskState::InputRequired,
            proto::TaskState::Completed => TaskState::Completed,
            proto::TaskState::Cancelled => TaskState::Canceled,
            proto::TaskState::Failed => TaskState::Failed,
            proto::TaskState::Rejected => TaskState::Rejected,
            proto::TaskState::AuthRequired => TaskState::AuthRequired,
        }
    }
}

/// Convert from internal TaskState to proto TaskState
impl From<types::TaskState> for proto::TaskState {
    fn from(state: types::TaskState) -> Self {
        match state {
            TaskState::Unknown => proto::TaskState::Unspecified,
            TaskState::Submitted => proto::TaskState::Submitted,
            TaskState::Working => proto::TaskState::Working,
            TaskState::InputRequired => proto::TaskState::InputRequired,
            TaskState::Completed => proto::TaskState::Completed,
            TaskState::Canceled => proto::TaskState::Cancelled,
            TaskState::Failed => proto::TaskState::Failed,
            TaskState::Rejected => proto::TaskState::Rejected,
            TaskState::AuthRequired => proto::TaskState::AuthRequired,
        }
    }
}

/// Helper function to convert prost_types::Struct to serde_json::Map
pub(super) fn struct_to_json_map(
    s: prost_types::Struct,
) -> serde_json::Map<String, serde_json::Value> {
    let mut map = serde_json::Map::new();
    for (k, v) in s.fields {
        if let Some(value) = proto_value_to_json(v) {
            map.insert(k, value);
        }
    }
    map
}

/// Helper function to convert serde_json::Map to prost_types::Struct
pub(super) fn json_map_to_struct(
    map: serde_json::Map<String, serde_json::Value>,
) -> prost_types::Struct {
    let mut fields = BTreeMap::new();
    for (k, v) in map {
        if let Some(value) = json_to_proto_value(v) {
            fields.insert(k, value);
        }
    }
    prost_types::Struct { fields }
}

/// Convert prost_types::Value to serde_json::Value
fn proto_value_to_json(value: prost_types::Value) -> Option<serde_json::Value> {
    use prost_types::value::Kind;
    match value.kind? {
        Kind::NullValue(_) => Some(serde_json::Value::Null),
        Kind::NumberValue(n) => Some(serde_json::Value::Number(serde_json::Number::from_f64(n)?)),
        Kind::StringValue(s) => Some(serde_json::Value::String(s)),
        Kind::BoolValue(b) => Some(serde_json::Value::Bool(b)),
        Kind::StructValue(s) => Some(serde_json::Value::Object(struct_to_json_map(s))),
        Kind::ListValue(l) => Some(serde_json::Value::Array(
            l.values
                .into_iter()
                .filter_map(proto_value_to_json)
                .collect(),
        )),
    }
}

/// Convert serde_json::Value to prost_types::Value
fn json_to_proto_value(value: serde_json::Value) -> Option<prost_types::Value> {
    use prost_types::value::Kind;
    let kind = match value {
        serde_json::Value::Null => Kind::NullValue(0),
        serde_json::Value::Bool(b) => Kind::BoolValue(b),
        serde_json::Value::Number(n) => Kind::NumberValue(n.as_f64()?),
        serde_json::Value::String(s) => Kind::StringValue(s),
        serde_json::Value::Array(arr) => Kind::ListValue(prost_types::ListValue {
            values: arr.into_iter().filter_map(json_to_proto_value).collect(),
        }),
        serde_json::Value::Object(obj) => Kind::StructValue(json_map_to_struct(obj)),
    };
    Some(prost_types::Value { kind: Some(kind) })
}

// impl From<SendStreamingMessageSuccessResponseResult> for proto::StreamResponse {
//     fn from(event: SendStreamingMessageSuccessResponseResult) -> Self {
//         let payload = match event {
//             SendStreamingMessageSuccessResponseResult::Task(task) => {
//                 stream_response::Payload::Task(task.into())
//             }
//             SendStreamingMessageSuccessResponseResult::TaskStatusUpdateEvent(
//                 task_status_update_event,
//             ) => stream_response::Payload::StatusUpdate(task_status_update_event.into()),
//             SendStreamingMessageSuccessResponseResult::TaskArtifactUpdateEvent(
//                 task_artifact_update_event,
//             ) => stream_response::Payload::ArtifactUpdate(task_artifact_update_event.into()),
//             SendStreamingMessageSuccessResponseResult::Message(message) => {
//                 stream_response::Payload::Msg(message.into())
//             }
//         };

//         return proto::StreamResponse {
//             payload: Some(payload),
//         };
//     }
// }
