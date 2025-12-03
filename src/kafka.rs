use crate::{record::LogRecord, sink::LogSink};
use async_trait::async_trait;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::error::Error;
use std::time::Duration;

/// Kafka sink that publishes each log record as a JSON message to
/// a configured topic.
#[derive(Clone)]
pub struct KafkaSink {
    producer: FutureProducer,
    topic: String,
}

impl KafkaSink {
    /// Create a new Kafka sink.
    ///
    /// `brokers` is a comma-separated list of broker addresses.
    /// `topic` is the target Kafka topic.
    pub fn new(brokers: &str, topic: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .create()?;

        Ok(KafkaSink {
            producer,
            topic: topic.to_string(),
        })
    }
}

#[async_trait]
impl LogSink for KafkaSink {
    async fn send(&self, record: &LogRecord) -> Result<(), Box<dyn Error + Send + Sync>> {
        let payload = serde_json::to_vec(record)?;

        let record = FutureRecord::to(&self.topic).payload(&payload);
        // Wait for the delivery report with a bounded timeout.
        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(e, _)| -> Box<dyn Error + Send + Sync> { Box::new(e) })?;

        Ok(())
    }
}
