# haruhi-db

统一管理后端的 SQLite 连接池：**每个业务模块一个独立库 + 一个 core 库**，全部启用 WAL，并负责执行迁移。是 `haruhi-server` 装配 `AppState` 时的数据访问基座。

## 技术栈 / 关键依赖

- [`sqlx`](https://docs.rs/sqlx)：`SqlitePool` 连接池，运行时校验查询（全仓约定不使用 `query!` 宏，故无需 `DATABASE_URL`）；`sqlx::migrate!` 编译期内嵌迁移目录。
- `haruhi-core`：提供 `Config`，由它解析 `data_dir` 与 `db_path(name)`。
- `tokio`（异步运行时、`fs::create_dir_all`）、`tracing`（迁移日志）、`anyhow`（错误传播）。

## 目录结构

本 crate 只有一个文件，所有逻辑都在 `src/lib.rs`：

```
backend/crates/db/
├── Cargo.toml
└── src/lib.rs    # Pools 结构体 + open_pool / connect / migrate
```

迁移 SQL **不在本 crate 内**，而是放在 workspace 的 `backend/migrations/<module>/`（相对本 crate 为 `../../migrations/<module>/`），现有：

```
migrations/{core,news,art,exam,novel,shop}/0001_init.sql
```

## 对外 API

- `pub struct Pools { core, news, art, exam, novel, shop }`：6 个 `SqlitePool`，`#[derive(Clone)]`，放进 axum `AppState` 共享。
- `open_pool(path: &Path) -> anyhow::Result<SqlitePool>`：打开（缺失则创建）单个库，自动建父目录。
- `Pools::connect(cfg: &Config)`：按 `cfg.db_path("<module>")` 依次打开全部 6 个库，先确保 `data_dir` 存在。
- `Pools::migrate(&self)`：对每个库运行其 `backend/migrations/<module>/` 下的迁移，完成后打印「数据库迁移完成」。

## 关键特性与约定

- **每模块一库**：`data/core.db` + `data/{news,art,exam,novel,shop}.db`，路径由 `Config::db_path` 拼出（`data_dir/<name>.db`，默认 `HARUHI_DATA_DIR=./data`）。`data/` 与上传目录一样在 gitignore，**永不进 git**。
- **连接选项**（见 `open_pool`）：`create_if_missing` + `journal_mode=WAL` + `synchronous=NORMAL` + `busy_timeout=10s` + `foreign_keys=ON`；池上限 8 连接、`acquire_timeout=15s`。
- **迁移内嵌**：`sqlx::migrate!("../../migrations/<module>/")` 把 SQL 在编译期嵌入二进制，运行时按版本表幂等执行，无需外部文件或 `DATABASE_URL`。
- **WAL 刷盘不在此处**：优雅停机时的 `PRAGMA wal_checkpoint(TRUNCATE)` 与 `pool.close()` 由 `haruhi-server` 的 `main.rs` 负责（本 crate 只建池）。

## 与上下游的关系

- 上游：`haruhi-core::Config` 决定库路径与数据目录。
- 下游：`haruhi-server` 在启动时 `Pools::connect(&cfg).await?` 再 `pools.migrate().await?`，把 `Pools` 存入 `AppState`；各模块业务代码从 `state.pools.<module>` 取对应库的连接池查询；后台鉴权也读 `pools.core`（RBAC 表所在）。

## 新增模块时

为新模块加一个库：在 `Pools` 加一个字段、在 `connect` 里 `open_pool(&cfg.db_path("<module>"))`、在 `migrate` 里加一行 `sqlx::migrate!("../../migrations/<module>/")`，并在 `backend/migrations/<module>/` 放 `0001_init.sql`。完整端到端流程见根仓 `docs/ADDING_MODULE.md`（`novel` 为模板）。

## 更多

- 根 README：[`../../../README.md`](../../../README.md)
- 贡献指南：[`../../../CONTRIBUTING.md`](../../../CONTRIBUTING.md)
- 提交 scope 用 `db`。
