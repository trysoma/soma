// We have some dummed down domain models so that
// it's easier for agents to complete and use.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::logic::{self, MessageRole, Metadata, TaskStatus};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct TextPart {
    pub text: String,
}

impl From<TextPart> for logic::TextPart {
    fn from(part: TextPart) -> Self {
        logic::TextPart {
            text: part.text,
            metadata: Metadata::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum MessagePart {
    TextPart(TextPart),
}

impl From<MessagePart> for logic::MessagePart {
    fn from(part: MessagePart) -> Self {
        match part {
            MessagePart::TextPart(part) => logic::MessagePart::TextPart(part.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct CreateMessageRequest {
    pub parts: Vec<MessagePart>,
}

impl From<CreateMessageRequest> for logic::CreateMessageRequest {
    fn from(request: CreateMessageRequest) -> Self {
        logic::CreateMessageRequest {
            parts: request.parts.into_iter().map(|part| part.into()).collect(),
            reference_task_ids: vec![],
            role: MessageRole::Agent,
            metadata: Metadata::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct UpdateTaskStatusRequest {
    pub status: TaskStatus,
}

impl From<UpdateTaskStatusRequest> for logic::UpdateTaskStatusRequest {
    fn from(request: UpdateTaskStatusRequest) -> Self {
        logic::UpdateTaskStatusRequest {
            status: request.status,
            message: None,
        }
    }
}
