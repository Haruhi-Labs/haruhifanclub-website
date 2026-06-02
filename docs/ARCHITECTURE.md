# 系统架构

> 凉宫春日应援团统一站点。**单一 Rust(axum) 后端进程** + **多个独立 pnpm 前端 app**，
> 全部挂在 `haruyuki.cn` 的子路径下，统一鉴权与工程化。
> 概览见根 `README.md`；本文档讲清楚“为什么这样拆”和“数据怎么流”。

## 1. 文字版架构图

```
                        浏览器（haruyuki.cn）
                                │
                   ┌────────────┴─────────────┐
                   │   Nginx 反向代理 / 静态托管  │   deploy/nginx.conf
                   └────────────┬─────────────┘
        静态文件 ┌──────────────┼──────────────┐ /api/*  /uploads/*
                 ▼              ▼              ▼        反代到 127.0.0.1:17777
        apps/news/dist   apps/art/dist   apps/...     ┌──────────────────────┐
        → /news/         → /art/         /library/    │  haruhi-server (axum) │
        → /console/      → /shop/        /exam/       │   单进程，systemd 守护  │
                                                      └───────────┬──────────┘
                                                                  │ 装配
   backend/crates/server/src/routes.rs  →  /api 路由器             │
      ├─ /api/health                                              │
      ├─ /api/auth/*     auth_routes.rs   登录 / me                │
      ├─ /api/admin/*    admin_routes.rs  超管控制台后端            │
      └─ /api/<module>/* modules/mod.rs   各业务模块               │
           news / art / exam / novel / shop                       │
                                                                  ▼
   共享 crate                          每模块一个 SQLite 库（WAL）+ core.db
   ┌──────────────────────────┐        data/core.db   用户 / 角色 / 审计
   │ core   配置/错误/响应       │        data/news.db   data/art.db
   │ db     连接池(WAL)+迁移     │        data/exam.db   data/novel.db
   │ auth   JWT+argon2+RBAC     │        data/shop.db
   │ media  上传/WebP/EPUB/音频  │        uploads/<module>/...   静态上传
   │ ai     DashScope 内容审核   │
   │ mail   SMTP/Resend 邮件队列 │
   └──────────────────────────┘
```

## 2. 后端：单进程 + 多模块

入口 `backend/crates/server/src/main.rs`：加载配置 → 打开连接池 → 跑迁移 → seed 超管 →
启动 shop 邮件 worker → 装配路由 → `axum::serve(...).with_graceful_shutdown(...)`。

### 路由装配（`routes.rs`）

- 所有 API 统一前缀 **`/api/<module>/*`**（取代旧站各异的 `/blog-api` `/art-api` `/exam/api` `/shop-api`）。
- `Router::nest("/api", ...)`；其中 `/api/<module>` 由 `modules::mount()` 逐个 `nest`：

  ```
  /api/novel  → modules/novel.rs::router()
  /api/art    → modules/art.rs
  /api/news   → modules/news.rs
  /api/exam   → modules/exam.rs
  /api/shop   → modules/shop/mod.rs   （shop 是目录，拆成 products/orders/coupons/… 子模块）
  ```
- `/uploads` 由 `ServeDir` 静态服务（生产改由 Nginx 直接 `alias`，绕过后端）。
- `DefaultBodyLimit::max(256 MiB)` —— 覆盖 axum 默认 2MB，容纳大 EPUB / 图片 / 音频；与 Nginx `client_max_body_size 256m` 对齐。
- `TraceLayer` 统一请求日志；CORS 层见 §6。

每个模块 `router()` 返回 `Router<AppState>`，公开接口 `GET /xxx`、后台接口 `POST/PATCH/DELETE /admin/xxx`，
handler 内调 `authorize(...)` 做 RBAC（见 §5）。`novel.rs` 是最小、端到端的复刻模板。

### 共享 crate 职责（`backend/crates/*`，Cargo workspace 见根 `Cargo.toml`）

