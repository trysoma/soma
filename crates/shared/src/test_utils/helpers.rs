use http::HeaderMap;
use std::sync::Once;

use crate::error::CommonError;
use crate::identity::{AuthClientLike, Human, Identity, Machine, RawCredentials, Role};

pub fn get_workspace_root() -> String {
    let crate_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_root
        .clone()
        .parent() // up from src/
        .unwrap()
        .parent() // up from identity-service/
        .unwrap()
        .to_string_lossy()
        .to_string()
}

// ============================================================================
// Test Identity Helpers
// ============================================================================

/// Create a test admin machine identity
pub fn test_admin_machine() -> Identity {
    Identity::Machine(Machine {
        sub: "test-admin-machine".to_string(),
        role: Role::Admin,
    })
}

/// Create a test maintainer machine identity
pub fn test_maintainer_machine() -> Identity {
    Identity::Machine(Machine {
        sub: "test-maintainer-machine".to_string(),
        role: Role::Maintainer,
    })
}

/// Create a test agent machine identity
pub fn test_agent_machine() -> Identity {
    Identity::Machine(Machine {
        sub: "test-agent-machine".to_string(),
        role: Role::Agent,
    })
}

/// Create a test user machine identity
pub fn test_user_machine() -> Identity {
    Identity::Machine(Machine {
        sub: "test-user-machine".to_string(),
        role: Role::User,
    })
}

/// Create a test admin human identity
pub fn test_admin_human() -> Identity {
    Identity::Human(Human {
        sub: "test-admin-human".to_string(),
        email: Some("admin@test.com".to_string()),
        groups: vec!["admins".to_string()],
        role: Role::Admin,
    })
}

/// Create a test maintainer human identity
pub fn test_maintainer_human() -> Identity {
    Identity::Human(Human {
        sub: "test-maintainer-human".to_string(),
        email: Some("maintainer@test.com".to_string()),
        groups: vec!["maintainers".to_string()],
        role: Role::Maintainer,
    })
}

/// Create a test agent human identity
pub fn test_agent_human() -> Identity {
    Identity::Human(Human {
        sub: "test-agent-human".to_string(),
        email: Some("agent@test.com".to_string()),
        groups: vec!["agents".to_string()],
        role: Role::Agent,
    })
}

/// Create a test user human identity
pub fn test_user_human() -> Identity {
    Identity::Human(Human {
        sub: "test-user-human".to_string(),
        email: Some("user@test.com".to_string()),
        groups: vec!["users".to_string()],
        role: Role::User,
    })
}

/// Create a test human identity with specific groups
pub fn test_human_with_groups(groups: Vec<String>) -> Identity {
    Identity::Human(Human {
        sub: "test-human".to_string(),
        email: Some("test@test.com".to_string()),
        groups,
        role: Role::User,
    })
}

/// Create a test human identity with specific role and groups
pub fn test_human_with_role_and_groups(role: Role, groups: Vec<String>) -> Identity {
    Identity::Human(Human {
        sub: format!("test-{}-human", role.as_str()),
        email: Some(format!("{}@test.com", role.as_str())),
        groups,
        role,
    })
}

/// Create a test machine on behalf of human identity
pub fn test_machine_on_behalf_of_human(machine_role: Role, human_role: Role) -> Identity {
    Identity::MachineOnBehalfOfHuman {
        machine: Machine {
            sub: format!("test-{}-machine", machine_role.as_str()),
            role: machine_role,
        },
        human: Human {
            sub: format!("test-{}-human", human_role.as_str()),
            email: Some(format!("{}@test.com", human_role.as_str())),
            groups: vec![],
            role: human_role,
        },
    }
}

// ============================================================================
// Mock Auth Client for Testing
// ============================================================================

/// A mock auth client that always returns a configured identity.
/// Useful for testing logic functions that require authentication.
#[derive(Clone)]
pub struct MockAuthClient {
    identity: Identity,
}

impl MockAuthClient {
    /// Create a new mock auth client that returns the given identity
    pub fn new(identity: Identity) -> Self {
        Self { identity }
    }

    /// Create a mock auth client that returns an admin identity
    pub fn admin() -> Self {
        Self::new(test_admin_machine())
    }

    /// Create a mock auth client that returns a maintainer identity
    pub fn maintainer() -> Self {
        Self::new(test_maintainer_machine())
    }

    /// Create a mock auth client that returns an agent identity
    pub fn agent() -> Self {
        Self::new(test_agent_machine())
    }

    /// Create a mock auth client that returns a user identity
    pub fn user() -> Self {
        Self::new(test_user_machine())
    }

    /// Create a mock auth client that returns unauthenticated
    pub fn unauthenticated() -> Self {
        Self::new(Identity::Unauthenticated)
    }
}

impl AuthClientLike for MockAuthClient {
    async fn authenticate(&self, _credentials: RawCredentials) -> Result<Identity, CommonError> {
        Ok(self.identity.clone())
    }

    async fn authenticate_from_headers(
        &self,
        _headers: &HeaderMap,
    ) -> Result<Identity, CommonError> {
        Ok(self.identity.clone())
    }
}

pub struct TestContext {
    pub workspace_root: String,
    pub crate_root: String,
}

pub static INIT_TEST_ONCE: Once = Once::new();

#[macro_export]
macro_rules! setup_test {
    // Explicit key form
    (
        db_conn_string_key: $db_conn_string_key:expr
    ) => {{
        $crate::setup_test!(@inner Some($db_conn_string_key))
    }};

    // No-key form (defaults)
    () => {{
        $crate::setup_test!(@inner None::<&str>)
    }};

    // Private implementation arm
    (@inner $db_conn_string_key:expr) => {{
        $crate::test_utils::helpers::INIT_TEST_ONCE.call_once(|| {
            $crate::crypto::configure_crypto_provider().unwrap();
            $crate::env::load_optional_env_files();
            $crate::logging::configure_logging().unwrap();
        });

        let crate_root = env!("CARGO_MANIFEST_DIR");

        let workspace_root = $crate::test_utils::helpers::get_workspace_root();
        let cur_thread = std::thread::current();
        let test_name = cur_thread.name().unwrap_or("unknown");

        fn construct_data_dir(workspace_root: &str, test_name: &str) -> String {
            let escaped_test_name = test_name
                .replace("::", "_")
                .replace(":", "_")
                .replace("/", "_")
                .replace("\\", "_");
            format!(
                "{}/{}/{}",
                workspace_root,
                std::env::var("ORIGINAL_DATA_DIR").unwrap(),
                escaped_test_name
            )
        }

        fn set_data_dir(workspace_root: &str, test_name: &str) {
            unsafe {
                std::env::set_var("DATA_DIR", construct_data_dir(workspace_root, test_name));
            }
        }

        if std::env::var("ORIGINAL_DATA_DIR").is_ok() {
            set_data_dir(&workspace_root, &test_name);
        } else if std::env::var("DATA_DIR").is_ok() {
            unsafe {
                std::env::set_var("ORIGINAL_DATA_DIR", std::env::var("DATA_DIR").unwrap());
            }
            set_data_dir(&workspace_root, &test_name);
        }

        if let Some(db_conn_string_key) = $db_conn_string_key {
            unsafe {
                std::env::set_var(db_conn_string_key, format!("libsql://{}/test.db?mode=local", construct_data_dir(&workspace_root, &test_name)));
            }
            tracing::info!(
                "db conn string: {}",
                std::env::var(db_conn_string_key).unwrap()
            );
        }


        $crate::test_utils::helpers::TestContext {
            workspace_root,
            crate_root: crate_root.to_string(),
        }
    }};
}
