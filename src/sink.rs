use crate::record::LogRecord;
use async_trait::async_trait;
use std::error::Error;

/// Asynchronous destination for [`LogRecord`]s produced by the logging layer.
///
/// Implementations are responsible for transporting records to a concrete
/// backend (ClickHouse, Loki, stdout, etc). The layer calls `send` from a
/// background task and never awaits it on the application thread.
#[async_trait]
pub trait LogSink: Send + Sync {
    /// Send a single log record to the underlying backend.
    ///
    /// **Parameters**
    /// - `record`: fully-populated [`LogRecord`] produced by the layer.
    ///
    /// **Returns**
    /// - `Ok(())` if the record was accepted by the backend.
    /// - `Err(..)` if the backend failed (network error, serialization
    ///   error, HTTP status, etc.). The layer will treat this as a
    ///   transient failure and retry the batch with backoff.
    ///
    /// This method is called from a Tokio task that owns the batching
    /// loop. Implementations should strive to be non-blocking and use
    /// async I/O under the hood.
    async fn send(&self, record: &LogRecord) -> Result<(), Box<dyn Error + Send + Sync>>;

    /// Flush any buffered records, if the backend implements buffering.
    ///
    /// **Returns**
    /// - `Ok(())` if all local buffers were successfully flushed.
    /// - `Err(..)` if the backend reported an error during flush.
    ///
    /// Default implementation is a no-op.
    async fn flush(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }
}
