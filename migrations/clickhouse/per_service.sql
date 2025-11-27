-- Dedicated table per microservice
CREATE TABLE IF NOT EXISTS {database}.{table} (
    timestamp      DateTime64(3, 'UTC'),
    level          String,
    target         String,
    module_path    Nullable(String),
    file           Nullable(String),
    line           Nullable(UInt32),
    message        Nullable(String),
    fields         String
) ENGINE = MergeTree
ORDER BY (timestamp, level, target);
