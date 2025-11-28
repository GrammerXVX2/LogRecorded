use crate::record::LogRecord;
use crate::sink::LogSink;
use async_trait::async_trait;
use reqwest::Client;
use serde::Serialize;
use std::error::Error;
use urlencoding;

/// Configuration for [`ClickHouseSink`].
///
/// The sink talks to ClickHouse over HTTP using the `JSONEachRow` format.
/// It supports both dedicated-table per service and shared-table modes by
/// toggling the `service_name` field and selected table.
#[derive(Clone, Debug)]
pub struct ClickHouseConfig {
    /// Base URL without query, e.g. "http://127.0.0.1:8123"
    pub url: String,
    pub database: String,
    pub table: String,
    pub service_name: Option<String>,
    pub user: Option<String>,
    pub password: Option<String>,
}

/// ClickHouse implementation of [`LogSink`] using the HTTP interface.
#[derive(Clone)]
pub struct ClickHouseSink {
    client: Client,
    config: ClickHouseConfig,
}

impl ClickHouseSink {
    /// Construct a new sink instance using the provided configuration.
    ///
    /// **Parameters**
    /// - `config`: [`ClickHouseConfig`] describing target URL, database,
    ///   table and optional authentication settings.
    ///
    /// **Returns**
    /// - A ready-to-use [`ClickHouseSink`] that can be passed into
    ///   [`init_tracing`] / [`init_tracing_with_config`].
    pub fn new(config: ClickHouseConfig) -> Self {
        let client = Client::new();
        Self { client, config }
    }

    fn endpoint(&self) -> String {
        let mut query = format!(
            "database={}&query=INSERT%20INTO%20{}%20FORMAT%20JSONEachRow",
            self.config.database, self.config.table
        );

        if let Some(user) = &self.config.user {
            query.push_str(&format!("&user={}", urlencoding::encode(user)));
        }
        if let Some(password) = &self.config.password {
            query.push_str(&format!("&password={}", urlencoding::encode(password)));
        }

        format!("{}/?{}", self.config.url, query)
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

    /// Validate that the target ClickHouse table exposes the expected
    /// columns. This is optional and is not called automatically.
    ///
    /// **Returns**
    /// - `Ok(())` if the `DESCRIBE TABLE` query succeeded.
    /// - `Err(..)` if ClickHouse responded with a non-success status.
    pub async fn validate_schema(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut query = format!(
            "query={}",
            urlencoding::encode(&format!(
                "DESCRIBE TABLE {}.{} FORMAT JSON",
                self.config.database, self.config.table
            ))
        );

        if let Some(user) = &self.config.user {
            query.push_str(&format!("&user={}", urlencoding::encode(user)));
        }
        if let Some(password) = &self.config.password {
            query.push_str(&format!("&password={}", urlencoding::encode(password)));
        }

        let url = format!("{}/?{}", self.config.url, query);
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
            let status = resp.status();
            let text = resp.text().await.unwrap_or_else(|_| "<no body>".to_string());
            Err(format!("ClickHouse insert failed with status {}: {}", status, text).into())
        }
    }
}
