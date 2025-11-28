
DROP TABLE IF EXISTS default.service_logs;

CREATE TABLE IF NOT EXISTS service_logs (
    timestamp      String,
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
