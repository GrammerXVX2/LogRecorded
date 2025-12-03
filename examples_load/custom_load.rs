use std::sync::Arc;
use std::time::Instant;
use tokio::time::{sleep, Duration};
use tracing::error;

use tracing_log_sink::init::{init_tracing_with_config, LayerConfig};
use tracing_log_sink::noop_sink::NoopSink;

#[tokio::main]
async fn main() {
    let sink = Arc::new(NoopSink::default());

    let layer_config = LayerConfig {
        channel_buffer: 50_000,
        batch_size: 1_000,
        flush_interval: Duration::from_millis(200),
        enable_stdout: false,
    };

    init_tracing_with_config(sink, layer_config);

    let n: u64 = 100_000;
    let start = Instant::now();

    for i in 0..n {
        error!(iteration = i, "custom load test error");
    }

    let elapsed = start.elapsed();
    println!("custom config: sent {} events in {:?} (~{:.0} ev/s)",
        n,
        elapsed,
        n as f64 / elapsed.as_secs_f64()
    );

    sleep(Duration::from_secs(2)).await;
}
