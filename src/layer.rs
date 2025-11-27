use crate::record::LogRecord;
use crate::sink::LogSink;
use chrono::Utc;
use std::collections::BTreeMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::registry::LookupSpan;

pub struct ErrorLogLayer {
    sender: mpsc::Sender<LogRecord>,
}

impl ErrorLogLayer {
    pub fn new(
        sink: Arc<dyn LogSink>,
        buffer: usize,
        batch_size: usize,
        flush_interval: Duration,
    ) -> (Self, JoinHandle<()>) {
        let (tx, mut rx) = mpsc::channel::<LogRecord>(buffer);

        let handle = tokio::spawn(async move {
            let mut batch = Vec::with_capacity(batch_size);
            let backoff = Duration::from_millis(100);
            let max_backoff = Duration::from_secs(10);

            loop {
                tokio::select! {
                    Some(record) = rx.recv() => {
                        batch.push(record);
                        if batch.len() >= batch_size {
                            if let Err(e) = send_batch(&*sink, &mut batch, backoff, max_backoff).await {
                                eprintln!("error sending log batch: {}", e);
                            }
                        }
                    }
                    _ = sleep(flush_interval) => {
                        if !batch.is_empty() {
                            if let Err(e) = send_batch(&*sink, &mut batch, backoff, max_backoff).await {
                                eprintln!("error flushing log batch: {}", e);
                            }
                        }
                    }
                }
            }
        });

        (Self { sender: tx }, handle)
    }
}

async fn send_batch(
    sink: &dyn LogSink,
    batch: &mut Vec<LogRecord>,
    mut backoff: Duration,
    max_backoff: Duration,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    loop {
        let mut last_err: Option<Box<dyn Error + Send + Sync>> = None;
        for record in batch.iter() {
            if let Err(e) = sink.send(record).await {
                last_err = Some(e);
                break;
            }
        }

        if last_err.is_none() {
            batch.clear();
            return Ok(());
        }

        eprintln!("log sink send failed, retrying in {:?}", backoff);
        sleep(backoff).await;
        backoff = std::cmp::min(backoff * 2, max_backoff);
    }
}

impl<S> Layer<S> for ErrorLogLayer
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    fn on_event(&self, event: &Event, _ctx: Context<'_, S>) {
        if *event.metadata().level() > Level::ERROR {
            return;
        }

        let mut fields = BTreeMap::new();
        let mut message: Option<String> = None;

        let mut visitor = crate::layer::FieldVisitor { fields: &mut fields, message: &mut message };
        event.record(&mut visitor);

        let meta = event.metadata();
        let record = LogRecord {
            timestamp: Utc::now(),
            level: meta.level().to_string(),
            target: meta.target().to_string(),
            module_path: meta.module_path().map(|s| s.to_string()),
            file: meta.file().map(|s| s.to_string()),
            line: meta.line(),
            fields,
            message,
            service_name: None,
        };

        if let Err(_e) = self.sender.try_send(record) {
            eprintln!("log channel full, dropping log record");
        }
    }
}

use tracing::field::{Field, Visit};

pub struct FieldVisitor<'a> {
    pub fields: &'a mut BTreeMap<String, serde_json::Value>,
    pub message: &'a mut Option<String>,
}

impl<'a> Visit for FieldVisitor<'a> {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            *self.message = Some(value.to_string());
        } else {
            self.fields.insert(field.name().to_string(), serde_json::Value::String(value.to_string()));
        }
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields.insert(field.name().to_string(), serde_json::Value::from(value));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields.insert(field.name().to_string(), serde_json::Value::from(value));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields.insert(field.name().to_string(), serde_json::Value::from(value));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.fields.insert(field.name().to_string(), serde_json::Value::String(format!("{:?}", value)));
    }
}
