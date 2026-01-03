use std::collections::{BTreeMap, HashMap};

use serde_json::{Map, Value};

use crate::adapters::grpc::impl_task::json_map_to_struct;
use crate::adapters::grpc::proto::{self, AuthenticationInfo};
use crate::types;

/// Convert from proto GetTaskRequest to internal TaskQueryParams
impl From<proto::GetTaskRequest> for types::TaskQueryParams {
    fn from(proto_req: proto::GetTaskRequest) -> Self {
        // Extract task ID from the name field (format: "tasks/{id}")
        let id = proto_req
            .name
            .strip_prefix("tasks/")
            .unwrap_or(&proto_req.name)
            .to_string();

        types::TaskQueryParams {
            id,
            history_length: if proto_req.history_length > 0 {
                Some(proto_req.history_length as i64)
            } else {
                None
            },
            metadata: Default::default(),
        }
    }
}

/// Convert from proto CancelTaskRequest to internal TaskIdParams
impl From<proto::CancelTaskRequest> for types::TaskIdParams {
    fn from(proto_req: proto::CancelTaskRequest) -> Self {
        // Extract task ID from the name field (format: "tasks/{id}")
        let id = proto_req
            .name
            .strip_prefix("tasks/")
            .unwrap_or(&proto_req.name)
            .to_string();

        types::TaskIdParams {
            id,
            metadata: Default::default(),
        }
    }
}

/// Convert from proto SendMessageRequest to internal MessageSendParams
impl From<proto::SendMessageRequest> for types::MessageSendParams {
    fn from(proto_req: proto::SendMessageRequest) -> Self {
        types::MessageSendParams {
            message: proto_req
                .request
                .expect("SendMessageRequest must have a message")
                .into(),
            configuration: proto_req.configuration.map(|c| c.into()),
            metadata: proto_req
                .metadata
                .map(super::impl_task::struct_to_json_map)
                .unwrap_or_default(),
        }
    }
}

/// Convert from proto SendMessageConfiguration to internal MessageSendConfiguration
impl From<proto::SendMessageConfiguration> for types::MessageSendConfiguration {
    fn from(proto_config: proto::SendMessageConfiguration) -> Self {
        types::MessageSendConfiguration {
            accepted_output_modes: proto_config.accepted_output_modes,
            push_notification_config: proto_config.push_notification.map(|p| p.into()),
            history_length: if proto_config.history_length > 0 {
                Some(proto_config.history_length as i64)
            } else {
                None
            },
            blocking: Some(proto_config.blocking),
        }
    }
}

impl From<types::MessageSendParams> for proto::SendMessageRequest {
    fn from(value: types::MessageSendParams) -> Self {
        proto::SendMessageRequest {
            request: Some(value.message.into()),
            configuration: value.configuration.map(|c| c.into()),
            metadata: Some(json_map_to_struct(value.metadata)),
        }
    }
}

impl From<types::MessageSendConfiguration> for proto::SendMessageConfiguration {
    fn from(value: types::MessageSendConfiguration) -> Self {
        proto::SendMessageConfiguration {
            accepted_output_modes: value.accepted_output_modes,
            push_notification: value.push_notification_config.map(|c| c.into()),
            history_length: value.history_length.unwrap_or_default().try_into().unwrap(),
            blocking: value.blocking.unwrap_or(false),
        }
    }
}

/// Convert from proto PushNotificationConfig to internal PushNotificationConfig
impl From<proto::PushNotificationConfig> for types::PushNotificationConfig {
    fn from(proto_config: proto::PushNotificationConfig) -> Self {
        types::PushNotificationConfig {
            id: Some(proto_config.id),
            url: proto_config.url,
            authentication: None, // TODO: Add authentication conversion when proto includes it
            token: None,          // TODO: Add token conversion when proto includes it
        }
    }
}

