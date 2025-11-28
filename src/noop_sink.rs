use crate::record::LogRecord;
use crate::sink::LogSink;
use async_trait::async_trait;
use std::error::Error;

/// A sink that simply drops all records.
///
/// Useful for measuring the overhead of the layer itself without any
/// external I/O, and for unit tests that don't care about persistence.
#[derive(Clone, Default)]
pub struct NoopSink;

#[async_trait]
impl LogSink for NoopSink {
    async fn send(&self, _record: &LogRecord) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }
}