| crate | 职责 |
|---|---|
| `core` | 强类型配置 `Config::from_env()`（`core/src/config.rs`，fail-fast）、统一错误 `AppError`/`AppResult`、JSON 响应 |
| `db`   | `Pools`：每模块一个 `SqlitePool` + `core`，全部 WAL；`connect()` / `migrate()`（`db/src/lib.rs`）|
| `auth` | JWT 签发/校验、argon2 密码、RBAC（`authorize`/`scope_chain`/`require_super`）、axum 提取器 `AuthUser` |
| `media`| 上传落盘、图片转 WebP、EPUB 解析取封面、音频处理（`media/src/{upload,image_ops,epub_ops,audio}.rs`）|
| `ai`   | DashScope（OpenAI 兼容）文本/图片内容审核；未配 `DASHSCOPE_API_KEY` 时离线放行 |
| `mail` | SMTP / Resend 邮件发送（shop 下单通知），`shop` 模块内有 `email_jobs` 队列 + worker |

### sqlx 约定

运行时校验查询（`query`/`query_as`，**不用 `query!` 宏**），因此构建无需 `DATABASE_URL`/离线缓存——
CI（`.github/workflows/ci.yml`）可直接 `cargo build --workspace --release`。handler 多用元组承接行
（`#![allow(clippy::type_complexity)]`，见 `main.rs` 注释），忠实移植旧后端、不为每个查询造 DTO。

## 3. 前端：多 app + 共享 api-client

- pnpm workspace（`pnpm-workspace.yaml`：`apps/*` + `packages/*`）。
- 每个 app 独立 Vite 构建，`base` 指向子路径：`news`→`/news/`、`art`→`/art/`、`exam`→`/exam/`、
  `novel`→`/library/`、`shop`→`/shop/`、`console`→`/console/`（见各 `vite.config.ts`）。
- dev 模式各 app 把 `/api` 与 `/uploads` 代理到 `127.0.0.1:17777`（端口示例：console 5200、exam 5202、novel 5203）。
- `packages/api-client`（`@haruhi/api-client`，workspace 依赖）统一封装：
  - `createApiClient(base)` —— 带 token 的 `get/post/put/patch/del/postForm`。
  - `createAuth()` / `createAdminAuth(app)` —— 登录、`me()`、token 存取、按 app 判权。
  - `hasScope(user, scope)` —— 前端权限判断，**与后端同款父级继承**（见 §5）。

## 4. 数据隔离：每模块独立 SQLite + core.db

`db/src/lib.rs` 的 `Pools` 为 `core/news/art/exam/novel/shop` 各开一个库：

- **写锁隔离**：SQLite 单写者，分库后某模块高频写不阻塞其它模块。
- **迁移零转换**：每个旧站本就是独立 sqlite，迁移时几乎原样拷贝（见 DEPLOYMENT.md 的数据迁移）。
- 每库 `open_pool()` 统一：`create_if_missing` + `WAL` + `synchronous=Normal` + `busy_timeout=10s` +
  `foreign_keys=on`，连接池 `max_connections=8`。
- `core.db` 只放跨模块的鉴权数据：`users` / `roles` / `user_app_roles` / `audit_log`（`migrations/core/0001_init.sql`）。

## 5. 统一鉴权：JWT + RBAC（含分层子作用域）

身份**无状态**（JWT），权限**每次查库**（`core.db`），保证改角色即时生效。核心在 `backend/crates/auth/src/lib.rs`。

### 登录与令牌

`POST /api/auth/login`（`auth_routes.rs`）→ 校验 argon2 密码 → `issue_token()` 签 JWT
（`Claims{ sub=user_id, super, iat, exp }`，TTL 默认 86400s）。前端把 token 存本地，请求带
`Authorization: Bearer <token>`。提取器 `AuthUser`（`FromRequestParts`）解析 token 得 `{id, is_super}`；
另有 `OptionalFromRequestParts` 供“可选登录”端点（游客可读、登录可见更多）。

### 角色模型（`core.db`）

- 预置 4 个角色，带数值等级：`viewer(1) < editor(2) < moderator(3) < admin(4)`。
- `user_app_roles(user_id, app, role_id)`：每个用户**在每个 app 至多一个角色**。
- `Action` → 所需最小等级：`Read=1 / Write=2 / Moderate=3 / Manage=4`。

### `authorize()` 判定流程

```
authorize(core, user, app, action):
    if user.is_super: 通过                         # 超管全通过
    if users.status != 'active': Forbidden          # 账号须 active
    best = max over scope in scope_chain(app):       # 取作用域链上最高角色等级
               role_level(core, user.id, scope)
    通过 当且仅当 best >= action.level()
```

