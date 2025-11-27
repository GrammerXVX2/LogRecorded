use crate::layer::ErrorLogLayer;
use crate::sink::LogSink;
use std::sync::Arc;
use tokio::time::Duration;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

pub fn init_tracing(sink: Arc<dyn LogSink>) {
    let (layer, _handle) = ErrorLogLayer::new(
        sink,
        1024,
        128,
        Duration::from_secs(1),
    );

    let subscriber = Registry::default().with(layer);
    tracing::subscriber::set_global_default(subscriber).expect("set global subscriber");
}
