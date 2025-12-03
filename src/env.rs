/// Environment variable names used by this crate for convenient
/// configuration of sinks from microservices.
///
/// These are purely helpers; the core sink types remain decoupled from
/// environment access.

/// ClickHouse base HTTP URL, e.g. `http://127.0.0.1:8123`.
pub const LOG_SINK_CLICKHOUSE_URL_ENV: &str = "LOG_SINK_CLICKHOUSE_URL";

/// ClickHouse database name.
pub const LOG_SINK_CLICKHOUSE_DB_ENV: &str = "LOG_SINK_CLICKHOUSE_DB";

/// ClickHouse target table name.
pub const LOG_SINK_CLICKHOUSE_TABLE_ENV: &str = "LOG_SINK_CLICKHOUSE_TABLE";

/// Optional ClickHouse user name.
pub const LOG_SINK_CLICKHOUSE_USER_ENV: &str = "LOG_SINK_CLICKHOUSE_USER";

/// Optional ClickHouse password.
pub const LOG_SINK_CLICKHOUSE_PASSWORD_ENV: &str = "LOG_SINK_CLICKHOUSE_PASSWORD";

/// Optional logical service name used in shared-table setups.
pub const LOG_SINK_SERVICE_NAME_ENV: &str = "LOG_SINK_SERVICE_NAME";

/// Read an environment variable or fall back to a provided default.
pub fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
