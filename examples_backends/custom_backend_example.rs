use std::sync::Arc;

use async_trait::async_trait;
use tracing::{error, info};
use tracing_log_sink::{
    init::init_tracing,
    record::LogRecord,
    sink::LogSink,
};

/// Example of integrating a completely custom backend by implementing
/// the `LogSink` trait directly. Imagine this talks to some
/// proprietary DB for which this crate does not provide a built-in
/// sink.
struct MyCustomDbSink;

#[async_trait]
impl LogSink for MyCustomDbSink {
    async fn send(&self, record: &LogRecord) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Here you would call your own client library for the target DB.
        // For the sake of example we just print the record.
        println!("[my-custom-db] {:?}", record);
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let sink: Arc<dyn LogSink> = Arc::new(MyCustomDbSink);

    init_tracing(sink);

    info!("custom backend example started");
    error!(db = "my-custom-db", "simulated error sent via custom backend");
}