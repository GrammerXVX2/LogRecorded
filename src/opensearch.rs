use crate::{record::LogRecord, sink::LogSink};
use async_trait::async_trait;
use reqwest::Client;
use std::error::Error;

/// OpenSearch sink that sends log records via HTTP bulk API.
#[derive(Clone)]
pub struct OpenSearchSink {
    client: Client,
    /// Base URL of the OpenSearch cluster, e.g. "http://localhost:9200".
    base_url: String,
    /// Target index name.
    index: String,
}

impl OpenSearchSink {
    pub fn new(base_url: String, index: String) -> Self {
        OpenSearchSink {
            client: Client::new(),
            base_url,
            index,
        }
    }
}

#[async_trait]
impl LogSink for OpenSearchSink {
    async fn send(&self, record: &LogRecord) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Minimal bulk body with a single operation.
        let action = format!("{{\"index\":{{\"_index\":\"{}\"}}}}\n", self.index);
        let doc = serde_json::to_string(record)? + "\n";
        let body = format!("{}{}", action, doc);

        let url = format!("{}/_bulk", self.base_url.trim_end_matches('/'));
        let resp = self
            .client
            .post(&url)
            .header("Content-Type", "application/x-ndjson")
            .body(body)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_else(|_| "<no body>".to_string());
            Err(format!("OpenSearch bulk insert failed with status {}: {}", status, text).into())
        }
    }
}
