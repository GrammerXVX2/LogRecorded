# tracing-log-sink

Асинхронный слой для `tracing`, который перехватывает события `error!`
и отправляет их в подключаемые backends (ClickHouse, ваша БД и т.д.)
через абстракцию sink.

## Возможности

- **`ErrorLogLayer`** — реализация `tracing_subscriber::Layer`
- **Универсальная модель**: `LogRecord` + трейт `LogSink`
- **Асинхронный пайплайн**: ограниченный канал, фоновой task, batching,
  retry с экспоненциальным backoff
- **Встроенные backends**:
  - ClickHouse по HTTP в формате `JSONEachRow` (feature `clickhouse`)
  - `NoopSink` для локальных и нагрузочных тестов без БД

---

## Установка

Добавьте в ваш `Cargo.toml`:

```toml
[dependencies]
tracing-log-sink = { version = "0.1.1", features = ["clickhouse"] } # ClickHouse support (optional)
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["fmt", "registry"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

Если вы хотите использовать только свой backend (например, Postgres)
и вам не нужен ClickHouse, можно отключить `default-features` и не
подключать feature `clickhouse`:

```toml
[dependencies]
tracing-log-sink = { version = "0.1.1", default-features = false }
```

---

## Базовые понятия

- **`LogRecord`** — нормализованное представление `tracing::Event`:
  - `timestamp: DateTime<Utc>` — время, когда событие поймал слой
  - `level: String` — уровень (`"ERROR"`, `"WARN"`, ...)
  - `target`, `module_path`, `file`, `line` — метаданные из `tracing`
  - `fields: BTreeMap<String, serde_json::Value>` — все структурированные поля (`error!(user_id = 42, ...)`)
  - `message: Option<String>` — форматированное сообщение
  - `service_name: Option<String>` — имя сервиса (может задаваться sink’ом)

- **`LogSink`** — async‑трейтом, который получает `LogRecord` и отправляет его в конкретный backend (ClickHouse, Postgres, Loki, stdout и т.д.):

```rust
#[async_trait::async_trait]
pub trait LogSink: Send + Sync {
    async fn send(&self, record: &LogRecord) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn flush(&self) -> Result<(), Box<dyn Error + Send + Sync>> { Ok(()) }
}
```

- **`ErrorLogLayer`** — слой `tracing_subscriber`, который:
  - слушает все события,
  - фильтрует уровни выше `ERROR` (`error!`, `warn!`, ...),
  - превращает их в `LogRecord`,
  - отправляет в `LogSink` через буферизированный канал и фоновой таск.

Для инициализации есть удобные функции в модуле `init`.

---

## Быстрый старт: `NoopSink` (без БД)

Полезно для локальной разработки и бенчмарков, когда нужно померить
overhead слоя без внешнего I/O.

```rust
use std::sync::Arc;
use tracing::{info, error};
use tracing_log_sink::init::init_tracing;
use tracing_log_sink::noop_sink::NoopSink;

#[tokio::main]
async fn main() {
    let sink = Arc::new(NoopSink::default());
    init_tracing(sink);

    info!("service started");
    error!(user_id = 42, reason = "invalid password", "authentication failed");
}
```

---

## Конфигурация слоя

По умолчанию используется разумная конфигурация буфера и батчинга. При необходимости можно переопределить её через `LayerConfig`:

```rust
use std::sync::Arc;
use tokio::time::Duration;
use tracing_log_sink::init::{init_tracing_with_config, LayerConfig};
use tracing_log_sink::noop_sink::NoopSink;

#[tokio::main]
async fn main() {
    let sink = Arc::new(NoopSink::default());

    let cfg = LayerConfig {
        channel_buffer: 10_000,
        batch_size: 500,
        flush_interval: Duration::from_millis(500),
        enable_stdout: true,
    };

    init_tracing_with_config(sink, cfg);
}
```

- `channel_buffer` — размер внутреннего канала; при переполнении события начинают дропаться.
- `batch_size` — сколько записей отправлять в sink за раз.
- `flush_interval` — максимальный интервал между форс‑флашами, даже если батч ещё не полный.
- `enable_stdout` — если `true`, поверх `ErrorLogLayer` добавляется `fmt`‑слой и события печатаются в консоль; если `false`, логи уходят только во внешний sink (БД и т.п.).

---

## Встроенный ClickHouse backend

Фича `clickhouse` включает реализацию `ClickHouseSink`, которая пишет в ClickHouse через HTTP в формате `JSONEachRow`.

Пример (отдельная таблица под сервис):

```rust
use std::sync::Arc;
use tracing::{info, error};
use tracing_log_sink::clickhouse::{ClickHouseConfig, ClickHouseSink};
use tracing_log_sink::init::init_tracing;

#[tokio::main]
async fn main() {
    let cfg = ClickHouseConfig {
        url: "http://127.0.0.1:8123".into(),
        database: "default".into(),
        table: "service_logs".into(),
        service_name: Some("auth-service".into()),
        user: Some("default".into()),
        password: None,
    };

    let sink = Arc::new(ClickHouseSink::new(cfg));
    init_tracing(sink);

    info!("service started");
    error!(user_id = 42, reason = "invalid password", "authentication failed");
}
```

### Примеры миграций ClickHouse

В каталоге `migrations/clickhouse` лежат SQL‑скрипты для двух схем:

- `per_service.sql` — отдельная таблица на каждый сервис
- `shared_table.sql` — общая таблица для всех сервисов с полем `service_name`

См. также примеры:

- `examples/per_service.rs`
- `examples/shared_table.rs`

---

## Пример собственного backend: Postgres

Чтобы отправлять логи в свою БД, нужно реализовать трейт `LogSink`.

Готовый рабочий пример для PostgreSQL и `sqlx` находится в файле:

- `examples/postgres.rs`

Минимальная миграция для таблицы `error_logs` находится здесь:

- `migrations/postgres.sql`

Алгоритм интеграции:

1. Создаёте свою реализацию `LogSink` (можно взять `PostgresSink` из `examples/postgres.rs` как шаблон).
2. Применяете миграцию под свою БД (или адаптируете `migrations/postgres.sql`).
3. В сервисе создаёте `Arc<dyn LogSink>` и передаёте его в `init_tracing` / `init_tracing_with_config`.

Тот же подход можно использовать для любой другой БД или системы логирования (Loki, Elasticsearch, Kafka и т.д.) — достаточно реализовать `LogSink`.

---

## Запуск примеров

```powershell
# Basic performance with NoopSink
cargo run --example default_load

# Custom layer settings
cargo run --example custom_load

# ClickHouse: per-service dedicated table
cargo run --example per_service --features clickhouse

# ClickHouse: shared table for all services
cargo run --example shared_table --features clickhouse
```
