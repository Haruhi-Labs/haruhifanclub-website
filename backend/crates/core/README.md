# haruhi-core

haruhifanclub 后端的**地基库**：把全后端共用的「强类型配置」「统一错误类型」「数值/文本解析工具」收在一处。被其余所有 crate（`server`/`db`/`auth`/`media`/`ai`/`mail`）复用，自己不含任何业务逻辑、不监听端口、不连数据库。

## 技术栈与关键依赖

- 纯 library crate（`Cargo.toml` 无 `[[bin]]`，无独立运行入口）。
- Rust 2021 / rust-version 1.80（继承 workspace）。
- 依赖均取自 workspace：`serde` / `serde_json`、`thiserror`（错误派生）、`anyhow`（`Config::from_env` 返回值）、`axum`（错误类型实现 `IntoResponse`）、`sqlx`（`AppError` 内嵌 `sqlx::Error`）、`chrono`、`tracing`（缺密钥/弱密钥时告警、5xx 落日志）。

## 目录结构（`src/`）

- `lib.rs` — crate 门面，`pub mod config/error/parse`，并 re-export `Config`、`MailConfig`、`AppError`、`AppResult`。
- `config.rs` — `Config` / `MailConfig` 两个结构体，从环境变量加载（`Config::from_env`），含 `db_path(name)`、`uploads_subdir(module)` 路径辅助。
- `error.rs` — `AppError` 枚举 + `AppResult<T>` 别名，统一 JSON 错误响应。
- `parse.rs` — 数值/文本解析小工具，内含单元测试。

## 本地开发

本 crate 不单独运行，跟随 workspace 编译/测试。在仓库根目录：

```bash
cargo build -p haruhi-core    # 单独编译本 crate
cargo test  -p haruhi-core    # 跑 parse.rs 内的单测
cargo run   -p haruhi-server  # 实际使用方：启动后端，监听 127.0.0.1:17777
```

`Config::from_env` 假定调用方已先执行 `dotenvy::dotenv()`（由 `haruhi-server` 负责），本 crate 不读 `.env`。

## 关键特性与约定

### Config（env 映射 + release fail-fast）

- `from_env()` 返回 `anyhow::Result<Config>`，逐项从环境变量解析，缺省值就近写在代码里。覆盖范围：绑定地址（`HARUHI_BIND`，默认 `127.0.0.1:8080`）、`data_dir`/`uploads_dir`、JWT（secret/TTL/超管账号）、AI 审核（DashScope 兼容地址与模型名）、art 匿名 Cookie 签名密钥、shop 包邮门槛、邮件（`MailConfig`，provider=`resend|smtp|auto`）、`public_site_url`、CORS 来源列表。
- **fail-fast**：`HARUHI_JWT_SECRET`、`ART_COOKIE_SECRET` 在 release 构建缺失时 `bail!` 直接报错；debug 构建退回不安全默认值（其中缺 `HARUHI_JWT_SECRET` 时还会 `tracing::warn!` 提示），并默认 seed 一个 `admin/admin123` 超管，方便本地 `cargo run` 开箱即用。JWT 密钥过短（<16 位）也会告警。
- 解析辅助 `env/env_or/env_parse/env_bool` 把空白值视同未设置；布尔接受 `true/1/yes/on` 与 `false/0/no/off`。
- 路径辅助：`db_path("news") → data/news.db`，`uploads_subdir("novel") → uploads/novel`，与「每模块一库 + 上传分子目录」的约定对齐。

### AppError → 统一 JSON

- 所有 handler 约定返回 `AppResult<T>`；`AppError` 实现 `IntoResponse`，统一输出 `{"error": "<消息>"}`。
- 状态码映射：`BadRequest`→400、`Unauthorized`→401、`Forbidden`→403、`NotFound`→404、`Conflict`→409、`TooManyRequests`→429；`Internal` 与 `Database` 多数→500，其中 `sqlx::Error::RowNotFound` 特判为 404。
- 安全：5xx 用 `tracing::error!` 记录原始错误，但对客户端只回脱敏文案（数据库错误回「服务器内部错误」/「资源不存在」），不泄露内部细节。
- 提供 `From<sqlx::Error>`、`From<anyhow::Error>` 与 `bad_request/not_found/conflict/internal` 构造函数，便于 `?` 透传。

### parse（数值/文本工具）

此前在 news/art/exam/shop 各写一份，现统一到此处：

- `parse_int_radix10` 模拟 JS `Number.parseInt(x, 10)`（取十进制前缀）；`parse_int_or` 叠加 JS falsy 语义（0 或无效取默认值）。
- `num_i64` 从 `serde_json::Value` 兼容取整（整数/浮点/数字字符串）。
- `clamp_int`（floor 后裁剪到 `[min, max]`）、`clamp_len`（按 char 截断）、`safe_text`（trim，`None`→`""`）。

## 与共享层 / 后端的关系

- 定位为后端依赖图的最底层：`server` 装配时调用 `Config::from_env`、用 `AppError` 作为全局错误类型；`db` 用 `Config::db_path`；`media` 用 `uploads_subdir`；`auth`/`ai`/`mail` 分别消费 JWT、AI、邮件相关配置字段。
- 不对应任何前端共享包；前端侧的解析/路径逻辑各自实现（见 `packages/api-client`），二者无代码共享。

## 更多

- 仓库总览与整体架构见根 [README](../../../README.md)。
- 协作规范、提交 scope（本 crate 用 `core`）见 [CONTRIBUTING](../../../CONTRIBUTING.md) 与 `docs/`。
