
DROP TABLE IF EXISTS default.auth_errors;

CREATE TABLE default.auth_errors (
    timestamp    String,
    level        String,
    target       String,
    module_path  Nullable(String),
    file         Nullable(String),
    line         Nullable(UInt32),
    message      Nullable(String),
    fields       String
) ENGINE = MergeTree
ORDER BY (timestamp, level, target);
