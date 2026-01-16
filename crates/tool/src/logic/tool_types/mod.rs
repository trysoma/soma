pub mod http;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared::primitives::{WrappedChronoDateTime, WrappedJsonValue};
use utoipa::ToSchema;

use super::Metadata;

/// Type of endpoint for tool execution
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EndpointType {
    /// HTTP endpoint that accepts POST requests
    Http,
}

impl EndpointType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EndpointType::Http => "http",
        }
    }
}

impl std::fmt::Display for EndpointType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<String> for EndpointType {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "http" => Ok(EndpointType::Http),
            _ => Err(format!("Invalid endpoint type: {}", s)),
        }
    }
}

impl TryFrom<&str> for EndpointType {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "http" => Ok(EndpointType::Http),
            _ => Err(format!("Invalid endpoint type: {}", s)),
        }
    }
}

// Implement From<EndpointType> for libsql::Value for database storage
impl From<EndpointType> for libsql::Value {
    fn from(endpoint_type: EndpointType) -> Self {
        libsql::Value::Text(endpoint_type.as_str().to_string())
    }
}

// Implement FromValue for EndpointType for database retrieval
impl libsql::FromValue for EndpointType {
    fn from_sql(value: libsql::Value) -> libsql::Result<Self> {
        match value {
            libsql::Value::Text(s) => EndpointType::try_from(s.as_str())
                .map_err(|_e| libsql::Error::InvalidColumnType),
            libsql::Value::Null => Err(libsql::Error::NullValue),
            _ => Err(libsql::Error::InvalidColumnType),
        }
    }
}

/// Configuration for HTTP-based tool endpoints
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct HttpEndpointConfiguration {
    /// URL of the HTTP endpoint
    pub url: String,
    /// Encrypted invocation key used for authentication
    pub invocation_key: String,
}

/// Tool definition with type and deployment identifiers
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct ToolGroupDeploymentSerialized {
    /// Unique type identifier for this tool (e.g., "weather-api")
    pub type_id: String,
    /// Deployment identifier (e.g., "v1.0.0", "production")
    pub deployment_id: String,
    /// Display name of the tool
    pub name: String,
    /// Documentation describing the tool's purpose and usage
    pub documentation: String,
    /// Categories this tool belongs to (for filtering and discovery)
    pub categories: Vec<String>,
    /// Type of endpoint (http)
    pub endpoint_type: EndpointType,
    /// Encrypted endpoint configuration
    #[schema(value_type = Object)]
    #[schemars(skip)]
    pub endpoint_configuration: WrappedJsonValue,
    /// Additional metadata
    pub metadata: Metadata,
    /// When this tool was registered
    pub created_at: WrappedChronoDateTime,
    /// When this tool was last updated
    pub updated_at: WrappedChronoDateTime,
}

/// Tool alias for version management
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct ToolGroupDeploymentAliasSerialized {
    /// Type ID of the tool this alias points to
    pub tool_type_id: String,
    /// Deployment ID of the tool this alias points to
    pub tool_deployment_id: String,
    /// Alias name (e.g., "latest", "stable", "beta")
    pub alias: String,
    /// When this alias was created
    pub created_at: WrappedChronoDateTime,
    /// When this alias was last updated
    pub updated_at: WrappedChronoDateTime,
}

// ============================================================================
// Tool API Request/Response Types
// ============================================================================

/// Request to register a new tool
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct RegisterToolRequest {
    pub type_id: String,
    pub deployment_id: String,
    pub name: String,
    pub documentation: String,
    #[serde(default)]
    pub categories: Vec<String>,
    pub endpoint_configuration: HttpEndpointConfiguration,
    #[serde(default)]
    pub metadata: Option<super::Metadata>,
}

/// Response after registering a tool
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct RegisterToolResponse {
    pub tool: ToolSerialized,
}

#[cfg(test)]
mod tests {
    mod unit {
        use super::super::*;

        #[test]
        fn test_endpoint_type_serialization() {
            // Test HTTP variant
            let http = EndpointType::Http;
            let json = serde_json::to_string(&http).unwrap();
            assert_eq!(json, "\"http\"");
            let deserialized: EndpointType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, EndpointType::Http);
        }

        #[test]
        fn test_endpoint_type_from_string() {
            assert_eq!(
                EndpointType::try_from("http".to_string()).unwrap(),
                EndpointType::Http
            );
            assert!(EndpointType::try_from("invalid".to_string()).is_err());
        }

        #[test]
        fn test_endpoint_type_to_string() {
            assert_eq!(EndpointType::Http.to_string(), "http");
        }

        #[test]
        fn test_http_endpoint_configuration_serialization() {
            let config = HttpEndpointConfiguration {
                url: "https://example.com/api/tool".to_string(),
                invocation_key: "encrypted_key_123".to_string(),
            };
            let json = serde_json::to_string(&config).unwrap();
            let deserialized: HttpEndpointConfiguration = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized.url, config.url);
            assert_eq!(deserialized.invocation_key, config.invocation_key);
        }
    }
}
