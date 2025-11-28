use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::BTreeMap;

/// Normalized representation of a `tracing` event that is ready to be
/// shipped to an external logging backend.
///
/// This struct is backend-agnostic and captures both the event metadata
/// (level, target, module, file, line) and all structured fields.
#[derive(Debug, Clone, Serialize)]
pub struct LogRecord {
    /// UTC timestamp when the event was observed by the layer.
    pub timestamp: DateTime<Utc>,
    /// Stringified log level (e.g. "ERROR").
    pub level: String,
    /// Event target from `tracing` metadata.
    pub target: String,
    /// Optional Rust module path where the event originated.
    pub module_path: Option<String>,
    /// Optional source file path.
    pub file: Option<String>,
    /// Optional source line number.
    pub line: Option<u32>,
    /// All structured fields attached to the event, including custom keys.
    pub fields: BTreeMap<String, serde_json::Value>,
    /// Optional formatted log message, if present.
    pub message: Option<String>,
    /// Optional logical service name, populated by sinks or callers.
    pub service_name: Option<String>,
}
