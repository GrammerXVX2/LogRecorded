use crate::record::LogRecord;
use async_trait::async_trait;
use std::error::Error;

#[async_trait]
pub trait LogSink: Send + Sync {
    async fn send(&self, record: &LogRecord) -> Result<(), Box<dyn Error + Send + Sync>>;

    async fn flush(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }
}
