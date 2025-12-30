use shared::error::CommonError;
use shared::primitives::PaginationRequest;
use tokio::sync::broadcast;
use tracing::{debug, error, trace, warn};

use crate::logic::on_change_pubsub::EnvironmentVariableChangeRx;
use crate::repository::EnvironmentVariableRepositoryLike;

/// An environment variable ready to be sent to the SDK
#[derive(Debug, Clone)]
pub struct EnvironmentVariableData {
    pub key: String,
    pub value: String,
}

/// Interpolate environment variable value with host environment variables.
///
/// Rules:
/// - If value starts with `$`, the rest is treated as an environment variable name
///   to look up from the host's environment (e.g., `$MY_VAR` -> value of `MY_VAR`)
/// - If value starts with `$$`, treat as literal `$` followed by the rest
///   (e.g., `$$MY_VAR` -> literal string `$MY_VAR`)
/// - Otherwise, return the value as-is
pub fn interpolate_env_value(value: &str) -> String {
    if let Some(stripped) = value.strip_prefix("$$") {
        // Escaped dollar sign - return literal $ + rest
        format!("${stripped}")
    } else if let Some(var_name) = value.strip_prefix('$') {
        // Environment variable reference - look up from host environment
        std::env::var(var_name).unwrap_or_default()
    } else {
        // No special prefix - return as-is
        value.to_string()
    }
}

/// Fetch all environment variables from the database
pub async fn fetch_all_environment_variables(
    repository: &std::sync::Arc<crate::repository::Repository>,
) -> Result<Vec<EnvironmentVariableData>, CommonError> {
    let mut all_env_vars = Vec::new();
    let mut page_token = None;

    // Paginate through all environment variables
    loop {
        let pagination = PaginationRequest {
            page_size: 100,
            next_page_token: page_token.clone(),
        };

        let page = repository
            .as_ref()
            .get_environment_variables(&pagination)
            .await?;

        for env_var in page.items {
            all_env_vars.push(EnvironmentVariableData {
                key: env_var.key,
                value: env_var.value,
            });
        }

        page_token = page.next_page_token;
        if page_token.is_none() {
            break;
        }
    }

    Ok(all_env_vars)
}

/// Sync environment variables to the SDK via gRPC (for initial sync - sends all env vars)
///
/// This function interpolates environment variable values before sending:
/// - Values starting with `$` are replaced with the host environment variable
/// - Values starting with `$$` become literal `$` + rest of string
pub async fn sync_environment_variables_to_sdk(
    sdk_client: &mut sdk_proto::soma_sdk_service_client::SomaSdkServiceClient<
        tonic::transport::Channel,
    >,
    env_vars: Vec<EnvironmentVariableData>,
) -> Result<(), CommonError> {
    let proto_env_vars: Vec<sdk_proto::EnvironmentVariable> = env_vars
        .into_iter()
        .map(|e| sdk_proto::EnvironmentVariable {
            key: e.key,
            value: interpolate_env_value(&e.value),
        })
        .collect();

    let request = tonic::Request::new(sdk_proto::SetEnvironmentVariablesRequest {
        environment_variables: proto_env_vars,
    });

    let response = sdk_client
        .set_environment_variables(request)
        .await
        .map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!(
                "Failed to call set_environment_variables RPC: {e}"
            ))
        })?;

    let inner = response.into_inner();

    match inner.kind {
        Some(sdk_proto::set_environment_variables_response::Kind::Data(_)) => {
            trace!("Environment variables synced to SDK");
            Ok(())
        }
        Some(sdk_proto::set_environment_variables_response::Kind::Error(error)) => {
            Err(CommonError::Unknown(anyhow::anyhow!(
                "SDK rejected environment variables: {}",
                error.message
            )))
        }
        None => Err(CommonError::Unknown(anyhow::anyhow!(
            "SDK rejected environment variables: unknown error"
        ))),
    }
}

/// Incrementally sync a single environment variable to the SDK via gRPC
pub async fn sync_environment_variable_to_sdk(
    sdk_client: &mut sdk_proto::soma_sdk_service_client::SomaSdkServiceClient<
        tonic::transport::Channel,
    >,
    key: String,
    value: String,
) -> Result<(), CommonError> {
    let request = tonic::Request::new(sdk_proto::SetEnvironmentVariablesRequest {
        environment_variables: vec![sdk_proto::EnvironmentVariable {
            key,
            value: interpolate_env_value(&value),
        }],
    });

    let response = sdk_client
        .set_environment_variables(request)
        .await
        .map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!(
                "Failed to call set_environment_variables RPC: {e}"
            ))
        })?;

    let inner = response.into_inner();

    match inner.kind {
        Some(sdk_proto::set_environment_variables_response::Kind::Data(_)) => {
            trace!("Environment variable synced to SDK");
            Ok(())
        }
        Some(sdk_proto::set_environment_variables_response::Kind::Error(error)) => {
            Err(CommonError::Unknown(anyhow::anyhow!(
                "SDK rejected environment variable: {}",
                error.message
            )))
        }
        None => Err(CommonError::Unknown(anyhow::anyhow!(
            "SDK rejected environment variable: unknown error"
        ))),
    }
}

