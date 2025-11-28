use crate::layer::ErrorLogLayer;
use crate::sink::LogSink;
use std::sync::Arc;
use tokio::time::Duration;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

/// Configuration for the logging layer.
///
/// Controls the size of the internal channel buffer, the maximum
/// batch size used when flushing to the sink, and how often pending
/// batches are flushed even if they are not full.
///
/// **Fields**
/// - `channel_buffer`: maximum number of [`LogRecord`]s that can be
///   queued in memory before new records start to be dropped.
/// - `batch_size`: number of records sent to the sink in a single batch.
/// - `flush_interval`: maximum time the layer will wait before flushing
///   a non-empty batch even if it did not reach `batch_size`.
#[derive(Clone, Debug)]
pub struct LayerConfig {
    pub channel_buffer: usize,
    pub batch_size: usize,
    pub flush_interval: Duration,
}

impl Default for LayerConfig {
    fn default() -> Self {
        Self {
            channel_buffer: 1024,
            batch_size: 128,
            flush_interval: Duration::from_secs(1),
        }
    }
}

/// Initialize global `tracing` subscriber using the provided sink and
/// [`LayerConfig`].
///
/// **Parameters**
/// - `sink`: implementation of [`LogSink`] that will receive
///   normalized [`LogRecord`]s.
/// - `config`: [`LayerConfig`] controlling buffering and batching
///   behavior of the layer.
///
/// **Effects**
///
/// This installs a [`Registry`] combined with [`ErrorLogLayer`] as the
/// global default subscriber, so all `tracing` events in the process
/// are observed by the layer.
pub fn init_tracing_with_config(sink: Arc<dyn LogSink>, config: LayerConfig) {
    let (layer, _handle) = ErrorLogLayer::new(
        sink,
        config.channel_buffer,
        config.batch_size,
        config.flush_interval,
    );

    let subscriber = Registry::default().with(layer);
    tracing::subscriber::set_global_default(subscriber).expect("set global subscriber");
}

/// Initialize tracing with sensible defaults.
///
/// **Parameters**
/// - `sink`: implementation of [`LogSink`] that will receive
///   normalized [`LogRecord`]s.
///
/// **Behavior**
///
/// Equivalent to calling [`init_tracing_with_config`] with
/// [`LayerConfig::default`]. This is the recommended entrypoint for
/// typical microservices.
pub fn init_tracing(sink: Arc<dyn LogSink>) {
    init_tracing_with_config(sink, LayerConfig::default());
}
