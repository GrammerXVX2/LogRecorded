use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{error, info};

use tracing_log_sink::init::init_tracing;
#[cfg(feature = "clickhouse")]
use tracing_log_sink::clickhouse::{ClickHouseConfig, ClickHouseSink};

#[tokio::main]
async fn main() {
    #[cfg(feature = "clickhouse")]
    {
        let config = ClickHouseConfig {
            url: "http://localhost:8123".to_string(),
            database: "logs".to_string(),
            table: "service_logs".to_string(),
            service_name: Some("auth-service".to_string()),
        };
        let sink = Arc::new(ClickHouseSink::new(config));
        init_tracing(sink);
    }

    info!("starting service");

    error!(user_id = 42, reason = "invalid password", "authentication failed");

    sleep(Duration::from_secs(2)).await;
}
