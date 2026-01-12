//! AI routes for Vercel AI SDK compatible endpoints
//!
//! Provides endpoints compatible with Vercel AI SDK's useChat hook:
//! - UI message stream (POST /ai/chat/stream) - Primary endpoint for useChat with full UI support
//! - Text stream (POST /ai/completion/stream) - For useCompletion hook (text-only)
//!
//! These routes use the InboxProviderState to publish messages to the event bus
//! and wait for responses from destinations (agents/workflows).
//!
//! ## Vercel AI SDK Protocol
//!
//! The streaming endpoints implement the Vercel AI SDK stream protocol:
//! - Required header: `x-vercel-ai-ui-message-stream: v1`
//! - SSE format with typed chunks supporting text, reasoning, tool calls, sources, files, etc.
//! - Terminates with `data: [DONE]`
//!
//! Reference: https://ai-sdk.dev/docs/ai-sdk-ui/stream-protocol

use std::time::Duration;

use axum::extract::Path;
use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::response::sse::{Event as SseEvent, Sse};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared::adapters::openapi::API_VERSION_TAG;
use shared::error::CommonError;
use shared::primitives::{PaginationRequest, WrappedJsonValue, WrappedUuidV4};
use tracing::trace;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use inbox::logic::event::{
    FinishReason, InboxEvent, InboxEventKind, MessageStreamingDelta, UiMessageDelta,
};
use inbox::logic::inbox::InboxProviderState;
use inbox::logic::message::{
    Approval, ApprovalDecision, CreateUIMessageRequest, DataUIPart, FileUIPart, Message,
    MessageRole, PartState, ReasoningUIPart, SourceDocumentUIPart, SourceUrlUIPart, StepStartUIPart,
    TextUIPart, ToolInvocationState, ToolUIPart, UIMessagePart,
};
use inbox::logic::thread::{CreateThreadRequest, get_thread_with_messages};
use inbox::repository::ThreadRepositoryLike;

pub const API_VERSION_1: &str = "v1";
pub const SERVICE_ROUTE_KEY: &str = "ai";

/// Required header for Vercel AI SDK UI message streams
pub const VERCEL_AI_STREAM_HEADER: &str = "x-vercel-ai-ui-message-stream";
pub const VERCEL_AI_STREAM_VERSION: &str = "v1";

/// Default timeout for waiting for a response from destinations
const DEFAULT_RESPONSE_TIMEOUT: Duration = Duration::from_secs(30);

// ============================================================================
// Vercel AI SDK Stream Chunk Types
// ============================================================================

/// Stream chunk types following Vercel AI SDK protocol
/// Reference: https://ai-sdk.dev/docs/ai-sdk-ui/stream-protocol
///
/// Note: This type is used only for SSE serialization and does not need JsonSchema.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum StreamChunk {
    // --- Message Control ---
    /// Start of a new message
    Start {
        #[serde(rename = "messageId")]
        message_id: String,
        #[serde(rename = "messageMetadata", skip_serializing_if = "Option::is_none")]
        message_metadata: Option<WrappedJsonValue>,
    },
    /// Complete message
    Finish {
        #[serde(rename = "finishReason", skip_serializing_if = "Option::is_none")]
        finish_reason: Option<String>,
        #[serde(rename = "messageMetadata", skip_serializing_if = "Option::is_none")]
        message_metadata: Option<WrappedJsonValue>,
    },

    // --- Text Content ---
    /// Begin text block
    TextStart { id: String },
    /// Incremental text content
    TextDelta { id: String, delta: String },
    /// End text block
    TextEnd { id: String },

    // --- Reasoning Content ---
    /// Begin reasoning block
    ReasoningStart { id: String },
    /// Incremental reasoning content
    ReasoningDelta { id: String, delta: String },
    /// End reasoning block
    ReasoningEnd { id: String },

    // --- Tool Calls ---
    /// Start of tool input streaming
    ToolInputStart {
        #[serde(rename = "toolCallId")]
        tool_call_id: String,
        #[serde(rename = "toolName")]
        tool_name: String,
    },
    /// Incremental tool input
    ToolInputDelta {
        #[serde(rename = "toolCallId")]
        tool_call_id: String,
        #[serde(rename = "inputTextDelta")]
        input_text_delta: String,
    },
    /// Tool input is fully available
    ToolInputAvailable {
        #[serde(rename = "toolCallId")]
        tool_call_id: String,
        #[serde(rename = "toolName")]
        tool_name: String,
        input: WrappedJsonValue,
    },
    /// Tool output is available
    ToolOutputAvailable {
        #[serde(rename = "toolCallId")]
        tool_call_id: String,
        output: WrappedJsonValue,
    },

    // --- Sources ---
    /// URL source reference
    SourceUrl {
        #[serde(rename = "sourceId")]
        source_id: String,
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
    },
    /// Document source reference
    SourceDocument {
        #[serde(rename = "sourceId")]
        source_id: String,
        #[serde(rename = "mediaType")]
        media_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        filename: Option<String>,
    },

    // --- Files ---
    /// File attachment
    File {
        url: String,
        #[serde(rename = "mediaType")]
        media_type: String,
    },

    // --- Steps ---
    /// Start of a step
    StartStep,
    /// End of a step
    FinishStep,

    // --- Custom Data ---
    /// Custom data chunk (type becomes "data-{data_type}")
    #[serde(rename = "data")]
    Data {
        #[serde(rename = "dataType")]
        data_type: String,
        data: WrappedJsonValue,
    },

    // --- Error ---
    /// Error occurred
    Error {
        #[serde(rename = "errorText")]
        error_text: String,
    },
}

/// Sentinel value for end of stream
const STREAM_DONE: &str = "[DONE]";

/// Convert FinishReason to string for streaming
fn finish_reason_to_string(reason: &FinishReason) -> String {
    match reason {
        FinishReason::Stop => "stop".to_string(),
        FinishReason::Length => "length".to_string(),
        FinishReason::ContentFilter => "content-filter".to_string(),
        FinishReason::ToolCalls => "tool-calls".to_string(),
        FinishReason::Error => "error".to_string(),
        FinishReason::Other => "other".to_string(),
        FinishReason::Unknown => "unknown".to_string(),
    }
}

// ============================================================================
// Request/Response Types (Vercel AI SDK Compatible)
// ============================================================================

/// A message in the Vercel AI SDK format
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct ChatMessage {
    /// Unique message ID
    pub id: String,
    /// Message role (user, assistant, system)
    pub role: String,
    /// Message parts (text, tool calls, etc.)
    pub parts: Vec<ChatMessagePart>,
}

/// Tool invocation state for chat message parts
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ChatToolInvocationState {
    InputStreaming,
    InputAvailable,
    ApprovalRequested,
    ApprovalResponded,
    OutputAvailable,
    OutputError,
    OutputDenied,
}

impl From<&ToolInvocationState> for ChatToolInvocationState {
    fn from(state: &ToolInvocationState) -> Self {
        match state {
            ToolInvocationState::InputStreaming => ChatToolInvocationState::InputStreaming,
            ToolInvocationState::InputAvailable => ChatToolInvocationState::InputAvailable,
            ToolInvocationState::ApprovalRequested => ChatToolInvocationState::ApprovalRequested,
            ToolInvocationState::ApprovalResponded => ChatToolInvocationState::ApprovalResponded,
            ToolInvocationState::OutputAvailable => ChatToolInvocationState::OutputAvailable,
            ToolInvocationState::OutputError => ChatToolInvocationState::OutputError,
            ToolInvocationState::OutputDenied => ChatToolInvocationState::OutputDenied,
        }
    }
}

