use std::{env, str::FromStr};

use tracing::warn;
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

pub fn configure_logging() -> Result<(), anyhow::Error> {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_str(
            env::var("RUST_LOG").unwrap_or("info".to_string()).as_str(),
        )?)
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
