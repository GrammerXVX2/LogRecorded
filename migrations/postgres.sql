-- Minimal schema for tracing-log-sink Postgres example
-- Adjust schema/database as needed for your environment.

CREATE TABLE IF NOT EXISTS error_logs (
    ts           TIMESTAMPTZ   NOT NULL,
    level        TEXT          NOT NULL,
    target       TEXT          NOT NULL,
    module_path  TEXT,
    file         TEXT,
    line         INT4,
    message      TEXT,
    fields       JSONB         NOT NULL,
    service_name TEXT
);
