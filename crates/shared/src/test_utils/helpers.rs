use std::{sync::Once, time::Duration};

use crate::error::DynError;
use tokio_graceful_shutdown::{SubsystemHandle, Toplevel};

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

#[macro_export]
macro_rules! setup_sql_fixtures {
    ($conn:expr, $($file:expr),* $(,)?) => {
        async {
            $(
                let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $file));
                    $conn.execute(sql, libsql::params!())
                    .await
                    .expect(&format!("Failed to execute SQL fixture: {}", $file));
            )*
        }
    };
}

pub use setup_sql_fixtures;

pub struct TestContext {
    pub workspace_root: String,
    pub crate_root: String,
}

pub async fn run_test_in_subsystem<Fut, Subsys>(subsys: Subsys)
where
    Subsys: 'static + FnOnce(SubsystemHandle<DynError>) -> Fut + Send,
    Fut: 'static + Future<Output = ()> + Send,
{
    Toplevel::new(subsys)
        .catch_signals()
        .handle_shutdown_requests(Duration::from_millis(30_000))
        .await
        .unwrap()
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
