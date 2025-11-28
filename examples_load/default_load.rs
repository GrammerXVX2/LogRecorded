use std::sync::Arc;
use std::time::Instant;
use tokio::time::{sleep, Duration};
use tracing::error;

use tracing_log_sink::init::init_tracing;
use tracing_log_sink::noop_sink::NoopSink;

#[tokio::main]
async fn main() {
    let sink = Arc::new(NoopSink::default());
    init_tracing(sink);

    let n: u64 = 100_000;
    let start = Instant::now();

    for i in 0..n {
        error!(iteration = i, "default load test error");
    }

    let elapsed = start.elapsed();
    println!("default config: sent {} events in {:?} (~{:.0} ev/s)",
        n,
        elapsed,
        n as f64 / elapsed.as_secs_f64()
    );

    // Give background task a little time to drain the channel
    sleep(Duration::from_secs(2)).await;
}
