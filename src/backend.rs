use std::sync::Arc;

use crate::sink::LogSink;

/// Supported backend kinds that can be selected via DSN or config.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    Clickhouse,
    Postgres,
    Kafka,
    OpenSearch,
}

/// High-level backend configuration built from a DSN or explicit fields.
///
/// For now it only stores the target kind and the raw DSN string; this
/// keeps the API stable while individual backends remain optional.
#[derive(Debug, Clone)]
pub struct BackendConfig {
    /// Selected backend implementation.
    pub kind: BackendKind,
    /// Raw DSN that was used to construct this config.
    pub dsn: String,
}

impl BackendConfig {
    pub fn new(kind: BackendKind, dsn: impl Into<String>) -> Self {
        BackendConfig { kind, dsn: dsn.into() }
    }
}

/// Parse a DSN string and infer the backend kind from its scheme.
///
/// Examples:
/// - "clickhouse://user:pass@127.0.0.1:8123/default/logs"
/// - "postgres://user:pass@127.0.0.1:5432/db"
/// - "kafka://broker1,broker2/topic"
/// - "opensearch://user:pass@127.0.0.1:9200/index"
pub fn parse_dsn(dsn: &str) -> Result<BackendConfig, DsnError> {
    let lower = dsn.to_ascii_lowercase();

    if lower.starts_with("clickhouse://") {
        Ok(BackendConfig::new(BackendKind::Clickhouse, dsn))
    } else if lower.starts_with("postgres://") || lower.starts_with("postgresql://") {
        Ok(BackendConfig::new(BackendKind::Postgres, dsn))
    } else if lower.starts_with("kafka://") {
        Ok(BackendConfig::new(BackendKind::Kafka, dsn))
    } else if lower.starts_with("opensearch://") {
        Ok(BackendConfig::new(BackendKind::OpenSearch, dsn))
    } else {
        Err(DsnError::UnknownScheme)
    }
}

/// Error type returned when parsing a DSN.
#[derive(thiserror::Error, Debug)]
pub enum DsnError {
    #[error("unknown or unsupported DSN scheme")] 
    UnknownScheme,
}

/// Error type returned when building a backend sink from configuration.
#[derive(thiserror::Error, Debug)]
pub enum BackendBuildError {
    #[error("clickhouse feature is not enabled")] 
    ClickhouseFeatureDisabled,

    #[error("backend kind not yet implemented: {0:?}")] 
    Unimplemented(BackendKind),
}

/// Create a concrete `LogSink` implementation from a `BackendConfig`.
///
/// This is the main entry point for applications that want to select
/// a backend using a single DSN string instead of constructing sinks
/// manually.
pub fn make_sink_from_config(cfg: &BackendConfig) -> Result<Arc<dyn LogSink>, BackendBuildError> {
    match cfg.kind {
        BackendKind::Clickhouse => {
            #[cfg(feature = "clickhouse")]
            {
                use crate::clickhouse::{ClickHouseConfig, ClickHouseSink};

                // For now we treat the entire DSN as the base HTTP URL and
                // use conservative defaults for database/table. A richer
                // ClickHouse-specific DSN parser can be added later.
                let config = ClickHouseConfig {
                    url: cfg.dsn.clone(),
                    database: "default".to_string(),
                    table: "logs".to_string(),
                    service_name: None,
                    user: None,
                    password: None,
                };

                let sink = ClickHouseSink::new(config);
                Ok(Arc::new(sink) as Arc<dyn LogSink>)
            }

            #[cfg(not(feature = "clickhouse"))]
            {
                let _ = cfg; // silence unused warning when feature is disabled
                Err(BackendBuildError::ClickhouseFeatureDisabled)
            }
        }
        BackendKind::Postgres => {
            #[cfg(feature = "postgres")]
            {
                use crate::postgres::PostgresSink;

                // Use the DSN as-is and write into a generic `logs` table
                // with a single `record JSONB` column.
                let table = "logs".to_string();
                let sink = tokio::runtime::Runtime::new()
                    .expect("create runtime")
                    .block_on(PostgresSink::connect(&cfg.dsn, table))
                    .expect("connect postgres");

                Ok(Arc::new(sink) as Arc<dyn LogSink>)
            }

            #[cfg(not(feature = "postgres"))]
            {
                let _ = cfg;
                Err(BackendBuildError::Unimplemented(BackendKind::Postgres))
            }
        }
        BackendKind::Kafka => {
            #[cfg(feature = "kafka")]
            {
                use crate::kafka::KafkaSink;

                // Expect DSN format: kafka://broker1,broker2/topic
                let without_scheme = cfg
                    .dsn
                    .trim_start_matches("kafka://");
                let parts: Vec<&str> = without_scheme.split('/').collect();
                let brokers = parts.get(0).cloned().unwrap_or("");
                let topic = parts.get(1).cloned().unwrap_or("logs");

                let sink = KafkaSink::new(brokers, topic)
                    .expect("create kafka sink");

                Ok(Arc::new(sink) as Arc<dyn LogSink>)
            }

            #[cfg(not(feature = "kafka"))]
            {
                let _ = cfg;
                Err(BackendBuildError::Unimplemented(BackendKind::Kafka))
            }
        }
        BackendKind::OpenSearch => {
            #[cfg(feature = "opensearch")]
            {
                use crate::opensearch::OpenSearchSink;

                // Expect DSN format: opensearch://host:port/index
                let without_scheme = cfg
                    .dsn
                    .trim_start_matches("opensearch://");
                let parts: Vec<&str> = without_scheme.split('/').collect();
                let base = parts.get(0).cloned().unwrap_or("localhost:9200");
                let index = parts.get(1).cloned().unwrap_or("logs");

                let base_url = if base.starts_with("http://") || base.starts_with("https://") {
                    base.to_string()
                } else {
                    format!("http://{}", base)
                };

                let sink = OpenSearchSink::new(base_url, index.to_string());
                Ok(Arc::new(sink) as Arc<dyn LogSink>)
            }

            #[cfg(not(feature = "opensearch"))]
            {
                let _ = cfg;
                Err(BackendBuildError::Unimplemented(BackendKind::OpenSearch))
            }
        }
    }
}
