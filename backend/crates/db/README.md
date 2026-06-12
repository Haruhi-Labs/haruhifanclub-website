# haruhi-db

SQLite 连接池和 schema 管理库。`haruhi-server` 启动时用它打开所有数据库并执行 schema 更新。

## 数据库

```text
data/core.db
data/<module>.db
```

`core.db` 存用户、角色和审计。业务模块各用自己的库。

## 主要 API

| API                   | 说明                     |
| --------------------- | ------------------------ |
| `open_pool(path)`     | 打开或创建 SQLite 库     |
| `Pools::connect(cfg)` | 按配置打开全部连接池     |
| `Pools::migrate()`    | 对全部库执行 schema 更新 |
| `Pools`               | 已登记数据库的连接池集合 |

## 连接选项

`open_pool()` 设置：

- `create_if_missing(true)`
- WAL
- `synchronous=NORMAL`
- `busy_timeout=10s`
- `foreign_keys(true)`
- `max_connections(8)`
- `acquire_timeout(15s)`

## schema SQL

schema SQL 在：

```text
backend/migrations/<module>/
```

`Pools::migrate()` 通过 `sqlx::migrate!("../../migrations/<module>")` 把 SQL 目录编译进二进制。运行时会按 sqlx 的版本表幂等执行。

`build.rs` 声明 `cargo:rerun-if-changed=migrations`，新增/修改迁移文件会触发本 crate 重建。否则 `sqlx::migrate!` 的编译期嵌入不会随新文件失效，增量构建会静默漏掉新迁移（典型症状：查询报 `no such column`）。新增迁移后无需 `cargo clean`。

库名由 `Config::db_path(<module>)` 和 `Pools` 字段决定。新增独立库时按下一节登记。

## 不负责的事

- 不定义业务查询。
- 不做 WAL checkpoint；关停刷 WAL 在 `haruhi-server/src/main.rs`。
- 不读取 `.env`；路径来自 `haruhi-core::Config`。

## 新增模块

新增独立库时改三处：

```rust
pub widget: SqlitePool
widget: open_pool(&cfg.db_path("widget")).await?
sqlx::migrate!("../../migrations/widget").run(&self.widget).await?
```

同时新增 `backend/migrations/widget/0001_init.sql`。详细步骤见 [../../../docs/ADDING_MODULE.md](../../../docs/ADDING_MODULE.md)。

## 开发

```bash
cargo build -p haruhi-db
cargo test -p haruhi-db
```
