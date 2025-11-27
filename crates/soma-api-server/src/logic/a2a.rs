// let task_store = Arc::new(
//     InMemoryTaskStoreBuilder::default()
//         .tasks(Arc::new(RwLock::new(HashMap::new())))
//         .build()
//         .unwrap(),
// );

use crate::{
    logic::task as task_logic,
    repository::{CreateTask, Repository, TaskRepositoryLike},
};
use a2a_rs::types::Task;
use a2a_rs::{
    errors::A2aServerError,
    tasks::store::TaskStore,
    types::{TaskId, TaskState, TaskStatus},
};
use shared::{
    error::CommonError,
    primitives::{WrappedChronoDateTime, WrappedJsonValue, WrappedUuidV4},
};
use tracing::info;

pub struct RepositoryTaskStore {
    repository: Repository,
}

impl RepositoryTaskStore {
    pub fn new(repository: Repository) -> Self {
        Self { repository }
    }
}

// TODO: implement better type conversion
fn convert_common_error(error: CommonError) -> A2aServerError {
    A2aServerError::InternalError(a2a_rs::errors::Error {
        message: error.to_string(),
        data: None,
        source: Some(Box::new(error)),
    })
}

#[async_trait::async_trait]
impl TaskStore for RepositoryTaskStore {
    async fn save(&self, task: &Task) -> Result<(), A2aServerError> {
        info!("Saving task: {:?}", task);
        let task_id = WrappedUuidV4::try_from(task.id.clone()).map_err(convert_common_error)?;

        // Check if task already exists
        let existing_task = self
            .repository
            .get_task_by_id(&task_id)
            .await
            .map_err(convert_common_error)?;

        let now = WrappedChronoDateTime::now();
        let status = match task.status.state {
            TaskState::Submitted => task_logic::TaskStatus::Submitted,
            TaskState::Working => task_logic::TaskStatus::Working,
            TaskState::Completed => task_logic::TaskStatus::Completed,
            TaskState::Failed => task_logic::TaskStatus::Failed,
            TaskState::Canceled => task_logic::TaskStatus::Canceled,
            TaskState::InputRequired => task_logic::TaskStatus::InputRequired,
            TaskState::Rejected => task_logic::TaskStatus::Rejected,
            TaskState::AuthRequired => task_logic::TaskStatus::AuthRequired,
            TaskState::Unknown => task_logic::TaskStatus::Unknown,
        };
        let status_timestamp =
            WrappedChronoDateTime::try_from(task.status.timestamp.as_deref().unwrap_or(""))
                .map_err(convert_common_error)?;

        if existing_task.is_some() {
            // Task exists, update it
            self.repository
                .update_task_status(&crate::repository::UpdateTaskStatus {
                    id: task_id,
                    status,
                    status_message_id: None, // TODO: Handle status message if present
                    status_timestamp,
                    updated_at: now,
                })
                .await
                .map_err(convert_common_error)
        } else {
            // Task doesn't exist, create it
            self.repository
                .create_task(&CreateTask {
                    id: task_id,
                    context_id: WrappedUuidV4::try_from(task.context_id.clone())
                        .map_err(convert_common_error)?,
                    status,
                    status_timestamp,
                    metadata: WrappedJsonValue::from(serde_json::Value::Object(
                        task.metadata.clone(),
                    )),
                    created_at: now,
                    updated_at: now,
                })
                .await
                .map_err(convert_common_error)
        }
    }

    async fn get(&self, id: &TaskId) -> Result<Option<Task>, A2aServerError> {
        let task = self
            .repository
            .get_task_by_id(&WrappedUuidV4::try_from(id.clone()).map_err(convert_common_error)?)
            .await
            .map_err(convert_common_error)?;

        let task_with_details = match task {
            Some(x) => x,
            None => return Ok(None),
        };

        // Convert status_message to a2a_rs::types::Message if present
        let status_message = task_with_details.status_message.map(|msg| msg.into());

        Ok(Some(Task {
            artifacts: vec![],
            context_id: task_with_details.task.context_id.to_string(),
            history: task_with_details
                .messages
                .iter()
                .map(|msg| msg.clone().into())
                .collect(),
            id: task_with_details.task.id.to_string(),
            kind: "task".to_string(),
            metadata: task_with_details.task.metadata.0,
            status: TaskStatus {
                message: status_message,
                state: match task_with_details.task.status {
                    task_logic::TaskStatus::Submitted => TaskState::Submitted,
                    task_logic::TaskStatus::Working => TaskState::Working,
                    task_logic::TaskStatus::Completed => TaskState::Completed,
                    task_logic::TaskStatus::Failed => TaskState::Failed,
                    task_logic::TaskStatus::Canceled => TaskState::Canceled,
                    task_logic::TaskStatus::InputRequired => TaskState::InputRequired,
                    task_logic::TaskStatus::Rejected => TaskState::Rejected,
                    task_logic::TaskStatus::AuthRequired => TaskState::AuthRequired,
                    task_logic::TaskStatus::Unknown => TaskState::Unknown,
                },
                timestamp: Some(task_with_details.task.status_timestamp.to_string()),
            },
        }))
    }

    async fn delete(&self, _id: &TaskId) -> Result<(), A2aServerError> {
        // TODO: Implement task deletion
        Err(A2aServerError::InternalError(a2a_rs::errors::Error {
            message: "Task deletion not implemented".to_string(),
            data: None,
            source: None,
        }))
    }
}

use std::collections::HashMap;

use a2a_rs::types::AgentCard;
use shared::soma_agent_definition::SomaAgentDefinition;

pub struct ConstructAgentCardParams {
    pub definition: SomaAgentDefinition,
    pub url: String,
}

pub fn construct_agent_card(params: ConstructAgentCardParams) -> a2a_rs::types::AgentCard {
    let _definition = params.definition;
    let url = params.url;

    AgentCard {
        additional_interfaces: vec![],
        capabilities: a2a_rs::types::AgentCapabilities {
            streaming: Some(true),
            push_notifications: None,
            state_transition_history: None,
            extensions: vec![],
        },
        default_input_modes: vec![],
        default_output_modes: vec![],
        description: String::new(),
        documentation_url: None,
        icon_url: None,
        name: String::new(),
        preferred_transport: "JSONRPC".to_string(),
        protocol_version: "1.0.0".to_string(),
        provider: None,
        security: vec![],
        security_schemes: HashMap::new(),
        signatures: vec![],
        skills: vec![],
        supports_authenticated_extended_card: None,
        url: url.to_string(),
        version: "1.0.0".to_string(),
    }
}