/// Convert from internal PushNotificationConfig to proto PushNotificationConfig
impl From<types::PushNotificationConfig> for proto::PushNotificationConfig {
    fn from(config: types::PushNotificationConfig) -> Self {
        proto::PushNotificationConfig {
            id: config.id.unwrap_or_default(),
            url: config.url,
            token: config.token.unwrap_or_default(),
            authentication: config.authentication.map(|auth| auth.into()),
        }
    }
}

impl From<types::PushNotificationAuthenticationInfo> for AuthenticationInfo {
    fn from(auth: types::PushNotificationAuthenticationInfo) -> Self {
        AuthenticationInfo {
            schemes: auth.schemes,
            credentials: auth.credentials.unwrap_or_default(),
        }
    }
}

/// Convert from proto CreateTaskPushNotificationConfigRequest to internal TaskPushNotificationConfig
impl From<proto::CreateTaskPushNotificationConfigRequest> for types::TaskPushNotificationConfig {
    fn from(proto_req: proto::CreateTaskPushNotificationConfigRequest) -> Self {
        // Extract task ID from the parent field (format: "tasks/{id}")
        let id = proto_req
            .parent
            .strip_prefix("tasks/")
            .unwrap_or(&proto_req.parent)
            .to_string();

        let config = proto_req
            .config
            .expect("CreateTaskPushNotificationConfigRequest must have a config");

        types::TaskPushNotificationConfig {
            task_id: id,
            push_notification_config: config
                .push_notification_config
                .expect("TaskPushNotificationConfig must have a push notification config")
                .into(),
        }
    }
}

/// Convert from internal TaskPushNotificationConfig to proto TaskPushNotificationConfig
impl From<types::TaskPushNotificationConfig> for proto::TaskPushNotificationConfig {
    fn from(config: types::TaskPushNotificationConfig) -> Self {
        // Build the name field (format: "tasks/{id}/pushNotificationConfigs/{config_id}")
        let config_id = config
            .push_notification_config
            .id
            .clone()
            .unwrap_or_default();
        let name = format!(
            "tasks/{}/pushNotificationConfigs/{}",
            config.task_id, config_id
        );

        proto::TaskPushNotificationConfig {
            name,
            push_notification_config: Some(config.push_notification_config.into()),
        }
    }
}

/// Convert from proto TaskPushNotificationConfig to internal TaskPushNotificationConfig
impl From<proto::TaskPushNotificationConfig> for types::TaskPushNotificationConfig {
    fn from(proto_config: proto::TaskPushNotificationConfig) -> Self {
        // Extract task ID from the name field (format: "tasks/{id}/pushNotificationConfigs/{config_id}")
        let parts: Vec<&str> = proto_config.name.split('/').collect();
        let task_id = if parts.len() >= 2 && parts[0] == "tasks" {
            parts[1].to_string()
        } else {
            proto_config.name.clone()
        };

        types::TaskPushNotificationConfig {
            task_id,
            push_notification_config: proto_config
                .push_notification_config
                .expect("TaskPushNotificationConfig must have a push notification config")
                .into(),
        }
    }
}

/// Convert from proto GetTaskPushNotificationConfigRequest to internal GetTaskPushNotificationConfigParams
impl From<proto::GetTaskPushNotificationConfigRequest>
    for types::GetTaskPushNotificationConfigParams
{
    fn from(proto_req: proto::GetTaskPushNotificationConfigRequest) -> Self {
        // Extract task ID and config ID from the name field (format: "tasks/{id}/pushNotificationConfigs/{config_id}")
        let parts: Vec<&str> = proto_req.name.split('/').collect();
        let (task_id, config_id) =
            if parts.len() >= 4 && parts[0] == "tasks" && parts[2] == "pushNotificationConfigs" {
                (parts[1].to_string(), Some(parts[3].to_string()))
            } else {
                (proto_req.name.clone(), None)
            };

        types::GetTaskPushNotificationConfigParams {
            id: task_id,
            push_notification_config_id: config_id,
            metadata: Default::default(),
        }
    }
}

