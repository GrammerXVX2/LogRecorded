use crate::record::LogRecord;
use crate::sink::LogSink;
use chrono::Utc;
use std::collections::BTreeMap;
use std::error::Error;
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::registry::LookupSpan;

/// `tracing_subscriber` layer that observes events and forwards them to
/// an asynchronous [`LogSink`] via a bounded channel and background task.
///
/// By default this layer only captures events with level `ERROR` and above
/// and turns them into [`LogRecord`]s. Network I/O is fully decoupled from
/// application threads to minimize impact on request latency.
pub struct ErrorLogLayer {
    sender: mpsc::Sender<LogRecord>,
    /// Total events seen by the layer (before filtering by level).
    pub total_events: Arc<AtomicU64>,
    /// Successfully enqueued into channel.
    pub enqueued_events: Arc<AtomicU64>,
    /// Dropped because the channel was full.
    pub dropped_events: Arc<AtomicU64>,
}

impl ErrorLogLayer {
    /// Create a new layer and spawn a background task that pulls
    /// [`LogRecord`]s from a bounded channel and sends them to the
    /// provided [`LogSink`].
    ///
    /// Minimal thresholds are enforced for `buffer`, `batch_size` and
    /// `flush_interval` to avoid degenerate configurations.
    pub fn new(
        sink: Arc<dyn LogSink>,
        buffer: usize,
        batch_size: usize,
        flush_interval: Duration,
    ) -> (Self, JoinHandle<()>) {
        // Enforce minimal thresholds to avoid degenerate configs.
        let buffer = buffer.max(16);
        let batch_size = batch_size.max(1);
        let flush_interval = if flush_interval < Duration::from_millis(10) {
            Duration::from_millis(10)
        } else {
            flush_interval
        };

        let (tx, mut rx) = mpsc::channel::<LogRecord>(buffer);

        let total_events = Arc::new(AtomicU64::new(0));
        let enqueued_events = Arc::new(AtomicU64::new(0));
        let dropped_events = Arc::new(AtomicU64::new(0));

        let _total_events_bg = Arc::clone(&total_events);
        let enqueued_events_bg = Arc::clone(&enqueued_events);
        let _dropped_events_bg = Arc::clone(&dropped_events);

        let handle = tokio::spawn(async move {
            let mut batch = Vec::with_capacity(batch_size);
            let backoff = Duration::from_millis(100);
            let max_backoff = Duration::from_secs(10);

            loop {
                tokio::select! {
                    Some(record) = rx.recv() => {
                        batch.push(record);
                        enqueued_events_bg.fetch_add(1, Ordering::Relaxed);
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

        (Self {
            sender: tx,
            total_events,
            enqueued_events,
            dropped_events,
        }, handle)
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
        self.total_events.fetch_add(1, Ordering::Relaxed);
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
            self.dropped_events.fetch_add(1, Ordering::Relaxed);
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
