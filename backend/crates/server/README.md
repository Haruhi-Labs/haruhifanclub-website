# haruhi-server

haruhifanclub 的统一后端二进制：单进程 axum 应用，装配全部 `/api/<module>/*` 路由、挂载 `/uploads/<module>/*` 静态资源，监听 `127.0.0.1:17777`。前端 6 个 app（news / art / exam / novel / shop / console）共用这一个进程。

## 技术栈与关键依赖

- `axum` + `tokio`（HTTP 服务与异步运行时）
- `tower-http`：`ServeDir`（静态上传）、`CorsLayer`、`TraceLayer`、`DefaultBodyLimit`
- `sqlx`（SQLite，运行时校验查询）、`serde` / `serde_json`、`chrono`
- `tracing` / `tracing-subscriber`、`dotenvy`、`anyhow`
- 内部 crate：`haruhi-core`（Config / AppError）、`haruhi-db`（Pools）、`haruhi-auth`（JWT / RBAC / AuthUser）、`haruhi-media`、`haruhi-ai`、`haruhi-mail`

二进制名 `haruhi-server`，入口 `src/main.rs`（见 `Cargo.toml` 的 `[[bin]]`）。

## 目录结构要点

```
src/
  main.rs          进程入口：加载 Config、连库迁移、seed 超管、启 shop 邮件 worker、优雅停机
  routes.rs        顶层路由装配：/api 嵌套、/uploads 静态、CORS、TraceLayer、body 上限
  state.rs         AppState（cfg / pools / login_limiter），并为 AuthSecret 实现 FromRef
  ratelimit.rs     per-IP 滑动窗口登录限流 + client_ip（X-Forwarded-For / X-Real-IP）
  auth_routes.rs   /api/auth/login、/api/auth/me（无状态 JWT）
  admin_routes.rs  /api/admin/*（超管控制台：用户/角色/审计，全程 require_super）
  seed.rs          首启 seed 超级管理员（仅当 core.db 无用户时）
  modules/
    mod.rs         mount()：把各业务模块挂到 /api/<module>
    news.rs art.rs exam.rs novel.rs   各模块路由
    shop/          最大模块：orders/products/coupons/admin_orders/pricing/email/misc/common
```

## 本地开发

```bash
# 仓库根目录执行
cargo run -p haruhi-server            # 监听 127.0.0.1:17777
cargo build --release -p haruhi-server
cargo test -p haruhi-server           # 含 ratelimit 单测
RUST_LOG=debug cargo run -p haruhi-server   # 调日志等级（默认 info）
```

启动前依赖环境变量（由 `haruhi-core::Config::from_env` 读取，release 下 fail-fast），关键项：`HARUHI_DATA_DIR` / `HARUHI_UPLOADS_DIR`（数据/上传目录）、`HARUHI_SUPERADMIN_USER` / `HARUHI_SUPERADMIN_PASSWORD`（首启 seed 用）、`HARUHI_CORS_ORIGINS`、`HARUHI_JWT_SECRET` / `HARUHI_JWT_TTL_SECONDS`、`HARUHI_BIND` 等。本地可用 `.env`（`dotenvy` 自动加载）。

健康探针：

- `GET /api/health` → `{"status":"ok","service":"haruhifanclub"}`
- `GET /api/health/ready` → 探测 core 库 `SELECT 1`，成功返回 `{"status":"ready"}`，不可用时返回 503

## 关键特性与约定

- **路由装配**：`routes::router` 把 health + `auth_routes` + `admin_routes` 合入 `/api`，再由 `modules::mount` 嵌套 `/novel /art /news /exam /shop`；上传目录经 `nest_service("/uploads", ServeDir)` 暴露为 `/uploads/<module>/*`。
- **优雅停机**：监听 Ctrl-C 与（Unix）SIGTERM，`with_graceful_shutdown` 等在途请求结束后，对 6 个连接池逐个执行 `PRAGMA wal_checkpoint(TRUNCATE)` 再 `close()`，确保 WAL 刷盘、数据一致。
- **登录限流**：`LoginLimiter` 为单进程内存版 per-IP 滑动窗口（默认 10 分钟内 10 次），单机部署无需 Redis；登录成功即 `reset` 清零。客户端 IP 优先取 `X-Forwarded-For` 首段（nginx 反代）。
- **CORS**：debug 构建放开 `Any` 便于本地；release 限定来源（`cfg.cors_origins`，为空时回落到 `public_site_url`）。
- **body 上限**：`DefaultBodyLimit` 抬到 256MB，覆盖 axum 默认 2MB，以容纳 EPUB / 图片 / 音频上传。
- **鉴权**：`/api/auth/*` 签发/校验无状态 JWT（登出由前端丢 token）；`/api/admin/*` 全部 `require_super`，用户增删改、改密、启停、按 app 分配角色，并写 `audit_log`。
- **元组承接行**：handler 直接用元组接 sqlx 行（忠实移植旧后端，不为每条查询造 DTO），`clippy::type_complexity` 在 crate 级有意放行。

## 与共享层 / 后端的关系

- 业务逻辑分散在各模块文件，复用 `haruhi-auth`（`authorize` / `require_super` / `AuthUser`）、`haruhi-db`（`Pools` 每模块一库 + core.db）、`haruhi-media`、`haruhi-ai`、`haruhi-mail`。
- shop 的邮件队列 worker 在 `main` 中通过 `modules::shop::spawn_email_worker` 启动（`Mailer` 为 `None` 即邮件未启用时内部空转）。
- 新增业务模块的接入方式见根目录 `docs/ADDING_MODULE.md`，`novel` 为端到端最简模板。

## 更多

- 项目总览见根 [README](../../../README.md)
- 贡献与提交规范见 [CONTRIBUTING](../../../CONTRIBUTING.md)（本 crate 的提交 scope 为 `server`）
