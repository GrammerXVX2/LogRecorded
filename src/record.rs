use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize)]
pub struct LogRecord {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub target: String,
    pub module_path: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub fields: BTreeMap<String, serde_json::Value>,
    pub message: Option<String>,
    pub service_name: Option<String>,
}
