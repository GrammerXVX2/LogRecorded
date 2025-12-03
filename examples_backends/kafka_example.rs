use std::sync::Arc;

use tracing::{error, info};
use tracing_log_sink::{
    backend::{make_sink_from_config, parse_dsn},
    init::init_tracing,
    sink::LogSink,
};

#[tokio::main]
async fn main() {
    // Example DSN: kafka://broker1:9092,broker2:9092/logs_topic
    let dsn = std::env::var("LOG_SINK_DSN")
        .unwrap_or_else(|_| "kafka://localhost:9092/logs".to_string());

    let backend_cfg = parse_dsn(&dsn).expect("invalid LOG_SINK_DSN");
    let sink: Arc<dyn LogSink> = make_sink_from_config(&backend_cfg)
        .expect("failed to build kafka backend sink");

    init_tracing(sink);

    info!("kafka backend example started");
    error!(topic = "logs", "simulated error sent via Kafka backend");
}