/// Convert from proto ListTaskPushNotificationConfigRequest to internal ListTaskPushNotificationConfigParams
impl From<proto::ListTaskPushNotificationConfigRequest>
    for types::ListTaskPushNotificationConfigParams
{
    fn from(proto_req: proto::ListTaskPushNotificationConfigRequest) -> Self {
        // Extract task ID from the parent field (format: "tasks/{id}")
        let id = proto_req
            .parent
            .strip_prefix("tasks/")
            .unwrap_or(&proto_req.parent)
            .to_string();

        types::ListTaskPushNotificationConfigParams {
            id,
            metadata: Default::default(),
        }
    }
}

/// Convert from proto DeleteTaskPushNotificationConfigRequest to internal DeleteTaskPushNotificationConfigParams
impl From<proto::DeleteTaskPushNotificationConfigRequest>
    for types::DeleteTaskPushNotificationConfigParams
{
    fn from(proto_req: proto::DeleteTaskPushNotificationConfigRequest) -> Self {
        // Extract task ID and config ID from the name field (format: "tasks/{id}/pushNotificationConfigs/{config_id}")
        let parts: Vec<&str> = proto_req.name.split('/').collect();
        let (task_id, config_id) =
            if parts.len() >= 4 && parts[0] == "tasks" && parts[2] == "pushNotificationConfigs" {
                (parts[1].to_string(), parts[3].to_string())
            } else {
                // Fallback if format doesn't match
                (proto_req.name.clone(), String::new())
            };

        types::DeleteTaskPushNotificationConfigParams {
            id: task_id,
            push_notification_config_id: config_id,
            metadata: Default::default(),
        }
    }
}

/// Convert from proto TaskSubscriptionRequest to internal TaskIdParams
impl From<proto::TaskSubscriptionRequest> for types::TaskIdParams {
    fn from(proto_req: proto::TaskSubscriptionRequest) -> Self {
        // Extract task ID from the name field (format: "tasks/{id}")
        let id = proto_req
            .name
            .strip_prefix("tasks/")
            .unwrap_or(&proto_req.name)
            .to_string();

        types::TaskIdParams {
            id,
            metadata: Default::default(),
        }
    }
}

impl From<Vec<types::TaskPushNotificationConfig>>
    for proto::ListTaskPushNotificationConfigResponse
{
    fn from(configs: Vec<types::TaskPushNotificationConfig>) -> Self {
        proto::ListTaskPushNotificationConfigResponse {
            configs: configs.into_iter().map(Into::into).collect(),
            // TODO spec doesnt allow ommitting this.
            next_page_token: "".to_string(),
        }
    }
}

impl From<types::AgentCardSignature> for proto::AgentCardSignature {
    fn from(signature: types::AgentCardSignature) -> Self {
        proto::AgentCardSignature {
            signature: signature.signature,
            protected: signature.protected,
            header: Some(from_map_to_struct(signature.header)),
        }
    }
}

fn from_value_to_prost_value(value: Value) -> prost_types::Value {
    match value {
        Value::Null => prost_types::Value {
            kind: Some(prost_types::value::Kind::NullValue(0)),
        },
        Value::Bool(value) => prost_types::Value {
            kind: Some(prost_types::value::Kind::BoolValue(value)),
        },
        Value::Number(number) => prost_types::Value {
            kind: Some(prost_types::value::Kind::NumberValue(
                number.as_f64().unwrap(),
            )),
        },
        Value::String(value) => prost_types::Value {
            kind: Some(prost_types::value::Kind::StringValue(value)),
        },
        Value::Array(values) => prost_types::Value {
            kind: Some(prost_types::value::Kind::ListValue(
                prost_types::ListValue {
                    values: values.into_iter().map(from_value_to_prost_value).collect(),
                },
            )),
        },
        Value::Object(map) => prost_types::Value {
            kind: Some(prost_types::value::Kind::StructValue(from_map_to_struct(
                map,
            ))),
        },
    }
}

fn from_map_to_struct(map: Map<String, Value>) -> prost_types::Struct {
    let mut struct_map = BTreeMap::new();
    for (key, value) in map {
        struct_map.insert(key, from_value_to_prost_value(value));
    }
    prost_types::Struct { fields: struct_map }
}