impl From<&ChatToolInvocationState> for ToolInvocationState {
    fn from(state: &ChatToolInvocationState) -> Self {
        match state {
            ChatToolInvocationState::InputStreaming => ToolInvocationState::InputStreaming,
            ChatToolInvocationState::InputAvailable => ToolInvocationState::InputAvailable,
            ChatToolInvocationState::ApprovalRequested => ToolInvocationState::ApprovalRequested,
            ChatToolInvocationState::ApprovalResponded => ToolInvocationState::ApprovalResponded,
            ChatToolInvocationState::OutputAvailable => ToolInvocationState::OutputAvailable,
            ChatToolInvocationState::OutputError => ToolInvocationState::OutputError,
            ChatToolInvocationState::OutputDenied => ToolInvocationState::OutputDenied,
        }
    }
}

/// Approval decision for tool invocations
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ChatApprovalDecision {
    Approved,
    Rejected,
}

impl From<&ApprovalDecision> for ChatApprovalDecision {
    fn from(decision: &ApprovalDecision) -> Self {
        match decision {
            ApprovalDecision::Approved => ChatApprovalDecision::Approved,
            ApprovalDecision::Rejected => ChatApprovalDecision::Rejected,
        }
    }
}

impl From<&ChatApprovalDecision> for ApprovalDecision {
    fn from(decision: &ChatApprovalDecision) -> Self {
        match decision {
            ChatApprovalDecision::Approved => ApprovalDecision::Approved,
            ChatApprovalDecision::Rejected => ApprovalDecision::Rejected,
        }
    }
}

/// Approval information for tool invocations
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct ChatApproval {
    pub decision: ChatApprovalDecision,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// A part of a chat message following Vercel AI SDK UIMessagePart format
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ChatMessagePart {
    /// Text content
    Text {
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<String>,
    },
    /// Reasoning/thinking content
    Reasoning {
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<String>,
    },
    /// File attachment
    File {
        #[serde(rename = "mediaType")]
        media_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        filename: Option<String>,
        url: String,
    },
    /// Tool/function invocation
    Tool {
        #[serde(rename = "toolInvocationId")]
        tool_invocation_id: String,
        #[serde(rename = "toolName")]
        tool_name: String,
        state: ChatToolInvocationState,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schemars(with = "Option<serde_json::Value>")]
        input: Option<WrappedJsonValue>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schemars(with = "Option<serde_json::Value>")]
        output: Option<WrappedJsonValue>,
        #[serde(rename = "errorText", skip_serializing_if = "Option::is_none")]
        error_text: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        approval: Option<ChatApproval>,
    },
    /// URL source reference
    SourceUrl {
        #[serde(rename = "sourceId")]
        source_id: String,
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
    },
    /// Document source reference
    SourceDocument {
        #[serde(rename = "sourceId")]
        source_id: String,
        #[serde(rename = "mediaType")]
        media_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        filename: Option<String>,
    },
    /// Step start marker
    StepStart,
    /// Custom data part
    Data {
        #[serde(rename = "dataType")]
        data_type: String,
        #[schemars(with = "serde_json::Value")]
        data: WrappedJsonValue,
    },
}

/// Request body for chat completion (Vercel AI SDK format)
/// Compatible with useChat hook with persistence support
/// Reference: https://ai-sdk.dev/docs/ai-sdk-ui/chatbot-message-persistence
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct ChatRequest {
    /// Unique identifier for the chat/thread (used for persistence)
    /// If not provided, a new thread will be created
    #[serde(rename = "chatId", skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<String>,
    /// Array of messages in the conversation
    /// Note: When chatId is provided, these messages are used to update the thread
    pub messages: Vec<ChatMessage>,
}

/// Response body for loading chat history
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct LoadChatResponse {
    /// The chat/thread ID
    #[serde(rename = "chatId")]
    pub chat_id: String,
    /// Array of messages in the conversation
    pub messages: Vec<ChatMessage>,
}

/// Request body for text completion (Vercel AI SDK format)
/// Compatible with useCompletion hook
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct CompletionRequest {
    /// The prompt text
    pub prompt: String,
}

/// Response for non-streaming chat
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct ChatResponse {
    /// The generated message
    pub message: ChatMessage,
}

