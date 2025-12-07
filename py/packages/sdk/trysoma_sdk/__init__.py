"""Soma Python SDK - Build AI agents with ease."""

from trysoma_sdk.agent import SomaAgent, create_soma_agent, HandlerParams
from trysoma_sdk.bridge import create_soma_function
from trysoma_sdk.patterns import patterns
from trysoma_sdk.standalone import generate_standalone, watch_and_regenerate

# Re-export core types from the native bindings
try:
    from trysoma_sdk_core import (
        Agent,
        ProviderController,
        FunctionController,
        CredentialController,
        Secret,
        EnvironmentVariable,
        SetSecretsResponse,
        SetSecretsSuccess,
        CallbackError,
        SetEnvironmentVariablesResponse,
        SetEnvironmentVariablesSuccess,
        UnsetSecretResponse,
        UnsetSecretSuccess,
        UnsetEnvironmentVariableResponse,
        UnsetEnvironmentVariableSuccess,
        FunctionInvokeRequest,
        FunctionInvokeResponse,
        start_grpc_server,
        add_provider,
        add_function,
        add_agent,
        set_secret_handler,
        set_environment_variable_handler,
        set_unset_secret_handler,
        set_unset_environment_variable_handler,
        resync_sdk,
    )
except ImportError:
    # Native bindings not available (e.g., during documentation builds)
    pass

__version__ = "0.0.4"

__all__ = [
    # High-level API
    "SomaAgent",
    "create_soma_agent",
    "HandlerParams",
    "create_soma_function",
    "patterns",
    "generate_standalone",
    "watch_and_regenerate",
    # Core types
    "Agent",
    "ProviderController",
    "FunctionController",
    "CredentialController",
    "Secret",
    "EnvironmentVariable",
    "SetSecretsResponse",
    "SetSecretsSuccess",
    "CallbackError",
    "SetEnvironmentVariablesResponse",
    "SetEnvironmentVariablesSuccess",
    "UnsetSecretResponse",
    "UnsetSecretSuccess",
    "UnsetEnvironmentVariableResponse",
    "UnsetEnvironmentVariableSuccess",
    "FunctionInvokeRequest",
    "FunctionInvokeResponse",
    # Low-level functions
    "start_grpc_server",
    "add_provider",
    "add_function",
    "add_agent",
    "set_secret_handler",
    "set_environment_variable_handler",
    "set_unset_secret_handler",
    "set_unset_environment_variable_handler",
    "resync_sdk",
]
