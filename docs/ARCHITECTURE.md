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
      ├─ /api/health        存活探针                              │
      ├─ /api/health/ready  就绪探针（探 core.db: SELECT 1）        │
      ├─ /api/auth/*     auth_routes.rs   登录 / me                │
      ├─ /api/admin/*    admin_routes.rs  超管控制台后端            │
      └─ /api/<module>/* modules/mod.rs   各业务模块               │
           news / art / exam / novel / shop                       │
                                                                  ▼
   共享 crate                          每模块一个 SQLite 库（WAL）+ core.db
   ┌──────────────────────────┐        data/core.db   用户 / 角色 / 审计
   │ core   配置/错误/响应/parse │        data/news.db   data/art.db
   │ db     连接池(WAL)+迁移     │        data/exam.db   data/novel.db
   │ auth   JWT+argon2+RBAC     │        data/shop.db
   │ media  上传/WebP/EPUB/音频  │        uploads/<module>/...   静态上传
   │ ai     DashScope 内容审核   │
   │ mail   SMTP/Resend 邮件发送 │
   └──────────────────────────┘
```

## 2. 后端：单进程 + 多模块

入口 `backend/crates/server/src/main.rs`：加载配置 → 打开连接池 → 跑迁移 → seed 超管
（`seed::seed_superadmin`）→ 启动 shop 邮件队列 worker（`modules::shop::spawn_email_worker`）→
装配路由 → `axum::serve(...).with_graceful_shutdown(...)`。

### 路由装配（`routes.rs`）

- 所有 API 统一前缀 **`/api/<module>/*`**（取代旧站各异的 `/blog-api` `/art-api` `/exam/api` `/shop-api`）。
- 顶层 `router(state)` 先组 `/api` 子路由（`/health` 存活 + `/health/ready` 就绪 + `auth_routes` + `admin_routes`），
  再由 `modules::mount(api)` 逐个 `nest` 业务模块：

  ```
  /api/novel  → modules/novel.rs::router()
  /api/art    → modules/art.rs
  /api/news   → modules/news.rs
  /api/exam   → modules/exam.rs
  /api/shop   → modules/shop/mod.rs   （shop 是目录，拆成 products/orders/coupons/email/… 子模块）
  ```
- `/uploads` 由 `ServeDir`（`nest_service`）静态服务（生产改由 Nginx 直接 `alias`，绕过后端）。
- `DefaultBodyLimit::max(256 MiB)` —— 覆盖 axum 默认 2MB，容纳大 EPUB / 图片 / 音频；与 Nginx `client_max_body_size 256m` 对齐。
- `TraceLayer` 统一请求日志；CORS 层见 §6。

每个模块 `router()` 返回 `Router<AppState>`，公开接口 `GET /xxx`、后台接口 `POST/PATCH/DELETE /admin/xxx`，
handler 内调 `authorize(...)` 做 RBAC（见 §5）。`novel.rs` 是最小、端到端的复刻模板（新增模块照抄它，见 `docs/ADDING_MODULE.md`）。

### 共享 crate 职责（`backend/crates/*`，Cargo workspace 见根 `Cargo.toml`）

| crate | 职责 |
|---|---|
| `core` | 强类型配置 `Config::from_env()`（`core/src/config.rs`，fail-fast）、统一错误 `AppError`/`AppResult`、JSON 响应；`core/src/parse.rs` 数值/文本解析小工具（此前 news/art/exam/shop 各写一份，现统一） |
| `db`   | `Pools`：每模块一个 `SqlitePool` + `core`，全部 WAL；`connect()` / `migrate()`（`db/src/lib.rs`）|
| `auth` | JWT 签发/校验、argon2 密码、RBAC（`authorize`/`scope_chain`/`require_super`）、axum 提取器 `AuthUser` |
| `media`| 上传落盘、图片转 WebP、EPUB 解析取封面、音频处理（`media/src/{upload,image_ops,epub_ops,audio}.rs`）|
| `ai`   | DashScope（OpenAI 兼容）文本/图片内容审核；未配 `DASHSCOPE_API_KEY` 或出错时 fail-open 放行，由业务层据 reason 决策 |
| `mail` | 统一邮件**发送**（Resend API + SMTP 双驱动，`mail/src/lib.rs`）；只负责“把一封邮件发出去”，**队列在 shop 模块** |

> 邮件分层：`mail` crate 只封装发送驱动；订单邮件的 `email_jobs` 队列、退避重试、后台 worker
> 由 shop 模块在其库上实现（`modules/shop/email.rs`，`spawn_email_worker`），`Mailer` 为 `None`（未配置）时 worker 空转、job 留在队列等配置就绪。

### sqlx 约定

运行时校验查询（`query`/`query_as`，**不用 `query!` 宏**），因此构建无需 `DATABASE_URL`/离线缓存——
CI（`.github/workflows/ci.yml`）可直接 `cargo build --workspace --release`。handler 多用元组承接行
（crate 级 `#![allow(clippy::type_complexity)]`，见 `main.rs` 注释），忠实移植旧后端、不为每个查询造 DTO。

## 3. 前端：多 app + 共享 api-client

- pnpm workspace（`pnpm-workspace.yaml`：`apps/*` + `packages/*`）。
- 每个 app 独立 Vite 构建，`base` 指向子路径；dev 模式各 app 把 `/api` 与 `/uploads`
  代理到 `127.0.0.1:17777`（见各 `vite.config.*`）。完整映射：

  | app | 包名 | dev 端口 | 部署子路径 | 主要技术栈 |
  |---|---|---|---|---|
  | news    | `@haruhi/news`    | 5204 | `/news/`    | Vue3 + Pinia3 + 原生 CSS |
  | art     | `@haruhi/art`     | 5201 | `/art/`     | Vue3 + Pinia3 + CSS vars |
  | exam    | `@haruhi/exam`    | 5202 | `/exam/`    | Vue3 + **TS** + SCSS（build 含 `vue-tsc`） |
  | novel   | `@haruhi/novel`   | 5203 | `/library/` | Vue3 + **Tailwind**（无 Pinia，epubjs 阅读器） |
  | shop    | `@haruhi/shop`    | 5205 | `/shop/`    | Vue3 + 纯 `reactive` store（无 Pinia） |
  | console | `@haruhi/console` | 5200 | `/console/` | Vue3 + **TS**（走 `/api/admin/*`，build 含 `vue-tsc`） |

- 全仓已对齐单版本工具链：**Vite 7 + Pinia 3**（用到 Pinia 的 news/art/exam 统一 `pinia@^3`）；
  各 app `scripts` 取自其 `package.json`（一般 `dev`/`build`/`preview`，TS app 的 `build` 先跑 `vue-tsc --noEmit`）。
- `packages/api-client`（`@haruhi/api-client`，workspace 依赖，被 6 个 app 全部依赖；扁平 `index.js` + `index.d.ts`）统一封装：
  - `createApiClient(base)` —— 带 token 的 `get/post/put/patch/del/postForm`。
  - `createAuth()` / `createAdminAuth(app)` —— 登录、`me()`、`getToken/setToken/clearToken` 存取、按 app 判权。
  - `hasScope(user, scope)` —— 前端权限判断，**与后端同款父级继承**（见 §5）。
  - `resolveUploadUrl(path, base?)` —— 上传路径→可访问 URL 的拼接，原本 art/novel 各写一份，已抽到此处去重（art 的 `services/api.js`、novel 的 Admin/Shelf/Reader 视图在用）。
- `packages/ui`、`packages/config` 目前为**空 stub**（无 `package.json`、未启用），是预留位，不要在文档里当作已有能力描述。

## 4. 数据隔离：每模块独立 SQLite + core.db

`db/src/lib.rs` 的 `Pools` 为 `core/news/art/exam/novel/shop` 各开一个库：

- **写锁隔离**：SQLite 单写者，分库后某模块高频写不阻塞其它模块。
- **迁移零转换**：每个旧站本就是独立 sqlite，迁移时几乎原样拷贝（见 `DEPLOYMENT.md` 的数据迁移）。
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
| **就绪探针** | `/api/health/ready` 对 `core.db` 跑 `SELECT 1`，通则 `ready`、断则 503；供负载/巡检判断是否真正可服务（`/api/health` 只判进程存活） | `routes.rs::ready` |
| **登录限流** | per-IP 滑动窗口，单 IP 10 分钟最多 10 次；成功即清零。单进程内存，无需 Redis。取 IP 优先 `X-Forwarded-For` 首个再退 `X-Real-IP`（适配 Nginx） | `ratelimit.rs`，`main.rs` `LoginLimiter::new(10, 600)` |
| **优雅关闭** | 监听 `SIGTERM`(systemd) / Ctrl-C；停止收新请求 → 等在途结束 → 每个库 `PRAGMA wal_checkpoint(TRUNCATE)` + `pool.close()` 刷 WAL → 安全退出。配合 service 的 `TimeoutStopSec=30` | `main.rs` |
| **CORS** | debug 放宽 `Any`（本地方便）；release 限定来源（`HARUHI_CORS_ORIGINS`，逗号分隔；留空默认仅 `public_site_url`） | `routes.rs::build_cors` |
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

## 8. 工程化与协作基建

近期已统一工具链与协作流程，文档/PR 评审都有据可循：

- **单版本前端工具链**：全仓对齐 Vite 7 + Pinia 3（exam/console 也已并入），上传 URL 拼接抽到
  api-client 的 `resolveUploadUrl`（art/novel 去重），数值/文本解析抽到 `core::parse`。
- **协作文档**：`CONTRIBUTING.md`；新增模块流程见 `docs/ADDING_MODULE.md`（novel 为端到端模板）；部署/数据迁移见 `docs/DEPLOYMENT.md`。
- **CI/质量门禁**：`.github/workflows/`（`ci.yml` 路径过滤 + `ci-ok` 汇聚 gate、`pr-checks.yml` 校验 PR 标题、`labeler.yml`）；
  `.coderabbit.yaml`（自动评审）、`.github/dependabot.yml`（依赖更新）。
- **提交规范**：约定式提交，仅 `type` 受约束，`scope` 自由、不在代码里维护集合；因 squash 合并，CI 只校验 PR 标题（详见 `CONTRIBUTING.md`）。

## 更多

- 顶层概览与本地起步：根 `README.md`
- 贡献流程与提交规范：`CONTRIBUTING.md`
- 加新模块（以 novel 为模板）：`docs/ADDING_MODULE.md`
- 部署与数据迁移：`docs/DEPLOYMENT.md`
