-- Shared table for all services
CREATE TABLE IF NOT EXISTS {database}.{table} (
    timestamp      DateTime64(3, 'UTC'),
    level          String,
    target         String,
    module_path    Nullable(String),
    file           Nullable(String),
    line           Nullable(UInt32),
    message        Nullable(String),
    service_name   String,
    fields         String
) ENGINE = MergeTree
ORDER BY (service_name, timestamp, level, target);
