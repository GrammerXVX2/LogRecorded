use crate::{record::LogRecord, sink::LogSink};
use async_trait::async_trait;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_postgres::{Client, NoTls};

/// Simple Postgres-based sink that inserts each log record into a table.
///
/// DSN is expected in the standard Postgres format, e.g.
///   postgres://user:pass@host:5432/dbname
///
/// The table is assumed to exist with a schema compatible with the
/// serialized [`LogRecord`]. For simplicity we store the full record as
/// JSON in a single column.
#[derive(Clone)]
pub struct PostgresSink {
    client: Arc<Mutex<Client>>,
    table: String,
}

impl PostgresSink {
    /// Create a new `PostgresSink` by connecting to the database using the
    /// provided DSN and target table name.
    pub async fn connect(dsn: &str, table: String) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let (client, connection) = tokio_postgres::connect(dsn, NoTls).await?;

        // Spawn the connection object to drive the I/O in the background.
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("postgres connection error: {}", e);
            }
        });

        Ok(PostgresSink {
            client: Arc::new(Mutex::new(client)),
            table,
        })
    }
}

#[async_trait]
impl LogSink for PostgresSink {
    async fn send(&self, record: &LogRecord) -> Result<(), Box<dyn Error + Send + Sync>> {
        let json = serde_json::to_value(record)?;
        let query = format!("INSERT INTO {} (record) VALUES ($1)", self.table);

        let client = self.client.clone();
        let mut guard = client.lock().await;
        guard.execute(&*query, &[&json]).await?;
        Ok(())
    }
}
