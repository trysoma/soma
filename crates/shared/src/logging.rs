use std::{env, str::FromStr};

use tracing::warn;
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

pub fn configure_logging() -> Result<(), anyhow::Error> {
    let rust_log = env::var("RUST_LOG").unwrap_or("info".to_string());

    // Check if user already specified pmdaemon::manager in RUST_LOG
    let has_pmdaemon_override = rust_log.contains("pmdaemon::manager");

    // Determine pmdaemon::manager log level based on overall log level
    // Only set if user hasn't explicitly specified it
    let filter_str = if has_pmdaemon_override {
        // User already specified pmdaemon::manager, use their setting
        rust_log
    } else {
        // Determine pmdaemon level based on overall log level
        let pmdaemon_level = if rust_log.contains("debug") || rust_log.contains("trace") {
            "debug"
        } else {
            "warn"
        };
        // Append pmdaemon override
        format!("{rust_log},pmdaemon::manager={pmdaemon_level},pmdaemon::process={pmdaemon_level}")
    };

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_str(filter_str.as_str())?)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_writer(std::io::stdout);

    let subscriber = if env::var("LOG_FORMAT").unwrap_or("text".to_string()) == "json" {
        subscriber.json().try_init()
    } else {
        subscriber.try_init()
    };

    match subscriber {
        Ok(_) => Ok(()),
        Err(e) => {
            warn!(
                "Failed to initialize logging, potentially because we have initialized logging already: {}",
                e
            );

            Ok(())
        }
    }
}
