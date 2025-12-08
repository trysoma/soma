"""Soma Python SDK - Build AI agents with ease."""

from trysoma_sdk.agent import SomaAgent, create_soma_agent, HandlerParams
from trysoma_sdk.bridge import create_soma_function, SomaFunction
from trysoma_sdk.patterns import patterns
from trysoma_sdk.standalone import generate_standalone, watch_and_regenerate

# Re-export core types from trysoma_sdk_core for convenience
from trysoma_sdk_core import (
    # Types
    Agent,
    ProviderController,
    ProviderCredentialController,
    FunctionController,
    FunctionMetadata,
    Metadata,
    Oauth2AuthorizationCodeFlowConfiguration,
    Oauth2AuthorizationCodeFlowStaticCredentialConfiguration,
    Oauth2JwtBearerAssertionFlowConfiguration,
    Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration,
    InvokeFunctionRequest,
    InvokeFunctionResponse,
    CallbackError,
    Secret,
    EnvironmentVariable,
    SetSecretsResponse,
    SetSecretsSuccess,
    SetEnvironmentVariablesResponse,
    SetEnvironmentVariablesSuccess,
    UnsetSecretResponse,
    UnsetSecretSuccess,
    UnsetEnvironmentVariableResponse,
    UnsetEnvironmentVariableSuccess,
    # Functions
    start_grpc_server,
    kill_grpc_service,
    add_provider,
    remove_provider,
    update_provider,
    remove_function,
    update_function,
    add_agent,
    remove_agent,
    update_agent,
    set_secret_handler,
    set_environment_variable_handler,
    set_unset_secret_handler,
    set_unset_environment_variable_handler,
    resync_sdk,
)

__version__ = "0.0.4"

__all__ = [
    # High-level API
    "SomaAgent",
    "create_soma_agent",
    "HandlerParams",
    "SomaFunction",
    "create_soma_function",
    "patterns",
    "generate_standalone",
    "watch_and_regenerate",
    # Core types (re-exported from trysoma_sdk_core)
    "Agent",
    "ProviderController",
    "ProviderCredentialController",
    "FunctionController",
    "FunctionMetadata",
    "Metadata",
    "Oauth2AuthorizationCodeFlowConfiguration",
    "Oauth2AuthorizationCodeFlowStaticCredentialConfiguration",
    "Oauth2JwtBearerAssertionFlowConfiguration",
    "Oauth2JwtBearerAssertionFlowStaticCredentialConfiguration",
    "InvokeFunctionRequest",
    "InvokeFunctionResponse",
    "CallbackError",
    "Secret",
    "EnvironmentVariable",
    "SetSecretsResponse",
    "SetSecretsSuccess",
    "SetEnvironmentVariablesResponse",
    "SetEnvironmentVariablesSuccess",
    "UnsetSecretResponse",
    "UnsetSecretSuccess",
    "UnsetEnvironmentVariableResponse",
    "UnsetEnvironmentVariableSuccess",
    # Core functions (re-exported from trysoma_sdk_core)
    "start_grpc_server",
    "kill_grpc_service",
    "add_provider",
    "remove_provider",
    "update_provider",
    "remove_function",
    "update_function",
    "add_agent",
    "remove_agent",
    "update_agent",
    "set_secret_handler",
    "set_environment_variable_handler",
    "set_unset_secret_handler",
    "set_unset_environment_variable_handler",
    "resync_sdk",
]