### `news.*` 分层子作用域 / 父级继承

`scope_chain(app)` 把点分作用域从具体到顶层展开，逐级查角色取最高等级：

```
scope_chain("news.activity") == ["news.activity", "news"]
scope_chain("news")          == ["news"]
```

含义：**给某人分配父作用域 `news` 的 `admin`，他对所有 `news.*` 子作用域自动有效**；
也可只给细粒度 `news.blog` 的 editor 而不碰 activity/store/points。news 模块的 handler 据此分四个子域鉴权：

| 子作用域 | 覆盖 | 典型动作等级 |
|---|---|---|
| `news.blog`     | 文章 | 公开读 / `Write` 发改 / `Manage` 删 |
| `news.activity` | 活动 | `Manage` |
| `news.store`    | 奖品/积分商城 | `Manage` |
| `news.points`   | 积分 | `Read` 查 / `Manage` 改 |

其它模块（art/exam/novel/shop）目前用单层作用域，handler 直接 `authorize(..., "<module>", Action::X)`。

前端 `hasScope(user, "news.activity")` 用**同样的父级继承规则**判断按钮/菜单可见性——和后端判定一致，
不会出现“前端显示得了、后端拒绝”的错配。超管旁路（`require_super`）用于 `/api/admin/*` 控制台后端
（用户增删改、改密、启停、按 app 分配角色，见 `admin_routes.rs`）；其中 `APPS` 常量列出全部可分配作用域
（含 `news.blog/activity/store/points` 四个子域）。敏感操作写 `audit_log`。

### 控制台（console）

超管在 `/console/` 给管理员**按 app 分配角色**（viewer/editor/moderator/admin）。`auth_routes::me()`
返回 `apps: { <app>: {role, roleName, level} }` 权限矩阵，前端据此渲染各后台的可用功能。

## 6. 健壮性设计

| 关注点 | 实现 | 位置 |
|---|---|---|
| **登录限流** | per-IP 滑动窗口，单 IP 10 分钟最多 10 次；成功即清零。单进程内存，无需 Redis。取 IP 优先 `X-Forwarded-For` 首个再退 `X-Real-IP`（适配 Nginx） | `ratelimit.rs`，`main.rs` `LoginLimiter::new(10, 600)` |
| **优雅关闭** | 监听 `SIGTERM`(systemd) / Ctrl-C；停止收新请求 → 等在途结束 → 每个库 `PRAGMA wal_checkpoint(TRUNCATE)` + `pool.close()` 刷 WAL → 安全退出。配合 service 的 `TimeoutStopSec=30` | `main.rs` |
| **CORS** | debug 放宽 `Any`（本地方便）；release 限定来源（`HARUHI_CORS_ORIGINS`，逗号分隔；留空默认仅 `PUBLIC_SITE_URL`） | `routes.rs::build_cors` |
| **密钥 fail-fast** | release 缺 `HARUHI_JWT_SECRET` 或 `ART_COOKIE_SECRET` 直接拒绝启动；debug 用不安全默认值 + 告警 | `core/src/config.rs` |
| **体积上限** | 后端 256MiB + Nginx `client_max_body_size 256m` 对齐 | `routes.rs` / `nginx.conf` |
| **DB 并发** | WAL + `busy_timeout 10s` + 连接池；写多读快不互锁 | `db/src/lib.rs` |
| **审计** | 控制台敏感操作写 `audit_log`（失败仅告警不阻断） | `admin_routes.rs` |

## 7. 请求/鉴权数据流（示例：上传一本书）

```
前端 novel admin  ──POST /api/novel/admin/upload (multipart, Bearer token)──▶ Nginx
  └─ Nginx 反代 127.0.0.1:17777，注入 X-Forwarded-For
       └─ routes.rs：/api → modules::mount → nest /novel → novel::upload
            ├─ AuthUser 提取器：解析 Bearer JWT → {id, is_super}
            ├─ authorize(core, user, "novel", Action::Write)：查 core.db 角色等级
            ├─ 解析 multipart → media::save_file 落盘 uploads/novel/files/
            ├─ media::read_epub（spawn_blocking）取标题/作者/封面 → 封面转 WebP
            └─ INSERT INTO books（data/novel.db）→ 返回 {success, id, title}
```
