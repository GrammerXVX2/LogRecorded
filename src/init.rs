use crate::layer::ErrorLogLayer;
use crate::sink::LogSink;
use std::sync::Arc;
use tokio::time::Duration;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

/// Конфигурация слоя логирования.
///
/// Управляет размером внутреннего буфера, максимальным размером батча
/// при отправке в sink, частотой принудительного flush, а также тем,
/// нужно ли дополнительно печатать логи в консоль через `fmt`‑слой.
///
/// **Поля**
/// - `channel_buffer`: максимальное число [`LogRecord`] в очереди до
///   начала дропа новых записей.
/// - `batch_size`: размер батча для отправки в sink.
/// - `flush_interval`: максимальный интервал между flush’ами даже при
///   неполном батче.
/// - `enable_stdout`: если `true`, поверх `ErrorLogLayer` добавляется
///   `tracing_subscriber::fmt::Layer` и ошибки печатаются в консоль.
#[derive(Clone, Debug)]
pub struct LayerConfig {
    pub channel_buffer: usize,
    pub batch_size: usize,
    pub flush_interval: Duration,
    pub enable_stdout: bool,
}

impl Default for LayerConfig {
    fn default() -> Self {
        Self {
            channel_buffer: 1024,
            batch_size: 128,
            flush_interval: Duration::from_secs(1),
            enable_stdout: true,
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

    // Всегда подключаем слой, который пишет в внешний sink (БД и т.д.).
    // Дополнительно, при `enable_stdout = true`, подключаем `fmt`‑слой,
    // чтобы видеть события в консоли. Для совместимости типов собираем
    // subscriber в двух вариантах.
    if config.enable_stdout {
        let fmt_layer = tracing_subscriber::fmt::layer();
        let subscriber = Registry::default().with(layer).with(fmt_layer);
        tracing::subscriber::set_global_default(subscriber).expect("set global subscriber");
    } else {
        let subscriber = Registry::default().with(layer);
        tracing::subscriber::set_global_default(subscriber).expect("set global subscriber");
    }
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
