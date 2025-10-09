use crate::adapters::grpc::proto;
use crate::types::{self, MessageRole};
use base64::{Engine as _, engine::general_purpose};

/// Convert from proto Message to internal Message
impl From<proto::Message> for types::Message {
    fn from(proto_msg: proto::Message) -> Self {
        types::Message {
            message_id: proto_msg.message_id,
            context_id: if proto_msg.context_id.is_empty() {
                None
            } else {
                Some(proto_msg.context_id)
            },
            task_id: if proto_msg.task_id.is_empty() {
                None
            } else {
                Some(proto_msg.task_id)
            },
            role: match proto::Role::try_from(proto_msg.role) {
                Ok(role) => role.into(),
                Err(_) => MessageRole::User, // Default to User if unknown
            },
            kind: "message".to_string(), // This field doesn't exist in proto, using default
            parts: proto_msg.content.into_iter().map(|p| p.into()).collect(),
            extensions: proto_msg.extensions,
            reference_task_ids: vec![], // This field doesn't exist in proto
            metadata: proto_msg
                .metadata
                .map(super::impl_task::struct_to_json_map)
                .unwrap_or_default(),
        }
    }
}

/// Convert from internal Message to proto Message
impl From<types::Message> for proto::Message {
    fn from(msg: types::Message) -> Self {
        proto::Message {
            message_id: msg.message_id,
            context_id: msg.context_id.unwrap_or_default(),
            task_id: msg.task_id.unwrap_or_default(),
            role: proto::Role::from(msg.role).into(),
            content: msg.parts.into_iter().map(|p| p.into()).collect(),
            metadata: if msg.metadata.is_empty() {
                None
            } else {
                Some(super::impl_task::json_map_to_struct(msg.metadata))
            },
            extensions: msg.extensions,
        }
    }
}

/// Convert from proto Role to internal MessageRole
impl From<proto::Role> for types::MessageRole {
    fn from(proto_role: proto::Role) -> Self {
        match proto_role {
            proto::Role::Unspecified => MessageRole::User, // Default to User
            proto::Role::User => MessageRole::User,
            proto::Role::Agent => MessageRole::Agent,
        }
    }
}

/// Convert from internal MessageRole to proto Role
impl From<types::MessageRole> for proto::Role {
    fn from(role: types::MessageRole) -> Self {
        match role {
            MessageRole::User => proto::Role::User,
            MessageRole::Agent => proto::Role::Agent,
        }
    }
}

/// Convert from proto Part to internal Part
impl From<proto::Part> for types::Part {
    fn from(proto_part: proto::Part) -> Self {
        use proto::part::Part as ProtoPartEnum;

        match proto_part.part {
            Some(ProtoPartEnum::Text(text)) => types::Part::TextPart(types::TextPart {
                text,
                kind: "text".to_string(),
                metadata: Default::default(),
            }),
            Some(ProtoPartEnum::File(file)) => types::Part::FilePart(file.into()),
            Some(ProtoPartEnum::Data(data)) => types::Part::DataPart(data.into()),
            None => {
                // Default to empty text part if no content
                types::Part::TextPart(types::TextPart {
                    text: String::new(),
                    kind: "text".to_string(),
                    metadata: Default::default(),
                })
            }
        }
    }
}

/// Convert from internal Part to proto Part
impl From<types::Part> for proto::Part {
    fn from(part: types::Part) -> Self {
        use proto::part::Part as ProtoPartEnum;

        let (part_enum, metadata) = match part {
            types::Part::TextPart(text_part) => {
                (ProtoPartEnum::Text(text_part.text), text_part.metadata)
            }
            types::Part::FilePart(file_part) => (
                ProtoPartEnum::File(file_part.clone().into()),
                file_part.metadata,
            ),
            types::Part::DataPart(data_part) => (
                ProtoPartEnum::Data(data_part.clone().into()),
                data_part.metadata,
            ),
        };

        proto::Part {
            part: Some(part_enum),
            metadata: Some(super::impl_task::json_map_to_struct(metadata)),
        }
    }
}

/// Convert from proto FilePart to internal FilePart
impl From<proto::FilePart> for types::FilePart {
    fn from(proto_file: proto::FilePart) -> Self {
        use proto::file_part::File;

        let file = match proto_file.file {
            Some(File::FileWithUri(uri)) => types::FilePartFile::Uri(types::FileWithUri {
                uri,
                mime_type: Some(proto_file.mime_type.clone()),
                name: None,
            }),
            Some(File::FileWithBytes(bytes)) => types::FilePartFile::Bytes(types::FileWithBytes {
                bytes: general_purpose::STANDARD.encode(bytes),
                mime_type: Some(proto_file.mime_type.clone()),
                name: None,
            }),
            None => {
                // Default to empty URI
                types::FilePartFile::Uri(types::FileWithUri {
                    uri: String::new(),
                    mime_type: Some(proto_file.mime_type.clone()),
                    name: None,
                })
            }
        };

        types::FilePart {
            kind: "file".to_string(),
            file,
            metadata: Default::default(),
        }
    }
}

/// Convert from internal FilePart to proto FilePart
impl From<types::FilePart> for proto::FilePart {
    fn from(file_part: types::FilePart) -> Self {
        use proto::file_part::File;

        let (file, mime_type, name) = match file_part.file {
            types::FilePartFile::Uri(uri_file) => (
                Some(File::FileWithUri(uri_file.uri)),
                uri_file.mime_type.unwrap_or_default(),
                uri_file.name,
            ),
            types::FilePartFile::Bytes(bytes_file) => {
                // Decode base64 string to bytes
                let decoded = general_purpose::STANDARD
                    .decode(bytes_file.bytes)
                    .ok()
                    .map(File::FileWithBytes);
                (
                    decoded,
                    bytes_file.mime_type.unwrap_or_default(),
                    bytes_file.name,
                )
            }
        };

        proto::FilePart {
            mime_type,
            file,
            name: name.unwrap_or_default(),
        }
    }
}

/// Convert from proto DataPart to internal DataPart
impl From<proto::DataPart> for types::DataPart {
    fn from(proto_data: proto::DataPart) -> Self {
        types::DataPart {
            kind: "data".to_string(),
            data: proto_data
                .data
                .map(super::impl_task::struct_to_json_map)
                .unwrap_or_default(),
            metadata: Default::default(),
        }
    }
}

/// Convert from internal DataPart to proto DataPart
impl From<types::DataPart> for proto::DataPart {
    fn from(data_part: types::DataPart) -> Self {
        proto::DataPart {
            data: if data_part.data.is_empty() {
                None
            } else {
                Some(super::impl_task::json_map_to_struct(data_part.data))
            },
        }
    }
}
