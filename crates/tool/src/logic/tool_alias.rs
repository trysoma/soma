use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared::error::CommonError;
use shared::primitives::{PaginatedResponse, PaginationRequest, WrappedChronoDateTime};
use utoipa::{IntoParams, ToSchema};

use crate::logic::ToolGroupDeploymentAliasSerialized;
use crate::repository::{CreateToolAlias, ProviderRepositoryLike};

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct CreateToolAliasRequest {
    pub tool_group_deployment_type_id: String,
    pub tool_group_deployment_deployment_id: String,
    pub alias: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct CreateToolAliasResponse {
    pub tool_alias: ToolGroupDeploymentAliasSerialized,
}

#[derive(Debug, Serialize, Deserialize, IntoParams, JsonSchema)]
#[into_params(style = Form, parameter_in = Query)]
pub struct ListToolAliasesParams {
    pub page_size: i64,
    pub next_page_token: Option<String>,
    pub tool_group_deployment_type_id: Option<String>,
    pub tool_group_deployment_deployment_id: Option<String>,
}

impl ListToolAliasesParams {
    pub fn pagination(&self) -> PaginationRequest {
        PaginationRequest {
            page_size: self.page_size,
            next_page_token: self.next_page_token.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListToolAliasesResponse {
    #[serde(flatten)]
    pub aliases: PaginatedResponse<ToolGroupDeploymentAliasSerialized>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, JsonSchema)]
pub struct UpdateToolAliasRequest {
    pub tool_group_deployment_deployment_id: String,
}

// ============================================================================
// Logic Functions
// ============================================================================

/// Create a tool alias
///
/// Creates an alias that points to a specific tool deployment
#[shared_macros::authz_role(Admin, permission = "tool:write")]
#[shared_macros::authn]
pub async fn create_tool_alias(
    repo: &impl ProviderRepositoryLike,
    request: CreateToolAliasRequest,
) -> Result<CreateToolAliasResponse, CommonError> {
    use tracing::trace;

    trace!(
        tool_group_deployment_type_id = %request.tool_group_deployment_type_id,
        tool_group_deployment_deployment_id = %request.tool_group_deployment_deployment_id,
        alias = %request.alias,
        "Creating tool alias"
    );

    let now = WrappedChronoDateTime::now();

    let tool_alias = ToolGroupDeploymentAliasSerialized {
        tool_group_deployment_type_id: request.tool_group_deployment_type_id,
        tool_group_deployment_deployment_id: request.tool_group_deployment_deployment_id,
        alias: request.alias,
        created_at: now.clone(),
        updated_at: now,
    };

    let create_params = CreateToolAlias::from(tool_alias.clone());
    repo.create_tool_group_deployment_alias(&create_params).await?;

    trace!(alias = %tool_alias.alias, "Tool alias created successfully");

    Ok(CreateToolAliasResponse { tool_alias })
}

/// List tool aliases
///
/// Returns a paginated list of tool aliases, optionally filtered by tool
#[shared_macros::authz_role(Admin, Maintainer, permission = "tool:read")]
#[shared_macros::authn]
pub async fn list_tool_aliases(
    repo: &impl ProviderRepositoryLike,
    params: ListToolAliasesParams,
) -> Result<ListToolAliasesResponse, CommonError> {
    use tracing::trace;

    trace!("Listing tool aliases");

    let pagination = params.pagination();
    let aliases = repo
        .list_tool_aliases(
            &pagination,
            params.tool_group_deployment_type_id.as_deref(),
            params.tool_group_deployment_deployment_id.as_deref(),
        )
        .await?;

    trace!(count = aliases.items.len(), "Tool aliases listed successfully");

    Ok(ListToolAliasesResponse { aliases })
}

/// Get tool by alias
///
/// Resolves an alias and returns the tool it points to
#[shared_macros::authz_role(Admin, Maintainer, permission = "tool:read")]
#[shared_macros::authn]
pub async fn get_tool_by_alias(
    repo: &impl ProviderRepositoryLike,
    alias: String,
) -> Result<crate::logic::ToolSerialized, CommonError> {
    use tracing::trace;

    trace!(alias = %alias, "Getting tool by alias");

    let tool = repo
        .get_tool_by_alias(&alias)
        .await?
        .ok_or_else(|| CommonError::NotFound {
            msg: format!("Alias not found: {}", alias),
            lookup_id: alias.clone(),
            source: None,
        })?;

    trace!(
        alias = %alias,
        tool_group_deployment_type_id = %tool.type_id,
        tool_group_deployment_deployment_id = %tool.deployment_id,
        "Tool resolved by alias successfully"
    );

    Ok(tool)
}

/// Update tool alias
///
/// Updates an alias to point to a different deployment
#[shared_macros::authz_role(Admin, permission = "tool:write")]
#[shared_macros::authn]
pub async fn update_tool_alias(
    repo: &impl ProviderRepositoryLike,
    tool_group_deployment_type_id: String,
    alias: String,
    new_deployment_id: String,
) -> Result<(), CommonError> {
    use tracing::trace;

    trace!(
        tool_group_deployment_type_id = %tool_type_id,
        alias = %alias,
        new_deployment_id = %new_deployment_id,
        "Updating tool alias"
    );

    repo.update_tool_group_deployment_alias(&tool_type_id, &alias, &new_deployment_id)
        .await?;

    trace!(
        tool_group_deployment_type_id = %tool_type_id,
        alias = %alias,
        "Tool alias updated successfully"
    );

    Ok(())
}

/// Delete tool alias
///
/// Removes an alias (does not delete the tool itself)
#[shared_macros::authz_role(Admin, permission = "tool:write")]
#[shared_macros::authn]
pub async fn delete_tool_alias(
    repo: &impl ProviderRepositoryLike,
    alias: String,
) -> Result<(), CommonError> {
    use tracing::trace;

    trace!(alias = %alias, "Deleting tool alias");

    repo.delete_tool_group_deployment_alias(&alias).await?;

    trace!(alias = %alias, "Tool alias deleted successfully");

    Ok(())
}