impl From<types::AgentCard> for proto::AgentCard {
    fn from(card: types::AgentCard) -> Self {
        proto::AgentCard {
            protocol_version: card.protocol_version,
            name: card.name,
            description: card.description,
            url: card.url,
            preferred_transport: card.preferred_transport,
            signatures: card.signatures.into_iter().map(Into::into).collect(),
            additional_interfaces: card
                .additional_interfaces
                .into_iter()
                .map(Into::into)
                .collect(),
            provider: card.provider.map(|p| p.into()),
            version: card.version,
            documentation_url: card.documentation_url.unwrap_or_default(),
            capabilities: Some(card.capabilities.into()),
            security_schemes: map_security_schemes_to_protos(card.security_schemes),
            security: map_security_to_proto(card.security),
            default_input_modes: card.default_input_modes,
            default_output_modes: card.default_output_modes,
            skills: card.skills.into_iter().map(Into::into).collect(),
            supports_authenticated_extended_card: card
                .supports_authenticated_extended_card
                .unwrap_or(false),
            icon_url: card.icon_url.unwrap_or_default(),
        }
    }
}

impl From<types::AgentSkill> for proto::AgentSkill {
    fn from(value: types::AgentSkill) -> Self {
        proto::AgentSkill {
            id: value.id,
            name: value.name,
            description: value.description,
            tags: value.tags,
            examples: value.examples,
            security: value
                .security
                .into_iter()
                .map(|s| {
                    let mut schemes = BTreeMap::new();
                    for (key, value) in s {
                        schemes.insert(key, proto::StringList { list: value });
                    }
                    proto::Security { schemes }
                })
                .collect(),
            input_modes: value.input_modes,
            output_modes: value.output_modes,
        }
    }
}

fn map_security_to_proto(
    security: Vec<HashMap<std::string::String, Vec<std::string::String>>>,
) -> Vec<proto::Security> {
    security
        .into_iter()
        .map(|s| {
            let mut schemes = BTreeMap::new();
            for (key, value) in s {
                schemes.insert(key, proto::StringList { list: value });
            }

            proto::Security { schemes }
        })
        .collect()
}

fn map_security_schemes_to_protos(
    schemes: HashMap<std::string::String, types::SecurityScheme>,
) -> BTreeMap<std::string::String, proto::SecurityScheme> {
    schemes.into_iter().map(|(k, v)| (k, v.into())).collect()
}

impl From<types::AgentInterface> for proto::AgentInterface {
    fn from(value: types::AgentInterface) -> Self {
        proto::AgentInterface {
            url: value.url,
            transport: value.transport,
        }
    }
}

impl From<types::AgentProvider> for proto::AgentProvider {
    fn from(value: types::AgentProvider) -> Self {
        proto::AgentProvider {
            url: value.url,
            organization: value.organization,
        }
    }
}

