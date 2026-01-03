use crate::adapters::grpc::proto;
use crate::types;

/// Convert from proto Artifact to internal Artifact
impl From<proto::Artifact> for types::Artifact {
    fn from(proto_artifact: proto::Artifact) -> Self {
        types::Artifact {
            artifact_id: proto_artifact.artifact_id,
            name: Some(proto_artifact.name),
            description: if proto_artifact.description.is_empty() {
                None
            } else {
                Some(proto_artifact.description)
            },
            // kind: "artifact".to_string(), // This field doesn't exist in proto, using default
            parts: proto_artifact.parts.into_iter().map(|p| p.into()).collect(),
            extensions: proto_artifact.extensions,
            metadata: proto_artifact
                .metadata
                .map(super::impl_task::struct_to_json_map)
                .unwrap_or_default(),
        }
    }
}

/// Convert from internal Artifact to proto Artifact
impl From<types::Artifact> for proto::Artifact {
    fn from(artifact: types::Artifact) -> Self {
        proto::Artifact {
            artifact_id: artifact.artifact_id,
            name: artifact.name.unwrap_or_default(),
            description: artifact.description.unwrap_or_default(),
            parts: artifact.parts.into_iter().map(|p| p.into()).collect(),
            metadata: if artifact.metadata.is_empty() {
                None
            } else {
                Some(super::impl_task::json_map_to_struct(artifact.metadata))
            },
            extensions: artifact.extensions,
        }
    }
}
