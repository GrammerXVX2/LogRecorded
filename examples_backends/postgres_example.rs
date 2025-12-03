use std::sync::Arc;

use tracing::{error, info};
use tracing_log_sink::{
    backend::{make_sink_from_config, parse_dsn},
    init::init_tracing,
    sink::LogSink,
};

#[tokio::main]
async fn main() {
    // Example DSN: postgres://user:pass@127.0.0.1:5432/db_name
    // Table `logs` with column `record jsonb` must exist.
    let dsn = std::env::var("LOG_SINK_DSN")
        .unwrap_or_else(|_| "postgres://user:pass@127.0.0.1:5432/db_name".to_string());

    let backend_cfg = parse_dsn(&dsn).expect("invalid LOG_SINK_DSN");
    let sink: Arc<dyn LogSink> = make_sink_from_config(&backend_cfg)
        .expect("failed to build postgres backend sink");

    init_tracing(sink);

    info!("postgres backend example started");
    error!(error_code = 123, "simulated error sent via Postgres backend");
}
