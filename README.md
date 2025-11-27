# tracing-log-sink

Async tracing layer that captures `error!` (and below) events and ships them to pluggable backends like ClickHouse.

## Features

- `ErrorLogLayer` implementing `tracing_subscriber::Layer`
- Backend-agnostic `LogRecord` and `LogSink` trait
- Bounded channel + background task with batching and retry
- ClickHouse backend via HTTP JSONEachRow (feature `clickhouse`)

## Running examples

```powershell
# Per-service dedicated table
cargo run --example per_service --features clickhouse

# Shared table for all services
cargo run --example shared_table --features clickhouse
```