impl From<types::AgentCapabilities> for proto::AgentCapabilities {
    fn from(value: types::AgentCapabilities) -> Self {
        proto::AgentCapabilities {
            streaming: value.streaming.unwrap_or(false),
            push_notifications: value.push_notifications.unwrap_or(false),
            extensions: value.extensions.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<types::AgentExtension> for proto::AgentExtension {
    fn from(value: types::AgentExtension) -> Self {
        proto::AgentExtension {
            uri: value.uri,
            description: value.description.unwrap_or_default(),
            required: value.required.unwrap_or(false),
            params: Some(json_map_to_struct(value.params)),
        }
    }
}

impl From<types::MutualTlsSecurityScheme> for proto::MutualTlsSecurityScheme {
    fn from(value: types::MutualTlsSecurityScheme) -> Self {
        proto::MutualTlsSecurityScheme {
            description: value.description.unwrap_or_default(),
        }
    }
}

impl From<types::SecurityScheme> for proto::SecurityScheme {
    fn from(value: types::SecurityScheme) -> Self {
        let scheme = match value {
            types::SecurityScheme::MutualTlsSecurityScheme(val) => {
                proto::security_scheme::Scheme::MtlsSecurityScheme(val.into())
            }
            types::SecurityScheme::ApiKeySecurityScheme(val) => {
                proto::security_scheme::Scheme::ApiKeySecurityScheme(val.into())
            }
            types::SecurityScheme::HttpAuthSecurityScheme(val) => {
                proto::security_scheme::Scheme::HttpAuthSecurityScheme(val.into())
            }
            types::SecurityScheme::OAuth2SecurityScheme(val) => {
                proto::security_scheme::Scheme::Oauth2SecurityScheme(val.into())
            }
            types::SecurityScheme::OpenIdConnectSecurityScheme(val) => {
                proto::security_scheme::Scheme::OpenIdConnectSecurityScheme(val.into())
            }
        };

        proto::SecurityScheme {
            scheme: Some(scheme),
        }
    }
}

impl From<types::ApiKeySecurityScheme> for proto::ApiKeySecurityScheme {
    fn from(value: types::ApiKeySecurityScheme) -> Self {
        proto::ApiKeySecurityScheme {
            description: value.description.unwrap_or_default(),
            location: value.in_.to_string(),
            name: value.name,
        }
    }
}

impl From<types::HttpAuthSecurityScheme> for proto::HttpAuthSecurityScheme {
    fn from(value: types::HttpAuthSecurityScheme) -> Self {
        proto::HttpAuthSecurityScheme {
            description: value.description.unwrap_or_default(),
            scheme: value.scheme.to_string(),
            bearer_format: value.bearer_format.unwrap_or_default(),
        }
    }
}

impl From<types::OAuth2SecurityScheme> for proto::OAuth2SecurityScheme {
    fn from(value: types::OAuth2SecurityScheme) -> Self {
        proto::OAuth2SecurityScheme {
            description: value.description.unwrap_or_default(),
            flows: Some(value.flows.into()),
            oauth2_metadata_url: value.oauth2_metadata_url.unwrap_or_default(),
        }
    }
}

impl From<types::OpenIdConnectSecurityScheme> for proto::OpenIdConnectSecurityScheme {
    fn from(value: types::OpenIdConnectSecurityScheme) -> Self {
        proto::OpenIdConnectSecurityScheme {
            description: value.description.unwrap_or_default(),
            open_id_connect_url: value.open_id_connect_url,
        }
    }
}

fn map_scopes_to_proto(scopes: HashMap<String, String>) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for (key, value) in scopes {
        map.insert(key, value);
    }
    map
}

impl From<types::OAuthFlows> for proto::OAuthFlows {
    fn from(value: types::OAuthFlows) -> Self {
        let flow = if let Some(val) = value.authorization_code {
            proto::o_auth_flows::Flow::AuthorizationCode(proto::AuthorizationCodeOAuthFlow {
                authorization_url: val.authorization_url,
                token_url: val.token_url,
                refresh_url: val.refresh_url.unwrap_or_default(),
                scopes: map_scopes_to_proto(val.scopes),
            })
        } else if let Some(val) = value.implicit {
            proto::o_auth_flows::Flow::Implicit(proto::ImplicitOAuthFlow {
                authorization_url: val.authorization_url,
                refresh_url: val.refresh_url.unwrap_or_default(),
                scopes: map_scopes_to_proto(val.scopes),
            })
        } else if let Some(val) = value.password {
            proto::o_auth_flows::Flow::Password(proto::PasswordOAuthFlow {
                refresh_url: val.refresh_url.unwrap_or_default(),
                scopes: map_scopes_to_proto(val.scopes),
                token_url: val.token_url,
            })
        } else if let Some(val) = value.client_credentials {
            proto::o_auth_flows::Flow::ClientCredentials(proto::ClientCredentialsOAuthFlow {
                refresh_url: val.refresh_url.unwrap_or_default(),
                scopes: map_scopes_to_proto(val.scopes),
                token_url: val.token_url,
            })
        } else {
            unreachable!()
        };

        proto::OAuthFlows { flow: Some(flow) }
    }
}
