//! Authorization utilities for ReBAC (Relationship-Based Access Control)
//!
//! This module provides types and macros for implementing authorization checks
//! beyond simple role-based access control.

use crate::error::CommonError;
use crate::identity::{Identity, Role};
use tracing::{debug, trace};

/// Entity that can be accessed with group-based permissions
pub trait AuthzEntity {
    /// Get the ID of the entity
    fn entity_id(&self) -> &str;

    /// Get the allowed groups for this entity (empty = all groups allowed)
    fn allowed_groups(&self) -> &[String];

    /// Get the allowed roles for this entity (empty = check default roles)
    fn allowed_roles(&self) -> &[Role];
}

/// Result of a ReBAC authorization check
#[derive(Debug)]
pub struct AuthzResult {
    pub allowed: bool,
    pub reason: String,
}

impl AuthzResult {
    pub fn allowed(reason: impl Into<String>) -> Self {
        Self {
            allowed: true,
            reason: reason.into(),
        }
    }

    pub fn denied(reason: impl Into<String>) -> Self {
        Self {
            allowed: false,
            reason: reason.into(),
        }
    }
}

/// Check ReBAC authorization for an identity accessing an entity
///
/// This function implements the following logic:
/// 1. If the entity has allowed_roles and the identity's role is in that list, allow
/// 2. If the entity has allowed_groups:
///    - If empty, allow (no group restriction)
///    - If non-empty, check if identity belongs to at least one allowed group
/// 3. Otherwise deny
pub fn check_rebac(identity: &Identity, action: &str, entity: &impl AuthzEntity) -> AuthzResult {
    trace!(
        action = %action,
        entity_id = %entity.entity_id(),
        "Checking ReBAC authorization"
    );

    // Get identity role and groups
    let identity_role = match identity.role() {
        Some(role) => role,
        None => {
            debug!(action = %action, entity_id = %entity.entity_id(), "Authorization denied: no role");
            return AuthzResult::denied("No role found for identity");
        }
    };

    let identity_groups = match identity {
        Identity::Human(h) => &h.groups,
        Identity::MachineOnBehalfOfHuman { human, .. } => &human.groups,
        Identity::Machine(_) => &vec![] as &Vec<String>,
        Identity::Unauthenticated => {
            debug!(action = %action, entity_id = %entity.entity_id(), "Authorization denied: unauthenticated");
            return AuthzResult::denied("Unauthenticated identity");
        }
    };

    let allowed_roles = entity.allowed_roles();
    let allowed_groups = entity.allowed_groups();

    trace!(
        role = %identity_role.as_str(),
        groups_count = identity_groups.len(),
        allowed_roles_count = allowed_roles.len(),
        allowed_groups_count = allowed_groups.len(),
        "Evaluating authorization constraints"
    );

    // Check role-based access
    if !allowed_roles.is_empty() && allowed_roles.contains(identity_role) {
        debug!(
            role = %identity_role.as_str(),
            action = %action,
            entity_id = %entity.entity_id(),
            "Authorization granted via role"
        );
        return AuthzResult::allowed(format!(
            "Role '{}' is allowed for action '{}' on entity '{}'",
            identity_role.as_str(),
            action,
            entity.entity_id()
        ));
    }

    // Check group-based access
    if allowed_groups.is_empty() {
        // No group restriction - allow if role check passed or no role restriction
        if allowed_roles.is_empty() {
            debug!(
                action = %action,
                entity_id = %entity.entity_id(),
                "Authorization granted: no restrictions"
            );
            return AuthzResult::allowed(format!(
                "No restrictions on entity '{}' for action '{}'",
                entity.entity_id(),
                action
            ));
        }
    } else {
        // Check if identity belongs to at least one allowed group
        for group in identity_groups {
            if allowed_groups.contains(group) {
                debug!(
                    group = %group,
                    action = %action,
                    entity_id = %entity.entity_id(),
                    "Authorization granted via group"
                );
                return AuthzResult::allowed(format!(
                    "Group '{}' is allowed for action '{}' on entity '{}'",
                    group,
                    action,
                    entity.entity_id()
                ));
            }
        }
    }

    // Default deny
    debug!(
        action = %action,
        entity_id = %entity.entity_id(),
        role = %identity_role.as_str(),
        "Authorization denied: no matching role or group"
    );
    AuthzResult::denied(format!(
        "Access denied for action '{}' on entity '{}'. Required groups: {:?}, identity groups: {:?}",
        action,
        entity.entity_id(),
        allowed_groups,
        identity_groups
    ))
}

