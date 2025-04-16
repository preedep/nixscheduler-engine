# NiX Scheduler-Engine
## NiX Scheduler-Engine is scheduler engine for Rust (Enterprise Grade).

A high-performance, pluggable, and scalable job scheduler engine written in Rust 🚀  
Supports cron expressions, priority timer wheel, sharded execution, and extensible task plugins.

---

## 🛠 Features

- ✅ **Cron Expression Support** (via `cron` crate)
- ✅ **Priority Timer Wheel** using `BinaryHeap` + `tokio::time`
- ✅ **Task Plugin System** (e.g. `print`, `http`, `shell`, WASM-ready)
- ✅ **Sharding Support**
    - Local shard (multi-thread)
    - Distributed shard (multi-node)
- ✅ **Dependency Injection** for clean architecture
- ✅ **SQLite-backed JobStore** (PostgreSQL-ready)
- ✅ **Compile-time SQL Check** (optional via `sqlx-cli`)
- ✅ Easily testable and maintainable

---

## 🧱 Project Structure

```text
src/
├── main.rs                # Entry point
├── config.rs              # AppConfig with DI
├── engine/                # JobEngine orchestration logic
├── job/                   # Job model + persistent store
├── scheduler/             # Timer wheel + priority queue
├── shard/                 # Local / Distributed sharding logic
├── task/                  # TaskHandler, Registry, Plugins
```

---

## 📦 Quick Start

```bash
# Clone & run
git clone https://github.com/yourname/rust-scheduler-engine.git
cd rust-scheduler-engine

# Run with SQLite
cargo run
```

Set `.env` for distributed config:

```env
SHARD_MODE=distributed
SHARD_ID=0
TOTAL_SHARDS=4
DATABASE_URL=sqlite://./data/jobs.db
```

---

## 📌 Mermaid Architecture Diagram

```mermaid
flowchart TD
    subgraph CoreEngine [Scheduler Engine]
        C1[JobEngine]
        C2[Scheduler (Priority Wheel)]
        C3[TaskRegistry]
        C4[JobStore (SQLite)]
        C5[ShardManager]
    end

    subgraph TaskPlugins
        T1[PrintTask]
        T2[HttpTask]
        T3[ShellTask]
    end

    C1 --> C2
    C1 --> C3
    C1 --> C4
    C1 --> C5

    C3 --> T1
    C3 --> T2
    C3 --> T3

    C2 -->|tick| TaskExecution[Execute Job via TaskHandler]
    TaskExecution --> C4
```

---

## 💡 Example Job Format

```sql
CREATE TABLE jobs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    cron TEXT NOT NULL,
    task_type TEXT NOT NULL,
    payload TEXT,
    last_run TEXT
);
```

```json
{
  "id": "job-123",
  "name": "print-hello",
  "cron": "* * * * * * *",
  "task_type": "print",
  "payload": "Hello from Rust!",
  "last_run": null
}
```

---

## 📚 Extending with Your Own Task

1. Implement `TaskHandler` trait.
2. Register it in `TaskRegistry`.

```rust
registry.register(MyCustomTask {});
```

---

## ✅ Todo & Enhancements

- [ ] Task Timeout & Retry Policy
- [ ] WASM Plugin Runtime
- [ ] REST API with Actix or Axum
- [ ] Dashboard UI for Monitoring
- [ ] Clustered Leader Election via etcd or Redis

---

## 📜 License

MIT
