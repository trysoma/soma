//! Authorization utilities for ReBAC (Relationship-Based Access Control)
//!
//! This module provides types and macros for implementing authorization checks
//! beyond simple role-based access control.

use crate::error::CommonError;
use crate::identity::{Identity, Role};

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
pub fn check_rebac(
    identity: &Identity,
    action: &str,
    entity: &impl AuthzEntity,
) -> AuthzResult {
    // Get identity role and groups
    let identity_role = match identity.role() {
        Some(role) => role,
        None => {
            return AuthzResult::denied("No role found for identity");
        }
    };

    let identity_groups = match identity {
        Identity::Human(h) => &h.groups,
        Identity::MachineOnBehalfOfHuman { human, .. } => &human.groups,
        Identity::Machine(_) => &vec![] as &Vec<String>,
        Identity::Unauthenticated => {
            return AuthzResult::denied("Unauthenticated identity");
        }
    };

    let allowed_roles = entity.allowed_roles();
    let allowed_groups = entity.allowed_groups();

    // Check role-based access
    if !allowed_roles.is_empty() && allowed_roles.contains(identity_role) {
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
    pub id: String,
    pub allowed_groups: Vec<String>,
    pub allowed_roles: Vec<Role>,
}

impl SimpleEntity {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            allowed_groups: vec![],
            allowed_roles: vec![],
        }
    }

    pub fn with_groups(mut self, groups: Vec<String>) -> Self {
        self.allowed_groups = groups;
        self
    }

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
    use crate::identity::{Human, Machine};

    #[test]
    fn test_rebac_role_allowed() {
        let identity = Identity::Machine(Machine {
            sub: "machine-1".to_string(),
            role: Role::Admin,
        });

        let entity = SimpleEntity::new("entity-1").with_roles(vec![Role::Admin, Role::Agent]);

        let result = check_rebac(&identity, "read", &entity);
        assert!(result.allowed);
    }

    #[test]
    fn test_rebac_role_denied() {
        let identity = Identity::Machine(Machine {
            sub: "machine-1".to_string(),
            role: Role::User,
        });

        let entity = SimpleEntity::new("entity-1").with_roles(vec![Role::Admin]);

        let result = check_rebac(&identity, "read", &entity);
        assert!(!result.allowed);
    }

    #[test]
    fn test_rebac_group_allowed() {
        let identity = Identity::Human(Human {
            sub: "user-1".to_string(),
            email: Some("user@example.com".to_string()),
            groups: vec!["finance".to_string(), "hr".to_string()],
            role: Role::User,
        });

        let entity = SimpleEntity::new("entity-1").with_groups(vec!["finance".to_string()]);

        let result = check_rebac(&identity, "read", &entity);
        assert!(result.allowed);
    }

    #[test]
    fn test_rebac_group_denied() {
        let identity = Identity::Human(Human {
            sub: "user-1".to_string(),
            email: Some("user@example.com".to_string()),
            groups: vec!["engineering".to_string()],
            role: Role::User,
        });

        let entity = SimpleEntity::new("entity-1").with_groups(vec!["finance".to_string()]);

        let result = check_rebac(&identity, "read", &entity);
        assert!(!result.allowed);
    }

    #[test]
    fn test_rebac_no_restrictions() {
        let identity = Identity::Human(Human {
            sub: "user-1".to_string(),
            email: Some("user@example.com".to_string()),
            groups: vec![],
            role: Role::User,
        });

        let entity = SimpleEntity::new("entity-1");

        let result = check_rebac(&identity, "read", &entity);
        assert!(result.allowed);
    }

    #[test]
    fn test_authz_rebac_macro() {
        let identity = Identity::Machine(Machine {
            sub: "machine-1".to_string(),
            role: Role::Admin,
        });

        let entity = SimpleEntity::new("entity-1").with_roles(vec![Role::Admin]);

        let result = authz_rebac!(&identity, "read", &entity);
        assert!(result.is_ok());
    }
}
