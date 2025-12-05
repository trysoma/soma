use shared::{
    error::CommonError,
    primitives::{PaginationRequest, WrappedChronoDateTime},
};

use crate::repository::{Group, GroupMembership, UserRepositoryLike};

/// Sync user's group memberships - add new groups, remove old ones
pub async fn sync_user_groups<R: UserRepositoryLike>(
    repository: &R,
    user_id: &str,
    groups: &[String],
) -> Result<(), CommonError> {
    let now = WrappedChronoDateTime::now();

    // Get all current group memberships by paginating through all pages
    let mut all_memberships = Vec::new();
    let mut next_page_token = None;
    loop {
        let pagination = PaginationRequest {
            page_size: 1000,
            next_page_token,
        };
        let result = repository.list_user_groups(user_id, &pagination).await?;
        all_memberships.extend(result.items);
        if result.next_page_token.is_none() {
            break;
        }
        next_page_token = result.next_page_token;
    }
    let current_group_ids: std::collections::HashSet<String> =
        all_memberships.iter().map(|m| m.group.id.clone()).collect();

    let desired_group_ids: std::collections::HashSet<String> = groups.iter().cloned().collect();

    // Add memberships to new groups
    for group_id in desired_group_ids.difference(&current_group_ids) {
        // Ensure group exists (using standardized name as both ID and name)
        if repository.get_group_by_id(group_id).await?.is_none() {
            let group = Group {
                id: group_id.clone(),
                name: group_id.clone(), // Use standardized name
                created_at: now,
                updated_at: now,
            };
            repository.create_group(&group).await?;
        }

        // Create membership
        let create_membership = GroupMembership {
            group_id: group_id.clone(),
            user_id: user_id.to_string(),
            created_at: now,
            updated_at: now,
        };
        repository
            .create_group_membership(&create_membership)
            .await?;
    }

    // Remove memberships from groups no longer in the token
    for group_id in current_group_ids.difference(&desired_group_ids) {
        repository
            .delete_group_membership(group_id, user_id)
            .await?;
    }

    Ok(())
}
