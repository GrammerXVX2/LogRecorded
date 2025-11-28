use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{error, info};

#[cfg(feature = "clickhouse")]
use tracing_log_sink::clickhouse::{ClickHouseConfig, ClickHouseSink};
use tracing_log_sink::init::init_tracing;

#[tokio::main]
async fn main() {
    #[cfg(feature = "clickhouse")]
    {
        let config = ClickHouseConfig {
            url: "http://127.0.0.1:8123".to_string(),
            database: "default".to_string(),
            table: "auth_errors".to_string(),
            service_name: None,
            user: Some("default".to_string()),
            password: None,
        };
        let sink = Arc::new(ClickHouseSink::new(config));
        init_tracing(sink);
    }

    info!("starting service");

    error!(
        user_id = 42,
        reason = "invalid password",
        "authentication failed"
    );

    sleep(Duration::from_secs(2)).await;
}
