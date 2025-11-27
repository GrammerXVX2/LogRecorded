use crate::record::LogRecord;
use crate::sink::LogSink;
use async_trait::async_trait;
use reqwest::Client;
use serde::Serialize;
use std::error::Error;
use urlencoding;

#[derive(Clone, Debug)]
pub struct ClickHouseConfig {
    pub url: String,
    pub database: String,
    pub table: String,
    pub service_name: Option<String>,
}

#[derive(Clone)]
pub struct ClickHouseSink {
    client: Client,
    config: ClickHouseConfig,
}

impl ClickHouseSink {
    pub fn new(config: ClickHouseConfig) -> Self {
        let client = Client::new();
        Self { client, config }
    }

    fn endpoint(&self) -> String {
        format!("{}/?database={}&query=INSERT%20INTO%20{}%20FORMAT%20JSONEachRow", self.config.url, self.config.database, self.config.table)
    }

    fn map_record(&self, record: &LogRecord) -> ClickHouseRow {
        ClickHouseRow {
            timestamp: record.timestamp.to_rfc3339(),
            level: record.level.clone(),
            target: record.target.clone(),
            module_path: record.module_path.clone(),
            file: record.file.clone(),
            line: record.line.map(|l| l as u64),
            message: record.message.clone(),
            service_name: self.config.service_name.clone().or_else(|| record.service_name.clone()),
            fields: serde_json::to_string(&record.fields).unwrap_or_else(|_| "{}".to_string()),
        }
    }

    pub async fn validate_schema(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let query = format!("DESCRIBE TABLE {}.{} FORMAT JSON", self.config.database, self.config.table);
        let url = format!("{}/?query={}", self.config.url, urlencoding::encode(&query));
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(format!("ClickHouse schema validation failed with status {}", resp.status()).into());
        }
        Ok(())
    }
}

#[cfg(feature = "clickhouse")]
#[derive(Serialize)]
struct ClickHouseRow {
    timestamp: String,
    level: String,
    target: String,
    module_path: Option<String>,
    file: Option<String>,
    line: Option<u64>,
    message: Option<String>,
    service_name: Option<String>,
    fields: String,
}

#[cfg(feature = "clickhouse")]
#[async_trait]
impl LogSink for ClickHouseSink {
    async fn send(&self, record: &LogRecord) -> Result<(), Box<dyn Error + Send + Sync>> {
        let row = self.map_record(record);
        let body = serde_json::to_string(&row)? + "\n";
        let resp = self.client.post(&self.endpoint()).body(body).send().await?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("ClickHouse insert failed with status {}", resp.status()).into())
        }
    }
}
