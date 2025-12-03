use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::{error, info};

use tracing_log_sink::init::init_tracing;
use tracing_log_sink::record::LogRecord;
use tracing_log_sink::sink::LogSink;

/// Simple `LogSink` implementation that writes `LogRecord`s
/// into a Postgres table using `sqlx`.
///
/// This example assumes the following table structure:
///
/// ```sql
/// CREATE TABLE error_logs (
///   ts           timestamptz   NOT NULL,
///   level        text          NOT NULL,
///   target       text          NOT NULL,
///   module_path  text,
///   file         text,
///   line         int4,
///   message      text,
///   fields       jsonb         NOT NULL,
///   service_name text
/// );
/// ```
#[derive(Clone)]
pub struct PostgresSink {
    pool: PgPool,
}

impl PostgresSink {
    /// Create a new sink from a Postgres connection string.
    ///
    /// Example DSN:
    /// `postgres://user:password@localhost:5432/postgres`
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        Ok(Self { pool })
    }
}

#[async_trait]
impl LogSink for PostgresSink {
    async fn send(&self, record: &LogRecord) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Insert a single normalized `LogRecord` into the `error_logs` table.
        sqlx::query(
            r#"
            INSERT INTO error_logs
                (ts, level, target, module_path, file, line, message, fields, service_name)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(record.timestamp)
        .bind(&record.level)
        .bind(&record.target)
        .bind(&record.module_path)
        .bind(&record.file)
        .bind(record.line.map(|l| l as i32))
        .bind(&record.message)
        .bind(serde_json::to_value(&record.fields)?)
        .bind(&record.service_name)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1) Point this DSN to your Postgres instance.
    //    You can also pass it via the `DATABASE_URL` env var.
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:6731867@localhost:5433/postgres".to_string());

    // 2) Create the sink and install the tracing layer.
    let sink = PostgresSink::new(&database_url).await?;
    init_tracing(Arc::new(sink));

    // 3) Emit some events; only `error!` will be persisted
    //    by the default `ErrorLogLayer` configuration.
    info!("service started");
    error!(order_id = 123, "order failed");

    // Give the background task a bit of time to
    // flush the log record into Postgres.
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    Ok(())
}
