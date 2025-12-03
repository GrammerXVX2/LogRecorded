use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{error, info};

#[cfg(feature = "clickhouse")]
use tracing_log_sink::clickhouse::{ClickHouseConfig, ClickHouseSink};
use tracing_log_sink::init::{init_tracing_with_config, LayerConfig};

#[tokio::main]
async fn main() {
    #[cfg(feature = "clickhouse")]
    {
        let config = ClickHouseConfig {
            url: "http://127.0.0.1:8123".to_string(),
            database: "default".to_string(),
            table: "service_logs".to_string(),
            service_name: Some("auth-service".to_string()),
            user: Some("default".to_string()),
            password: None,
        };
        let sink = Arc::new(ClickHouseSink::new(config));
        let layer_config = LayerConfig {
            channel_buffer: 10_000,
            batch_size: 500,
            flush_interval: Duration::from_millis(500),
            enable_stdout: true,
        };
        init_tracing_with_config(sink, layer_config);
    }

    info!("starting service");

    error!(
        user_id = 42,
        reason = "invalid password",
        "authentication failed"
    );

    sleep(Duration::from_secs(2)).await;
}