/// Response for non-streaming completion
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct CompletionResponse {
    /// The generated text
    pub text: String,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert ChatMessagePart to internal UIMessagePart
fn chat_part_to_internal(part: &ChatMessagePart) -> UIMessagePart {
    match part {
        ChatMessagePart::Text { text } => UIMessagePart::Text(TextUIPart {
            text: text.clone(),
            state: Some(PartState::Done),
            provider_metadata: None,
        }),
        ChatMessagePart::Reasoning { text } => UIMessagePart::Reasoning(ReasoningUIPart {
            text: text.clone(),
            state: Some(PartState::Done),
            provider_metadata: None,
        }),
        ChatMessagePart::File {
            media_type,
            filename,
            url,
        } => UIMessagePart::File(FileUIPart {
            media_type: media_type.clone(),
            filename: filename.clone(),
            url: url.clone(),
            provider_metadata: None,
        }),
        ChatMessagePart::Tool {
            tool_invocation_id,
            tool_name,
            state,
            input,
            output,
            error_text,
            approval,
        } => UIMessagePart::Tool(ToolUIPart {
            tool_invocation_id: tool_invocation_id.clone(),
            tool_name: tool_name.clone(),
            state: ToolInvocationState::from(state),
            input: input.clone(),
            output: output.clone(),
            error_text: error_text.clone(),
            approval: approval.as_ref().map(|a| Approval {
                decision: ApprovalDecision::from(&a.decision),
                reason: a.reason.clone(),
            }),
            provider_metadata: None,
        }),
        ChatMessagePart::SourceUrl {
            source_id,
            url,
            title,
        } => UIMessagePart::SourceUrl(SourceUrlUIPart {
            source_id: source_id.clone(),
            url: url.clone(),
            title: title.clone(),
            provider_metadata: None,
        }),
        ChatMessagePart::SourceDocument {
            source_id,
            media_type,
            title,
            filename,
        } => UIMessagePart::SourceDocument(SourceDocumentUIPart {
            source_id: source_id.clone(),
            media_type: media_type.clone(),
            title: title.clone(),
            filename: filename.clone(),
            provider_metadata: None,
        }),
        ChatMessagePart::StepStart => UIMessagePart::StepStart(StepStartUIPart {
            provider_metadata: None,
        }),
        ChatMessagePart::Data { data_type, data } => UIMessagePart::Data(DataUIPart {
            data_type: data_type.clone(),
            data: data.clone(),
            provider_metadata: None,
        }),
    }
}

/// Convert internal UIMessagePart to ChatMessagePart
fn internal_part_to_chat(part: &UIMessagePart) -> ChatMessagePart {
    match part {
        UIMessagePart::Text(text_part) => ChatMessagePart::Text {
            text: text_part.text.clone(),
        },
        UIMessagePart::Reasoning(reasoning_part) => ChatMessagePart::Reasoning {
            text: reasoning_part.text.clone(),
        },
        UIMessagePart::File(file_part) => ChatMessagePart::File {
            media_type: file_part.media_type.clone(),
            filename: file_part.filename.clone(),
            url: file_part.url.clone(),
        },
        UIMessagePart::Tool(tool_part) => ChatMessagePart::Tool {
            tool_invocation_id: tool_part.tool_invocation_id.clone(),
            tool_name: tool_part.tool_name.clone(),
            state: ChatToolInvocationState::from(&tool_part.state),
            input: tool_part.input.clone(),
            output: tool_part.output.clone(),
            error_text: tool_part.error_text.clone(),
            approval: tool_part.approval.as_ref().map(|a| ChatApproval {
                decision: ChatApprovalDecision::from(&a.decision),
                reason: a.reason.clone(),
            }),
        },
        UIMessagePart::SourceUrl(source_part) => ChatMessagePart::SourceUrl {
            source_id: source_part.source_id.clone(),
            url: source_part.url.clone(),
            title: source_part.title.clone(),
        },
        UIMessagePart::SourceDocument(source_part) => ChatMessagePart::SourceDocument {
            source_id: source_part.source_id.clone(),
            media_type: source_part.media_type.clone(),
            title: source_part.title.clone(),
            filename: source_part.filename.clone(),
        },
        UIMessagePart::StepStart(_) => ChatMessagePart::StepStart,
        UIMessagePart::Data(data_part) => ChatMessagePart::Data {
            data_type: data_part.data_type.clone(),
            data: data_part.data.clone(),
        },
    }
}

/// Convert ChatMessage to internal Message type
fn chat_message_to_internal(msg: &ChatMessage, thread_id: WrappedUuidV4) -> Message {
    let role = match msg.role.as_str() {
        "user" => MessageRole::User,
        "assistant" => MessageRole::Assistant,
        "system" => MessageRole::System,
        _ => MessageRole::User,
    };

    let parts: Vec<UIMessagePart> = msg.parts.iter().map(chat_part_to_internal).collect();

    Message::ui(thread_id, role, parts)
}

/// Convert internal Message to ChatMessage
fn internal_to_chat_message(msg: &Message) -> ChatMessage {
    let role = match msg.role() {
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
        MessageRole::System => "system",
    };

    // Extract parts based on message type
    let parts = match msg {
        Message::Text(text_msg) => {
            vec![ChatMessagePart::Text {
                text: Some(text_msg.text.clone()),
            }]
        }
        Message::Ui(ui_msg) => ui_msg.parts.iter().map(internal_part_to_chat).collect(),
    };

    ChatMessage {
        id: msg.id().to_string(),
        role: role.to_string(),
        parts,
    }
}

/// Create SSE event with JSON data
fn sse_json_event(chunk: &StreamChunk) -> Result<SseEvent, axum::Error> {
    SseEvent::default().json_data(chunk)
}

/// Create the [DONE] terminator event
fn sse_done_event() -> SseEvent {
    SseEvent::default().data(STREAM_DONE)
}

// ============================================================================
// Router
// ============================================================================

/// Creates the AI router with all endpoints
pub fn create_router() -> OpenApiRouter<InboxProviderState> {
    OpenApiRouter::new()
        .routes(routes!(route_load_chat))
        .routes(routes!(route_chat))
        .routes(routes!(route_chat_stream))
        .routes(routes!(route_completion))
        .routes(routes!(route_completion_stream))
}

// ============================================================================
// Chat History Endpoint (for persistence)
// ============================================================================

/// GET /ai/chat/{chatId} - Load chat history
///
/// Loads the chat history for a given chatId (thread ID).
/// This endpoint is used by the useChat hook to restore conversation state.
#[utoipa::path(
    get,
    path = "/ai/chat/{chatId}",
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    params(
        ("chatId" = String, Path, description = "The chat/thread ID to load")
    ),
    responses(
        (status = 200, description = "Chat history loaded successfully", body = LoadChatResponse),
        (status = 404, description = "Chat not found", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Load chat history",
    description = "Load the chat history for a given chatId. \
                   Returns the messages in Vercel AI SDK UIMessage format.",
    operation_id = "load-chat",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
pub async fn route_load_chat(
    State(state): State<InboxProviderState>,
    Path(chat_id): Path<String>,
) -> impl IntoResponse {
    trace!(inbox_id = %state.inbox.id, chat_id = %chat_id, "Loading chat history");

    // Get repository from state
    let repository = match &state.repository {
        Some(repo) => repo,
        None => {
            return (
                http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(CommonError::InvalidResponse {
                    msg: "Repository not available".to_string(),
                    source: None,
                }),
            )
                .into_response();
        }
    };

    // Parse chat_id as UUID
    let thread_id = match chat_id.parse::<WrappedUuidV4>() {
        Ok(id) => id,
        Err(_) => {
            return (
                http::StatusCode::BAD_REQUEST,
                Json(CommonError::InvalidRequest {
                    msg: format!("Invalid chat ID format: {chat_id}"),
                    source: None,
                }),
            )
                .into_response();
        }
    };

    // Load thread with messages
    let pagination = PaginationRequest {
        next_page_token: None,
        page_size: 100, // Load up to 100 messages
    };

    match get_thread_with_messages(repository.as_ref(), thread_id, pagination).await {
        Ok(response) => {
            let messages: Vec<ChatMessage> = response
                .messages
                .iter()
                .map(internal_to_chat_message)
                .collect();

            (
                http::StatusCode::OK,
                Json(LoadChatResponse {
                    chat_id,
                    messages,
                }),
            )
                .into_response()
        }
        Err(CommonError::NotFound { .. }) => (
            http::StatusCode::NOT_FOUND,
            Json(CommonError::NotFound {
                msg: format!("Chat with id {chat_id} not found"),
                lookup_id: chat_id,
                source: None,
            }),
        )
            .into_response(),
        Err(e) => (http::StatusCode::INTERNAL_SERVER_ERROR, Json(e)).into_response(),
    }
}

// ============================================================================
// Chat Endpoints (for useChat hook)
// ============================================================================

/// POST /chat - Generate chat response (non-streaming)
#[utoipa::path(
    post,
    path = format!("{}/{}/chat", API_VERSION_1, SERVICE_ROUTE_KEY),
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = ChatRequest,
    responses(
        (status = 200, description = "Generated chat response", body = ChatResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 408, description = "Request Timeout", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Generate chat response",
    description = "Generate a chat response compatible with Vercel AI SDK. \
                   Accepts messages array format from useChat hook.",
    operation_id = "chat",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
pub async fn route_chat(
    State(state): State<InboxProviderState>,
    Json(request): Json<ChatRequest>,
) -> impl IntoResponse {
    trace!(inbox_id = %state.inbox.id, "Processing chat request");

    let thread_id = WrappedUuidV4::new();

    // Get the last user message to send
    let last_message = request.messages.last();
    let input_message = match last_message {
        Some(msg) => chat_message_to_internal(msg, thread_id.clone()),
        None => {
            return (
                http::StatusCode::BAD_REQUEST,
                Json(CommonError::InvalidRequest {
                    msg: "No messages provided".to_string(),
                    source: None,
                }),
            )
                .into_response();
        }
    };

    let input_event = InboxEvent::message_created(input_message);
    let mut rx = state.handle.subscribe();

    if let Err(e) = state.handle.publish(input_event) {
        trace!(error = %e, "Failed to publish input message");
        return (
            http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CommonError::InvalidResponse {
                msg: "Failed to publish message to event bus".to_string(),
                source: None,
            }),
        )
            .into_response();
    }

    let timeout = tokio::time::timeout(DEFAULT_RESPONSE_TIMEOUT, async {
        while let Ok(event) = rx.recv().await {
            if !event.should_deliver_to_inbox(&state.inbox.id) {
                continue;
            }

            if let InboxEventKind::MessageCreated { message } = event.kind {
                if *message.role() == MessageRole::Assistant {
                    return Some(internal_to_chat_message(&message));
                }
            }
        }
        None
    })
    .await;

    match timeout {
        Ok(Some(response_message)) => {
            let response = ChatResponse {
                message: response_message,
            };
            trace!("Chat response completed");
            (http::StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => (
            http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CommonError::InvalidResponse {
                msg: "No response received from destination".to_string(),
                source: None,
            }),
        )
            .into_response(),
        Err(_) => (
            http::StatusCode::REQUEST_TIMEOUT,
            Json(CommonError::InvalidResponse {
                msg: "Timeout waiting for response".to_string(),
                source: None,
            }),
        )
            .into_response(),
    }
}

/// POST /ai/chat/stream - Stream chat response (SSE)
///
/// Primary endpoint for Vercel AI SDK useChat hook with persistence support.
/// Returns SSE stream with x-vercel-ai-ui-message-stream header.
///
/// If chatId is provided in the request, the user message will be persisted
/// to that chat thread. If no chatId is provided, a new thread will be created.
/// The assistant's response will also be persisted after streaming completes.
#[utoipa::path(
    post,
    path = "/ai/chat/stream",
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = ChatRequest,
    responses(
        (status = 200, description = "SSE stream of chat chunks"),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Stream chat response with persistence",
    description = "Stream a chat response using Server-Sent Events. \
                   Compatible with Vercel AI SDK useChat hook. \
                   When chatId is provided, messages are persisted to the database. \
                   Returns x-vercel-ai-ui-message-stream: v1 header.",
    operation_id = "chat-stream",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
pub async fn route_chat_stream(
    State(state): State<InboxProviderState>,
    Json(request): Json<ChatRequest>,
) -> impl IntoResponse {
    trace!(inbox_id = %state.inbox.id, chat_id = ?request.chat_id, "Starting chat stream");

    // Determine thread_id: use provided chatId or generate a new one
    let thread_id = if let Some(ref chat_id) = request.chat_id {
        match chat_id.parse::<WrappedUuidV4>() {
            Ok(id) => id,
            Err(_) => {
                // Invalid chat ID format - return error
                let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Result<SseEvent, axum::Error>>();
                let _ = tx.send(sse_json_event(&StreamChunk::Error {
                    error_text: format!("Invalid chatId format: {chat_id}"),
                }));
                let _ = tx.send(Ok(sse_done_event()));
                drop(tx);

                let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
                return (
                    [(VERCEL_AI_STREAM_HEADER, VERCEL_AI_STREAM_VERSION)],
                    Sse::new(stream),
                )
                    .into_response();
            }
        }
    } else {
        WrappedUuidV4::new()
    };

    let message_id = WrappedUuidV4::new().to_string();

    // Handle persistence if repository is available and chatId is provided
    let persistence_enabled = state.repository.is_some() && request.chat_id.is_some();
    if persistence_enabled {
        if let (Some(repository), Some(event_bus)) = (&state.repository, &state.event_bus) {
            // Ensure thread exists
            let thread_exists = repository.get_thread_by_id(&thread_id).await.ok().flatten().is_some();
            if !thread_exists {
                // Create new thread
                let create_request = CreateThreadRequest {
                    id: Some(thread_id.clone()),
                    title: None,
                    metadata: None,
                    inbox_settings: Default::default(),
                };
                if let Err(e) = inbox::logic::thread::create_thread(
                    repository.as_ref(),
                    event_bus,
                    create_request,
                ).await {
                    trace!(error = %e, "Failed to create thread for persistence");
                }
            }

            // Persist user message (the last message in the request)
            if let Some(last_msg) = request.messages.last() {
                let internal_parts: Vec<UIMessagePart> = last_msg.parts.iter().map(chat_part_to_internal).collect();
                let create_request = inbox::logic::message::CreateMessageRequest::Ui(CreateUIMessageRequest {
                    thread_id: thread_id.clone(),
                    role: MessageRole::User,
                    parts: internal_parts,
                    metadata: None,
                    inbox_settings: Default::default(),
                });
                if let Err(e) = inbox::logic::message::create_message(
                    repository.as_ref(),
                    event_bus,
                    create_request,
                ).await {
                    trace!(error = %e, "Failed to persist user message");
                }
            }
        }
    }

    // Get the last user message
    let last_message = request.messages.last();
    let input_message = match last_message {
        Some(msg) => chat_message_to_internal(msg, thread_id.clone()),
        None => {
            // Return error stream
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Result<SseEvent, axum::Error>>();
            let _ = tx.send(sse_json_event(&StreamChunk::Error {
                error_text: "No messages provided".to_string(),
            }));
            let _ = tx.send(Ok(sse_done_event()));
            drop(tx);

            let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
            return (
                [(VERCEL_AI_STREAM_HEADER, VERCEL_AI_STREAM_VERSION)],
                Sse::new(stream),
            )
                .into_response();
        }
    };

    let input_event = InboxEvent::message_created(input_message);
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Result<SseEvent, axum::Error>>();

    let mut event_rx = state.handle.subscribe();
    let inbox_id = state.inbox.id.clone();

    if let Err(e) = state.handle.publish(input_event) {
        let _ = tx.send(sse_json_event(&StreamChunk::Error {
            error_text: format!("Failed to publish message: {e}"),
        }));
        let _ = tx.send(Ok(sse_done_event()));
    } else {
        let msg_id = message_id.clone();
        let thread_id_for_persistence = thread_id.clone();
        let repository_for_persistence = if persistence_enabled { state.repository.clone() } else { None };
        let event_bus_for_persistence = if persistence_enabled { state.event_bus.clone() } else { None };

        tokio::spawn(async move {
            // Send start chunk
            let _ = tx.send(sse_json_event(&StreamChunk::Start {
                message_id: msg_id.clone(),
                message_metadata: None,
            }));

            // Track active parts for proper start/end sequencing
            let mut text_started = false;
            let mut reasoning_started = false;

            // Track streamed content for persistence
            let mut collected_text = String::new();
            #[allow(unused)]
            let collected_parts: Vec<UIMessagePart> = Vec::new();

            loop {
                match tokio::time::timeout(DEFAULT_RESPONSE_TIMEOUT, event_rx.recv()).await {
                    Ok(Ok(event)) => {
                        if !event.should_deliver_to_inbox(&inbox_id) {
                            continue;
                        }

                        match event.kind {
                            InboxEventKind::MessageStreaming { delta, part_id, .. } => {
                                match delta {
                                    // Simple text delta (from TextMessage sources)
                                    MessageStreamingDelta::Text(t) => {
                                        if !text_started {
                                            let _ =
                                                tx.send(sse_json_event(&StreamChunk::TextStart {
                                                    id: part_id.clone(),
                                                }));
                                            text_started = true;
                                        }
                                        // Collect text for persistence
                                        collected_text.push_str(&t.delta);
                                        let _ = tx.send(sse_json_event(&StreamChunk::TextDelta {
                                            id: part_id,
                                            delta: t.delta,
                                        }));
                                    }

                                    // UI Message Deltas - handle all types
                                    MessageStreamingDelta::Ui(ui_delta) => {
                                        match ui_delta {
                                            // --- Text ---
                                            UiMessageDelta::TextStart => {
                                                let _ = tx.send(sse_json_event(
                                                    &StreamChunk::TextStart { id: part_id },
                                                ));
                                                text_started = true;
                                            }
                                            UiMessageDelta::TextDelta { delta } => {
                                                if !text_started {
                                                    let _ = tx.send(sse_json_event(
                                                        &StreamChunk::TextStart {
                                                            id: part_id.clone(),
                                                        },
                                                    ));
                                                    text_started = true;
                                                }
                                                // Collect text for persistence
                                                collected_text.push_str(&delta);
                                                let _ = tx.send(sse_json_event(
                                                    &StreamChunk::TextDelta { id: part_id, delta },
                                                ));
                                            }
                                            UiMessageDelta::TextEnd => {
                                                let _ = tx.send(sse_json_event(
                                                    &StreamChunk::TextEnd { id: part_id },
                                                ));
                                                text_started = false;
                                            }

                                            // --- Reasoning ---
                                            UiMessageDelta::ReasoningStart => {
                                                let _ = tx.send(sse_json_event(
                                                    &StreamChunk::ReasoningStart { id: part_id },
                                                ));
                                                reasoning_started = true;
                                            }
                                            UiMessageDelta::ReasoningDelta { delta } => {
                                                if !reasoning_started {
                                                    let _ = tx.send(sse_json_event(
                                                        &StreamChunk::ReasoningStart {
                                                            id: part_id.clone(),
                                                        },
                                                    ));
                                                    reasoning_started = true;
                                                }
                                                let _ = tx.send(sse_json_event(
                                                    &StreamChunk::ReasoningDelta {
                                                        id: part_id,
                                                        delta,
                                                    },
                                                ));
                                            }
                                            UiMessageDelta::ReasoningEnd => {
                                                let _ = tx.send(sse_json_event(
                                                    &StreamChunk::ReasoningEnd { id: part_id },
                                                ));
                                                reasoning_started = false;
                                            }

                                            // --- Tool Calls ---
                                            UiMessageDelta::ToolInputStart {
                                                tool_call_id,
                                                tool_name,
                                                ..
                                            } => {
                                                let _ = tx.send(sse_json_event(
                                                    &StreamChunk::ToolInputStart {
                                                        tool_call_id,
                                                        tool_name,
                                                    },
                                                ));
                                            }
                                            UiMessageDelta::ToolInputDelta {
                                                tool_call_id,
                                                input_text_delta,
                                            } => {
                                                let _ = tx.send(sse_json_event(
                                                    &StreamChunk::ToolInputDelta {
                                                        tool_call_id,
                                                        input_text_delta,
                                                    },
                                                ));
                                            }
                                            UiMessageDelta::ToolInputAvailable {
                                                tool_call_id,
                                                tool_name,
                                                input,
                                                ..
                                            } => {
                                                let _ = tx.send(sse_json_event(
                                                    &StreamChunk::ToolInputAvailable {
                                                        tool_call_id,
                                                        tool_name,
                                                        input,
                                                    },
                                                ));
                                            }
                                            UiMessageDelta::ToolOutputAvailable {
                                                tool_call_id,
                                                output,
                                                ..
                                            } => {
                                                let _ = tx.send(sse_json_event(
                                                    &StreamChunk::ToolOutputAvailable {
                                                        tool_call_id,
                                                        output,
                                                    },
                                                ));
                                            }
                                            // Tool errors/denials - we can send as error or just skip
                                            UiMessageDelta::ToolInputError {
                                                tool_call_id,
                                                error_text,
                                                ..
                                            } => {
                                                let _ =
                                                    tx.send(sse_json_event(&StreamChunk::Error {
                                                        error_text: format!(
                                                            "Tool {tool_call_id} error: {error_text}"
                                                        ),
                                                    }));
                                            }
                                            UiMessageDelta::ToolOutputError {
                                                tool_call_id,
                                                error_text,
                                                ..
                                            } => {
                                                let _ =
                                                    tx.send(sse_json_event(&StreamChunk::Error {
                                                        error_text: format!(
                                                            "Tool {tool_call_id} output error: {error_text}"
                                                        ),
                                                    }));
                                            }
                                            UiMessageDelta::ToolOutputDenied { .. }
                                            | UiMessageDelta::ToolApprovalRequest { .. } => {
                                                // These are internal workflow events, skip in stream
                                            }

                                            // --- Sources ---
                                            UiMessageDelta::SourceUrl {
                                                source_id,
                                                url,
                                                title,
                                            } => {
                                                let _ = tx.send(sse_json_event(
                                                    &StreamChunk::SourceUrl {
                                                        source_id,
                                                        url,
                                                        title,
                                                    },
                                                ));
                                            }
                                            UiMessageDelta::SourceDocument {
                                                source_id,
                                                media_type,
                                                title,
                                                filename,
                                            } => {
                                                let _ = tx.send(sse_json_event(
                                                    &StreamChunk::SourceDocument {
                                                        source_id,
                                                        media_type,
                                                        title: Some(title),
                                                        filename,
                                                    },
                                                ));
                                            }

                                            // --- Files ---
                                            UiMessageDelta::File { url, media_type } => {
                                                let _ =
                                                    tx.send(sse_json_event(&StreamChunk::File {
                                                        url,
                                                        media_type,
                                                    }));
                                            }

                                            // --- Steps ---
                                            UiMessageDelta::StartStep => {
                                                let _ = tx
                                                    .send(sse_json_event(&StreamChunk::StartStep));
                                            }
                                            UiMessageDelta::FinishStep => {
                                                let _ = tx
                                                    .send(sse_json_event(&StreamChunk::FinishStep));
                                            }

                                            // --- Message Control ---
                                            UiMessageDelta::Start { message_metadata } => {
                                                // Already sent at the beginning, but update if metadata provided
                                                if message_metadata.is_some() {
                                                    let _ = tx.send(sse_json_event(
                                                        &StreamChunk::Start {
                                                            message_id: msg_id.clone(),
                                                            message_metadata,
                                                        },
                                                    ));
                                                }
                                            }
                                            UiMessageDelta::Finish {
                                                finish_reason,
                                                message_metadata,
                                            } => {
                                                // Close any open parts
                                                if text_started {
                                                    let _ = tx.send(sse_json_event(
                                                        &StreamChunk::TextEnd {
                                                            id: part_id.clone(),
                                                        },
                                                    ));
                                                }
                                                if reasoning_started {
                                                    let _ = tx.send(sse_json_event(
                                                        &StreamChunk::ReasoningEnd { id: part_id },
                                                    ));
                                                }
                                                let _ =
                                                    tx.send(sse_json_event(&StreamChunk::Finish {
                                                        finish_reason: finish_reason
                                                            .as_ref()
                                                            .map(finish_reason_to_string),
                                                        message_metadata,
                                                    }));
                                                let _ = tx.send(Ok(sse_done_event()));

                                                // Persist assistant message if persistence is enabled
                                                if let (Some(repo), Some(bus)) = (&repository_for_persistence, &event_bus_for_persistence) {
                                                    // Build final parts from collected content
                                                    let final_parts = if !collected_text.is_empty() {
                                                        vec![UIMessagePart::text(&collected_text)]
                                                    } else if !collected_parts.is_empty() {
                                                        collected_parts.clone()
                                                    } else {
                                                        vec![]
                                                    };

                                                    if !final_parts.is_empty() {
                                                        let create_request = inbox::logic::message::CreateMessageRequest::Ui(CreateUIMessageRequest {
                                                            thread_id: thread_id_for_persistence.clone(),
                                                            role: MessageRole::Assistant,
                                                            parts: final_parts,
                                                            metadata: None,
                                                            inbox_settings: Default::default(),
                                                        });
                                                        if let Err(e) = inbox::logic::message::create_message(
                                                            repo.as_ref(),
                                                            bus,
                                                            create_request,
                                                        ).await {
                                                            trace!(error = %e, "Failed to persist assistant message");
                                                        } else {
                                                            trace!(thread_id = %thread_id_for_persistence, "Persisted assistant message");
                                                        }
                                                    }
                                                }

                                                return;
                                            }
                                            UiMessageDelta::Abort => {
                                                let _ =
                                                    tx.send(sse_json_event(&StreamChunk::Error {
                                                        error_text: "Message generation aborted"
                                                            .to_string(),
                                                    }));
                                                let _ = tx.send(Ok(sse_done_event()));
                                                return;
                                            }

                                            // --- Custom Data ---
                                            UiMessageDelta::Data {
                                                data_type, data, ..
                                            } => {
                                                let _ =
                                                    tx.send(sse_json_event(&StreamChunk::Data {
                                                        data_type,
                                                        data,
                                                    }));
                                            }
                                            UiMessageDelta::MessageMetadata { .. } => {
                                                // Metadata updates are informational, skip
                                            }

                                            // --- Error ---
                                            UiMessageDelta::Error { error_text } => {
                                                let _ =
                                                    tx.send(sse_json_event(&StreamChunk::Error {
                                                        error_text,
                                                    }));
                                            }
                                        }
                                    }
                                }
                            }
                            InboxEventKind::MessageCreated { message } => {
                                if *message.role() == MessageRole::Assistant {
                                    // Non-streaming response - send full message content
                                    if let Message::Ui(ui_msg) = &message {
                                        // Stream each part
                                        for (idx, part) in ui_msg.parts.iter().enumerate() {
                                            let part_id = format!("part-{idx}");
                                            match part {
                                                UIMessagePart::Text(text_part) => {
                                                    let _ = tx.send(sse_json_event(
                                                        &StreamChunk::TextStart {
                                                            id: part_id.clone(),
                                                        },
                                                    ));
                                                    if let Some(text) = &text_part.text {
                                                        let _ = tx.send(sse_json_event(
                                                            &StreamChunk::TextDelta {
                                                                id: part_id.clone(),
                                                                delta: text.clone(),
                                                            },
                                                        ));
                                                    }
                                                    let _ = tx.send(sse_json_event(
                                                        &StreamChunk::TextEnd { id: part_id },
                                                    ));
                                                }
                                                UIMessagePart::Reasoning(reasoning_part) => {
                                                    let _ = tx.send(sse_json_event(
                                                        &StreamChunk::ReasoningStart {
                                                            id: part_id.clone(),
                                                        },
                                                    ));
                                                    if let Some(text) = &reasoning_part.text {
                                                        let _ = tx.send(sse_json_event(
                                                            &StreamChunk::ReasoningDelta {
                                                                id: part_id.clone(),
                                                                delta: text.clone(),
                                                            },
                                                        ));
                                                    }
                                                    let _ = tx.send(sse_json_event(
                                                        &StreamChunk::ReasoningEnd { id: part_id },
                                                    ));
                                                }
                                                UIMessagePart::Tool(tool_part) => {
                                                    // Send tool input available
                                                    if let Some(input) = &tool_part.input {
                                                        let _ = tx.send(sse_json_event(
                                                            &StreamChunk::ToolInputAvailable {
                                                                tool_call_id: tool_part
                                                                    .tool_invocation_id
                                                                    .clone(),
                                                                tool_name: tool_part
                                                                    .tool_name
                                                                    .clone(),
                                                                input: input.clone(),
                                                            },
                                                        ));
                                                    }
                                                    // Send tool output if available
                                                    if let Some(output) = &tool_part.output {
                                                        let _ = tx.send(sse_json_event(
                                                            &StreamChunk::ToolOutputAvailable {
                                                                tool_call_id: tool_part
                                                                    .tool_invocation_id
                                                                    .clone(),
                                                                output: output.clone(),
                                                            },
                                                        ));
                                                    }
                                                }
                                                UIMessagePart::File(file_part) => {
                                                    let _ = tx.send(sse_json_event(
                                                        &StreamChunk::File {
                                                            url: file_part.url.clone(),
                                                            media_type: file_part
                                                                .media_type
                                                                .clone(),
                                                        },
                                                    ));
                                                }
                                                UIMessagePart::SourceUrl(source_part) => {
                                                    let _ = tx.send(sse_json_event(
                                                        &StreamChunk::SourceUrl {
                                                            source_id: source_part
                                                                .source_id
                                                                .clone(),
                                                            url: source_part.url.clone(),
                                                            title: source_part.title.clone(),
                                                        },
                                                    ));
                                                }
                                                UIMessagePart::SourceDocument(source_part) => {
                                                    let _ = tx.send(sse_json_event(
                                                        &StreamChunk::SourceDocument {
                                                            source_id: source_part
                                                                .source_id
                                                                .clone(),
                                                            media_type: source_part
                                                                .media_type
                                                                .clone(),
                                                            title: source_part.title.clone(),
                                                            filename: source_part.filename.clone(),
                                                        },
                                                    ));
                                                }
                                                UIMessagePart::StepStart(_) => {
                                                    let _ = tx.send(sse_json_event(
                                                        &StreamChunk::StartStep,
                                                    ));
                                                }
                                                UIMessagePart::Data(data_part) => {
                                                    let _ = tx.send(sse_json_event(
                                                        &StreamChunk::Data {
                                                            data_type: data_part.data_type.clone(),
                                                            data: data_part.data.clone(),
                                                        },
                                                    ));
                                                }
                                            }
                                        }
                                    } else {
                                        // Text message - just send text
                                        let _ = tx.send(sse_json_event(&StreamChunk::TextStart {
                                            id: msg_id.clone(),
                                        }));
                                        let _ = tx.send(sse_json_event(&StreamChunk::TextDelta {
                                            id: msg_id.clone(),
                                            delta: message.text_content(),
                                        }));
                                        let _ = tx.send(sse_json_event(&StreamChunk::TextEnd {
                                            id: msg_id.clone(),
                                        }));
                                    }
                                    let _ = tx.send(sse_json_event(&StreamChunk::Finish {
                                        finish_reason: Some("stop".to_string()),
                                        message_metadata: None,
                                    }));
                                    let _ = tx.send(Ok(sse_done_event()));

                                    // Persist the complete message if persistence is enabled
                                    if let (Some(repo), Some(bus)) = (&repository_for_persistence, &event_bus_for_persistence) {
                                        let parts = match &message {
                                            Message::Ui(ui_msg) => ui_msg.parts.clone(),
                                            Message::Text(text_msg) => vec![UIMessagePart::text(&text_msg.text)],
                                        };

                                        if !parts.is_empty() {
                                            let create_request = inbox::logic::message::CreateMessageRequest::Ui(CreateUIMessageRequest {
                                                thread_id: thread_id_for_persistence.clone(),
                                                role: MessageRole::Assistant,
                                                parts,
                                                metadata: None,
                                                inbox_settings: Default::default(),
                                            });
                                            if let Err(e) = inbox::logic::message::create_message(
                                                repo.as_ref(),
                                                bus,
                                                create_request,
                                            ).await {
                                                trace!(error = %e, "Failed to persist assistant message");
                                            } else {
                                                trace!(thread_id = %thread_id_for_persistence, "Persisted assistant message");
                                            }
                                        }
                                    }

                                    return;
                                }
                            }
                            _ => continue,
                        }
                    }
                    Ok(Err(_)) => break,
                    Err(_) => {
                        let _ = tx.send(sse_json_event(&StreamChunk::Error {
                            error_text: "Timeout waiting for response".to_string(),
                        }));
                        let _ = tx.send(Ok(sse_done_event()));
                        break;
                    }
                }
            }

            // Connection closed unexpectedly - close any open parts
            if text_started {
                let _ = tx.send(sse_json_event(&StreamChunk::TextEnd { id: msg_id.clone() }));
            }
            if reasoning_started {
                let _ = tx.send(sse_json_event(&StreamChunk::ReasoningEnd {
                    id: msg_id.clone(),
                }));
            }
            let _ = tx.send(sse_json_event(&StreamChunk::Finish {
                finish_reason: None,
                message_metadata: None,
            }));
            let _ = tx.send(Ok(sse_done_event()));
        });
    }

    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);

    // Return with required Vercel AI SDK header
    (
        [(VERCEL_AI_STREAM_HEADER, VERCEL_AI_STREAM_VERSION)],
        Sse::new(stream).keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(1))
                .text("keep-alive"),
        ),
    )
        .into_response()
}

// ============================================================================
// Completion Endpoints (for useCompletion hook)
// ============================================================================

/// POST /ai/completion - Generate text completion (non-streaming)
#[utoipa::path(
    post,
    path = "/ai/completion",
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = CompletionRequest,
    responses(
        (status = 200, description = "Generated completion", body = CompletionResponse),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 408, description = "Request Timeout", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Generate text completion",
    description = "Generate a text completion. Compatible with Vercel AI SDK useCompletion hook.",
    operation_id = "completion",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
pub async fn route_completion(
    State(state): State<InboxProviderState>,
    Json(request): Json<CompletionRequest>,
) -> impl IntoResponse {
    trace!(inbox_id = %state.inbox.id, "Processing completion request");

    let thread_id = WrappedUuidV4::new();
    let input_message = Message::ui(
        thread_id,
        MessageRole::User,
        vec![UIMessagePart::text(request.prompt)],
    );

    let input_event = InboxEvent::message_created(input_message);
    let mut rx = state.handle.subscribe();

    if let Err(e) = state.handle.publish(input_event) {
        return (
            http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CommonError::InvalidResponse {
                msg: format!("Failed to publish message: {e}"),
                source: None,
            }),
        )
            .into_response();
    }

    let timeout = tokio::time::timeout(DEFAULT_RESPONSE_TIMEOUT, async {
        while let Ok(event) = rx.recv().await {
            if !event.should_deliver_to_inbox(&state.inbox.id) {
                continue;
            }

            if let InboxEventKind::MessageCreated { message } = event.kind {
                if *message.role() == MessageRole::Assistant {
                    return Some(message.text_content());
                }
            }
        }
        None
    })
    .await;

    match timeout {
        Ok(Some(text)) => {
            let response = CompletionResponse { text };
            (http::StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => (
            http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(CommonError::InvalidResponse {
                msg: "No response received".to_string(),
                source: None,
            }),
        )
            .into_response(),
        Err(_) => (
            http::StatusCode::REQUEST_TIMEOUT,
            Json(CommonError::InvalidResponse {
                msg: "Timeout waiting for response".to_string(),
                source: None,
            }),
        )
            .into_response(),
    }
}

/// POST /ai/completion/stream - Stream text completion (SSE)
///
/// For Vercel AI SDK useCompletion hook with TextStreamChatTransport.
/// Returns plain text deltas in SSE format.
#[utoipa::path(
    post,
    path = "/ai/completion/stream",
    tags = [SERVICE_ROUTE_KEY, API_VERSION_TAG],
    request_body = CompletionRequest,
    responses(
        (status = 200, description = "SSE stream of text chunks"),
        (status = 400, description = "Bad Request", body = CommonError),
        (status = 500, description = "Internal Server Error", body = CommonError),
    ),
    summary = "Stream text completion",
    description = "Stream a text completion using Server-Sent Events. \
                   Compatible with Vercel AI SDK useCompletion hook.",
    operation_id = "completion-stream",
    security(
        (),
        ("api_key" = []),
        ("bearer_token" = [])
    )
)]
pub async fn route_completion_stream(
    State(state): State<InboxProviderState>,
    Json(request): Json<CompletionRequest>,
) -> impl IntoResponse {
    trace!(inbox_id = %state.inbox.id, "Starting completion stream");

    let thread_id = WrappedUuidV4::new();
    let input_message = Message::ui(
        thread_id,
        MessageRole::User,
        vec![UIMessagePart::text(request.prompt)],
    );

    let input_event = InboxEvent::message_created(input_message);
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Result<SseEvent, axum::Error>>();

    let mut event_rx = state.handle.subscribe();
    let inbox_id = state.inbox.id.clone();

    if let Err(e) = state.handle.publish(input_event) {
        let _ = tx.send(Ok(SseEvent::default().data(format!("Error: {e}"))));
        let _ = tx.send(Ok(sse_done_event()));
    } else {
        tokio::spawn(async move {
            loop {
                match tokio::time::timeout(DEFAULT_RESPONSE_TIMEOUT, event_rx.recv()).await {
                    Ok(Ok(event)) => {
                        if !event.should_deliver_to_inbox(&inbox_id) {
                            continue;
                        }

                        match event.kind {
                            InboxEventKind::MessageStreaming { delta, .. } => {
                                let text = match delta {
                                    MessageStreamingDelta::Text(t) => t.delta,
                                    MessageStreamingDelta::Ui(UiMessageDelta::TextDelta {
                                        delta,
                                    }) => delta,
                                    MessageStreamingDelta::Ui(UiMessageDelta::Finish {
                                        ..
                                    }) => {
                                        let _ = tx.send(Ok(sse_done_event()));
                                        return;
                                    }
                                    _ => continue,
                                };

                                // For text streaming, just send raw text
                                let _ = tx.send(Ok(SseEvent::default().data(text)));
                            }
                            InboxEventKind::MessageCreated { message } => {
                                if *message.role() == MessageRole::Assistant {
                                    let _ = tx
                                        .send(Ok(SseEvent::default().data(message.text_content())));
                                    let _ = tx.send(Ok(sse_done_event()));
                                    return;
                                }
                            }
                            _ => continue,
                        }
                    }
                    Ok(Err(_)) => break,
                    Err(_) => {
                        let _ = tx.send(Ok(SseEvent::default().data("Error: Timeout")));
                        let _ = tx.send(Ok(sse_done_event()));
                        break;
                    }
                }
            }
            let _ = tx.send(Ok(sse_done_event()));
        });
    }

    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);

    Sse::new(stream)
        .keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(Duration::from_secs(1))
                .text("keep-alive"),
        )
        .into_response()
}

#[cfg(test)]
mod tests {
    mod unit {
        use super::super::*;

        #[test]
        fn test_path_constants() {
            assert_eq!(SERVICE_ROUTE_KEY, "ai");
            assert_eq!(API_VERSION_1, "v1");
        }

        #[test]
        fn test_vercel_header_constants() {
            assert_eq!(VERCEL_AI_STREAM_HEADER, "x-vercel-ai-ui-message-stream");
            assert_eq!(VERCEL_AI_STREAM_VERSION, "v1");
        }

        #[test]
        fn test_chat_request_serialization() {
            let request = ChatRequest {
                chat_id: None,
                messages: vec![ChatMessage {
                    id: "msg_1".to_string(),
                    role: "user".to_string(),
                    parts: vec![ChatMessagePart::Text {
                        text: Some("Hello".to_string()),
                    }],
                }],
            };

            let json = serde_json::to_string(&request).unwrap();
            assert!(json.contains("\"messages\""));
            assert!(json.contains("\"role\":\"user\""));
            assert!(json.contains("\"parts\""));
        }

        #[test]
        fn test_chat_request_with_chat_id_serialization() {
            let request = ChatRequest {
                chat_id: Some("550e8400-e29b-41d4-a716-446655440000".to_string()),
                messages: vec![ChatMessage {
                    id: "msg_1".to_string(),
                    role: "user".to_string(),
                    parts: vec![ChatMessagePart::Text {
                        text: Some("Hello".to_string()),
                    }],
                }],
            };

            let json = serde_json::to_string(&request).unwrap();
            assert!(json.contains("\"chatId\":\"550e8400-e29b-41d4-a716-446655440000\""));
            assert!(json.contains("\"messages\""));
        }

        #[test]
        fn test_load_chat_response_serialization() {
            let response = LoadChatResponse {
                chat_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
                messages: vec![ChatMessage {
                    id: "msg_1".to_string(),
                    role: "user".to_string(),
                    parts: vec![ChatMessagePart::Text {
                        text: Some("Hello".to_string()),
                    }],
                }],
            };

            let json = serde_json::to_string(&response).unwrap();
            assert!(json.contains("\"chatId\":\"550e8400-e29b-41d4-a716-446655440000\""));
            assert!(json.contains("\"messages\""));
        }

        #[test]
        fn test_stream_chunk_serialization() {
            let chunk = StreamChunk::TextDelta {
                id: "msg_1".to_string(),
                delta: "Hello".to_string(),
            };

            let json = serde_json::to_string(&chunk).unwrap();
            assert!(json.contains("\"type\":\"text-delta\""));
            assert!(json.contains("\"delta\":\"Hello\""));
        }

        #[test]
        fn test_stream_chunk_start() {
            let chunk = StreamChunk::Start {
                message_id: "msg_123".to_string(),
                message_metadata: None,
            };

            let json = serde_json::to_string(&chunk).unwrap();
            assert!(json.contains("\"type\":\"start\""));
            assert!(json.contains("\"messageId\":\"msg_123\""));
        }

        #[test]
        fn test_stream_chunk_finish() {
            let chunk = StreamChunk::Finish {
                finish_reason: Some("stop".to_string()),
                message_metadata: None,
            };
            let json = serde_json::to_string(&chunk).unwrap();
            assert!(json.contains("\"type\":\"finish\""));
            assert!(json.contains("\"finishReason\":\"stop\""));
        }

        #[test]
        fn test_stream_chunk_reasoning() {
            let chunk = StreamChunk::ReasoningDelta {
                id: "reasoning_1".to_string(),
                delta: "Let me think...".to_string(),
            };

            let json = serde_json::to_string(&chunk).unwrap();
            assert!(json.contains("\"type\":\"reasoning-delta\""));
            assert!(json.contains("\"delta\":\"Let me think...\""));
        }

        #[test]
        fn test_stream_chunk_tool_input() {
            let chunk = StreamChunk::ToolInputAvailable {
                tool_call_id: "call_123".to_string(),
                tool_name: "get_weather".to_string(),
                input: WrappedJsonValue::new(serde_json::json!({"city": "London"})),
            };

            let json = serde_json::to_string(&chunk).unwrap();
            assert!(json.contains("\"type\":\"tool-input-available\""));
            assert!(json.contains("\"toolCallId\":\"call_123\""));
            assert!(json.contains("\"toolName\":\"get_weather\""));
        }

        #[test]
        fn test_stream_chunk_source_url() {
            let chunk = StreamChunk::SourceUrl {
                source_id: "src_1".to_string(),
                url: "https://example.com".to_string(),
                title: Some("Example".to_string()),
            };

            let json = serde_json::to_string(&chunk).unwrap();
            assert!(json.contains("\"type\":\"source-url\""));
            assert!(json.contains("\"sourceId\":\"src_1\""));
        }

        #[test]
        fn test_stream_chunk_file() {
            let chunk = StreamChunk::File {
                url: "https://example.com/file.png".to_string(),
                media_type: "image/png".to_string(),
            };

            let json = serde_json::to_string(&chunk).unwrap();
            assert!(json.contains("\"type\":\"file\""));
            assert!(json.contains("\"mediaType\":\"image/png\""));
        }

        #[test]
        fn test_completion_request() {
            let request = CompletionRequest {
                prompt: "Complete this sentence:".to_string(),
            };

            let json = serde_json::to_string(&request).unwrap();
            assert!(json.contains("\"prompt\""));
        }

        #[test]
        fn test_chat_message_to_internal() {
            let msg = ChatMessage {
                id: "msg_1".to_string(),
                role: "user".to_string(),
                parts: vec![ChatMessagePart::Text {
                    text: Some("Hello".to_string()),
                }],
            };

            let thread_id = WrappedUuidV4::new();
            let internal = chat_message_to_internal(&msg, thread_id);

            assert_eq!(*internal.role(), MessageRole::User);
            assert_eq!(internal.text_content(), "Hello");
        }

        #[test]
        fn test_chat_message_with_tool_part() {
            let msg = ChatMessage {
                id: "msg_1".to_string(),
                role: "assistant".to_string(),
                parts: vec![
                    ChatMessagePart::Text {
                        text: Some("Let me check the weather.".to_string()),
                    },
                    ChatMessagePart::Tool {
                        tool_invocation_id: "call_123".to_string(),
                        tool_name: "get_weather".to_string(),
                        state: ChatToolInvocationState::OutputAvailable,
                        input: Some(WrappedJsonValue::new(serde_json::json!({"city": "London"}))),
                        output: Some(WrappedJsonValue::new(serde_json::json!({"temp": 20}))),
                        error_text: None,
                        approval: None,
                    },
                ],
            };

            let thread_id = WrappedUuidV4::new();
            let internal = chat_message_to_internal(&msg, thread_id);

            assert_eq!(*internal.role(), MessageRole::Assistant);
            if let Message::Ui(ui_msg) = internal {
                assert_eq!(ui_msg.parts.len(), 2);
            } else {
                panic!("Expected UI message");
            }
        }

        #[test]
        fn test_internal_to_chat_message() {
            let thread_id = WrappedUuidV4::new();
            let internal = Message::ui(
                thread_id,
                MessageRole::Assistant,
                vec![UIMessagePart::text("Response text")],
            );

            let chat_msg = internal_to_chat_message(&internal);

            assert_eq!(chat_msg.role, "assistant");
            assert_eq!(chat_msg.parts.len(), 1);
            match &chat_msg.parts[0] {
                ChatMessagePart::Text { text } => {
                    assert_eq!(text, &Some("Response text".to_string()))
                }
                _ => panic!("Expected Text part"),
            }
        }

        #[test]
        fn test_internal_to_chat_message_with_reasoning() {
            let thread_id = WrappedUuidV4::new();
            let internal = Message::ui(
                thread_id,
                MessageRole::Assistant,
                vec![
                    UIMessagePart::Reasoning(ReasoningUIPart {
                        text: Some("Thinking...".to_string()),
                        state: Some(PartState::Done),
                        provider_metadata: None,
                    }),
                    UIMessagePart::text("The answer is 42."),
                ],
            );

            let chat_msg = internal_to_chat_message(&internal);

            assert_eq!(chat_msg.parts.len(), 2);
            match &chat_msg.parts[0] {
                ChatMessagePart::Reasoning { text } => {
                    assert_eq!(text, &Some("Thinking...".to_string()))
                }
                _ => panic!("Expected Reasoning part"),
            }
        }

        #[test]
        fn test_chat_tool_invocation_state_conversion() {
            let internal_state = ToolInvocationState::OutputAvailable;
            let chat_state = ChatToolInvocationState::from(&internal_state);
            assert!(matches!(
                chat_state,
                ChatToolInvocationState::OutputAvailable
            ));

            let back = ToolInvocationState::from(&chat_state);
            assert!(matches!(back, ToolInvocationState::OutputAvailable));
        }
    }
}
