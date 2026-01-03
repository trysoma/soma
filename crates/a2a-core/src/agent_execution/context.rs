use uuid::Uuid;

use crate::{
    errors::{A2aServerError, ErrorBuilder},
    types::{Message, MessageSendConfiguration, MessageSendParams, Part, Task},
};
/// Request Context.
///
/// Holds information about the current request being processed by the server,
/// including the incoming message, task and context identifiers, and related
/// tasks.
#[derive(Debug)]
pub struct RequestContext {
    params: Option<MessageSendParams>,
    task_id: Option<String>,
    context_id: Option<String>,
    current_task: Option<Task>,
    related_tasks: Vec<Task>,
}

impl RequestContext {
    /// Initializes the RequestContext.
    pub fn new(
        request: Option<MessageSendParams>,
        task_id: Option<String>,
        context_id: Option<String>,
        task: Option<Task>,
        related_tasks: Option<Vec<Task>>,
    ) -> Result<Self, A2aServerError> {
        let related_tasks = related_tasks.unwrap_or_default();
        let mut context = Self {
            params: request,
            task_id,
            context_id,
            current_task: task,
            related_tasks,
        };

        // If the task id and context id were provided, make sure they
        // match the request. Otherwise, create them

        // Generate IDs if needed
        if context.task_id.is_none() {
            context.check_or_generate_task_id();
        }
        if context.context_id.is_none() {
            context.check_or_generate_context_id();
        }

        // Now validate and set the IDs
        if let Some(ref mut params) = context.params {
            if let Some(ref task_id) = context.task_id {
                params.message.task_id = Some(task_id.clone());
                if let Some(ref task) = context.current_task {
                    if &task.id != task_id {
                        return Err(A2aServerError::InvalidParamsError(
                            ErrorBuilder::default()
                                .message("bad task id".to_string())
                                .build()
                                .unwrap(),
                        ));
                    }
                }
            }

            if let Some(ref context_id) = context.context_id {
                params.message.context_id = Some(context_id.clone());
                if let Some(ref task) = context.current_task {
                    if &task.context_id != context_id {
                        return Err(A2aServerError::InvalidParamsError(
                            ErrorBuilder::default()
                                .message("bad context id".to_string())
                                .build()
                                .unwrap(),
                        ));
                    }
                }
            }
        }

        Ok(context)
    }

    /// Extracts text content from the user's message parts.
    pub fn get_user_input(&self, delimiter: &str) -> String {
        if let Some(ref params) = self.params {
            get_message_text(&params.message, delimiter)
        } else {
            String::new()
        }
    }

    /// Attaches a related task to the context.
    ///
    /// This is useful for scenarios like tool execution where a new task
    /// might be spawned.
    pub fn attach_related_task(&mut self, task: Task) {
        self.related_tasks.push(task);
    }

    /// The incoming `Message` object from the request, if available.
    pub fn message(&self) -> Option<&Message> {
        self.params.as_ref().map(|p| &p.message)
    }

    pub fn params(&self) -> &Option<MessageSendParams> {
        &self.params
    }

    /// A list of tasks related to the current request.
    pub fn related_tasks(&self) -> &[Task] {
        &self.related_tasks
    }

    /// The current `Task` object being processed.
    pub fn current_task(&self) -> Option<&Task> {
        self.current_task.as_ref()
    }

    /// Sets the current task object.
    pub fn set_current_task(&mut self, task: Task) {
        self.current_task = Some(task);
    }

    /// The ID of the task associated with this context.
    pub fn task_id(&self) -> Option<&str> {
        self.task_id.as_deref()
    }

    /// The ID of the conversation context associated with this task.
    pub fn context_id(&self) -> Option<&str> {
        self.context_id.as_deref()
    }

    /// The `MessageSendConfiguration` from the request, if available.
    pub fn configuration(&self) -> Option<&MessageSendConfiguration> {
        self.params.as_ref().and_then(|p| p.configuration.as_ref())
    }

    /// Ensures a task ID is present, generating one if necessary.
    fn check_or_generate_task_id(&mut self) {
        if let Some(ref mut params) = self.params {
            if self.task_id.is_none() && params.message.task_id.is_none() {
                params.message.task_id = Some(Uuid::new_v4().to_string());
            }
            if let Some(ref task_id) = params.message.task_id {
                self.task_id = Some(task_id.clone());
            }
        }
    }

    /// Ensures a context ID is present, generating one if necessary.
    fn check_or_generate_context_id(&mut self) {
        if let Some(ref mut params) = self.params {
            if self.context_id.is_none() && params.message.context_id.is_none() {
                params.message.context_id = Some(Uuid::new_v4().to_string());
            }
            if let Some(ref context_id) = params.message.context_id {
                self.context_id = Some(context_id.clone());
            }
        }
    }
}

/// Extracts text content from a message by concatenating TextPart contents
pub fn get_message_text(message: &Message, delimiter: &str) -> String {
    message
        .parts
        .iter()
        .filter_map(|part| match part {
            Part::TextPart(text_part) => Some(text_part.text.as_str()),
            _ => None,
        })
        .collect::<Vec<&str>>()
        .join(delimiter)
}