/// Check ReBAC authorization and return an error if denied
pub fn require_rebac(
    identity: &Identity,
    action: &str,
    entity: &impl AuthzEntity,
) -> Result<(), CommonError> {
    let result = check_rebac(identity, action, entity);
    if result.allowed {
        Ok(())
    } else {
        Err(CommonError::Authorization {
            msg: result.reason,
            source: anyhow::anyhow!("ReBAC authorization failed"),
        })
    }
}

/// Simple entity wrapper for inline ReBAC checks
pub struct SimpleEntity {
    /// Unique identifier for this entity
    pub id: String,
    /// List of groups allowed to access this entity
    pub allowed_groups: Vec<String>,
    /// List of roles allowed to access this entity
    pub allowed_roles: Vec<Role>,
}

impl SimpleEntity {
    /// Create a new SimpleEntity with no restrictions
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the entity
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            allowed_groups: vec![],
            allowed_roles: vec![],
        }
    }

    /// Set the allowed groups for this entity (builder pattern)
    ///
    /// # Arguments
    ///
    /// * `groups` - List of group names that can access this entity
    pub fn with_groups(mut self, groups: Vec<String>) -> Self {
        self.allowed_groups = groups;
        self
    }

    /// Set the allowed roles for this entity (builder pattern)
    ///
    /// # Arguments
    ///
    /// * `roles` - List of roles that can access this entity
    pub fn with_roles(mut self, roles: Vec<Role>) -> Self {
        self.allowed_roles = roles;
        self
    }
}

impl AuthzEntity for SimpleEntity {
    fn entity_id(&self) -> &str {
        &self.id
    }

    fn allowed_groups(&self) -> &[String] {
        &self.allowed_groups
    }

    fn allowed_roles(&self) -> &[Role] {
        &self.allowed_roles
    }
}

/// Macro for inline ReBAC authorization checks
///
/// This macro provides a convenient way to perform ReBAC authorization checks
/// within function bodies where you have runtime context available.
///
/// # Usage
///
/// ```rust,ignore
/// use shared::authz_rebac;
///
/// // Basic usage with entity that implements AuthzEntity
/// authz_rebac!(identity, "invokeFunction", &function_instance)?;
///
/// // With inline entity construction
/// authz_rebac!(identity, "mcpConnect", SimpleEntity::new("mcp-123")
///     .with_groups(vec!["finance".to_string()])
///     .with_roles(vec![Role::Admin, Role::Agent]))?;
/// ```
///
/// # Returns
///
/// Returns `Result<(), CommonError>` - `Ok(())` if authorized, `Err(CommonError::Authorization)` if denied.
#[macro_export]
macro_rules! authz_rebac {
    ($identity:expr, $action:expr, $entity:expr) => {
        $crate::authz::require_rebac($identity, $action, $entity)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::helpers::{
        test_admin_machine, test_human_with_groups, test_user_human, test_user_machine,
    };

    #[test]
    fn test_rebac_role_allowed() {
        let identity = test_admin_machine();
        let entity = SimpleEntity::new("entity-1").with_roles(vec![Role::Admin, Role::Agent]);

        let result = check_rebac(&identity, "user:read", &entity);
        assert!(result.allowed);
    }

    #[test]
    fn test_rebac_role_denied() {
        let identity = test_user_machine();
        let entity = SimpleEntity::new("entity-1").with_roles(vec![Role::Admin]);

        let result = check_rebac(&identity, "user:write", &entity);
        assert!(!result.allowed);
    }

    #[test]
    fn test_rebac_group_allowed() {
        let identity = test_human_with_groups(vec!["finance".to_string(), "hr".to_string()]);
        let entity = SimpleEntity::new("entity-1").with_groups(vec!["finance".to_string()]);

        let result = check_rebac(&identity, "report:read", &entity);
        assert!(result.allowed);
    }

    #[test]
    fn test_rebac_group_denied() {
        let identity = test_human_with_groups(vec!["engineering".to_string()]);
        let entity = SimpleEntity::new("entity-1").with_groups(vec!["finance".to_string()]);

        let result = check_rebac(&identity, "report:read", &entity);
        assert!(!result.allowed);
    }

    #[test]
    fn test_rebac_no_restrictions() {
        let identity = test_user_human();
        let entity = SimpleEntity::new("entity-1");

        let result = check_rebac(&identity, "public:read", &entity);
        assert!(result.allowed);
    }

    #[test]
    fn test_authz_rebac_macro() {
        let identity = test_admin_machine();
        let entity = SimpleEntity::new("entity-1").with_roles(vec![Role::Admin]);

        let result = authz_rebac!(&identity, "config:write", &entity);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unauthenticated_denied() {
        let identity = Identity::Unauthenticated;
        let entity = SimpleEntity::new("entity-1");

        let result = check_rebac(&identity, "any:action", &entity);
        assert!(!result.allowed);
        // Unauthenticated has no role, so the first check fails
        assert!(result.reason.contains("No role found"));
    }
}