/// Unset an environment variable in the SDK via gRPC
pub async fn unset_environment_variable_in_sdk(
    sdk_client: &mut sdk_proto::soma_sdk_service_client::SomaSdkServiceClient<
        tonic::transport::Channel,
    >,
    key: String,
) -> Result<(), CommonError> {
    let request = tonic::Request::new(sdk_proto::UnsetEnvironmentVariableRequest { key });

    let response = sdk_client
        .unset_environment_variables(request)
        .await
        .map_err(|e| {
            CommonError::Unknown(anyhow::anyhow!(
                "Failed to call unset_environment_variables RPC: {e}"
            ))
        })?;

    let inner = response.into_inner();

    match inner.kind {
        Some(sdk_proto::unset_environment_variable_response::Kind::Data(_)) => {
            trace!("Unset environment variable in SDK");
            Ok(())
        }
        Some(sdk_proto::unset_environment_variable_response::Kind::Error(error)) => {
            Err(CommonError::Unknown(anyhow::anyhow!(
                "SDK rejected unset environment variable: {}",
                error.message
            )))
        }
        None => Err(CommonError::Unknown(anyhow::anyhow!(
            "SDK rejected unset environment variable: unknown error"
        ))),
    }
}

pub struct EnvironmentVariableSyncParams {
    pub repository: std::sync::Arc<crate::repository::Repository>,
    pub socket_path: String,
    pub environment_variable_change_rx: EnvironmentVariableChangeRx,
}

/// Run the environment variable sync loop - listens for env var changes and syncs to SDK.
/// This function runs indefinitely until aborted by the process manager.
pub async fn run_environment_variable_sync_loop(
    params: EnvironmentVariableSyncParams,
) -> Result<(), CommonError> {
    let EnvironmentVariableSyncParams {
        repository,
        socket_path,
        mut environment_variable_change_rx,
    } = params;
    let repository = repository.clone();

    debug!("Environment variable sync loop started");

    loop {
        match environment_variable_change_rx.recv().await {
            Ok(evt) => {
                trace!(event = ?evt, "Environment variable change event");

                // On any env var change, re-sync all env vars
                // This is simpler than tracking individual changes and ensures consistency
                match sync_all_environment_variables(&repository, &socket_path).await {
                    Ok(()) => {
                        trace!("Environment variables re-synced");
                    }
                    Err(e) => {
                        error!(error = ?e, "Failed to re-sync environment variables");
                    }
                }
            }
            Err(broadcast::error::RecvError::Closed) => {
                debug!("Environment variable change channel closed");
                break;
            }
            Err(broadcast::error::RecvError::Lagged(skipped)) => {
                warn!(
                    skipped,
                    "Environment variable change channel lagged, re-syncing"
                );
                // Re-sync all env vars to ensure we're in a consistent state
                if let Err(e) = sync_all_environment_variables(&repository, &socket_path).await {
                    error!(error = ?e, "Failed to re-sync environment variables after lag");
                }
            }
        }
    }

    Ok(())
}

/// Helper to sync all environment variables to SDK
async fn sync_all_environment_variables(
    repository: &std::sync::Arc<crate::repository::Repository>,
    socket_path: &str,
) -> Result<(), CommonError> {
    // Fetch all environment variables
    let env_vars = fetch_all_environment_variables(repository).await?;
    trace!(
        count = env_vars.len(),
        "Syncing environment variables to SDK"
    );

    // Connect to SDK and sync
    let mut client = shared::uds::create_soma_unix_socket_client(socket_path).await?;
    sync_environment_variables_to_sdk(&mut client, env_vars).await
}

#[cfg(test)]
mod tests {
    mod unit {
        use super::super::*;

        #[test]
        fn test_interpolate_env_value_passthrough() {
            // Set a test environment variable
            // SAFETY: This is a test that runs in isolation
            unsafe {
                std::env::set_var("TEST_PASSTHROUGH_VAR", "hello_from_host");
            }

            // Value starting with $ should be interpolated from host env
            let result = interpolate_env_value("$TEST_PASSTHROUGH_VAR");
            assert_eq!(result, "hello_from_host");

            // Clean up
            // SAFETY: This is a test that runs in isolation
            unsafe {
                std::env::remove_var("TEST_PASSTHROUGH_VAR");
            }
        }

        #[test]
        fn test_interpolate_env_value_passthrough_missing_var() {
            // Ensure the variable doesn't exist
            // SAFETY: This is a test that runs in isolation
            unsafe {
                std::env::remove_var("NON_EXISTENT_TEST_VAR");
            }

            // Missing env var should return empty string
            let result = interpolate_env_value("$NON_EXISTENT_TEST_VAR");
            assert_eq!(result, "");
        }

        #[test]
        fn test_interpolate_env_value_escaped_dollar() {
            // Value starting with $$ should become literal $
            let result = interpolate_env_value("$$MY_VAR");
            assert_eq!(result, "$MY_VAR");
        }

        #[test]
        fn test_interpolate_env_value_escaped_empty() {
            // Just $$ should become just $
            let result = interpolate_env_value("$$");
            assert_eq!(result, "$");
        }

        #[test]
        fn test_interpolate_env_value_literal() {
            // Regular value without $ prefix should remain unchanged
            let result = interpolate_env_value("regular_value");
            assert_eq!(result, "regular_value");
        }

        #[test]
        fn test_interpolate_env_value_empty() {
            // Empty string should remain empty
            let result = interpolate_env_value("");
            assert_eq!(result, "");
        }

        #[test]
        fn test_interpolate_env_value_dollar_in_middle() {
            // Dollar sign in middle of string should not be interpolated
            let result = interpolate_env_value("prefix$VAR");
            assert_eq!(result, "prefix$VAR");
        }

        #[test]
        fn test_interpolate_env_value_triple_dollar() {
            // $$$ should become $$ (escape first two, third remains)
            // Actually: $$ -> $, then the third $ is part of the result
            let result = interpolate_env_value("$$$VAR");
            assert_eq!(result, "$$VAR");
        }

        #[test]
        fn test_interpolate_env_value_just_dollar() {
            // Just $ alone - var name is empty, should return empty string
            let result = interpolate_env_value("$");
            // std::env::var("") returns Err, so unwrap_or_default returns ""
            assert_eq!(result, "");
        }
    }
}
