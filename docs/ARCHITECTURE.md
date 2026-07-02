# 架构

本文说明运行时结构、数据流和几个需要遵守的工程约定。快速启动看根 [README.md](../README.md)。

## 运行时结构

```text
浏览器
  |
  v
Nginx
  ├─ 子路径静态文件
  |    /news/     -> apps/news/dist
  |    /art/      -> apps/art/dist
  |    /exam/     -> apps/exam/dist
  |    /library/  -> apps/novel/dist
  |    /fiction/  -> apps/fiction/dist
  |    /shop/     -> apps/shop/dist
  |    /console/  -> apps/console/dist
  |    /design-system/ -> apps/design-system/dist
  ├─ /api/*       -> 127.0.0.1:17777
  └─ /uploads/*   -> uploads/<module>/

haruhi-server(axum)
  ├─ /api/health
  ├─ /api/health/ready
  ├─ /api/auth/*
  ├─ /api/admin/*
  └─ /api/<module>/*

SQLite
  ├─ data/core.db
  └─ data/{news,art,exam,novel,fiction,shop}.db
```

本地开发同样走 `/api` 与 `/uploads`，只是由 Vite dev server 代理到 `127.0.0.1:17777`。

## 后端

入口在 `backend/crates/server/src/main.rs`：

1. `dotenvy::dotenv()` 加载 `.env`。
2. `Config::from_env()` 读取配置。
3. `Pools::connect()` 打开 `core` 和各业务模块的 SQLite 连接池。
4. `Pools::migrate()` 执行 schema 更新。
5. `seed::seed_superadmin()` 在 `core.db` 无用户时创建超管。
6. 启动 shop 邮件队列 worker。
7. `routes::router(state)` 装配路由并启动 axum。
8. 收到 `SIGTERM` 或 Ctrl-C 后，等待在途请求结束，对每个库执行 `PRAGMA wal_checkpoint(TRUNCATE)` 并关闭连接池。

crate 分工：

| crate    | 职责                                                      |
| -------- | --------------------------------------------------------- |
| `server` | HTTP 路由、模块 handler、登录限流、health、CORS、关停逻辑 |
| `core`   | 配置、错误响应、解析工具                                  |
| `db`     | SQLite 连接池与 schema 更新                               |
| `auth`   | JWT、密码哈希、RBAC、axum 鉴权提取器                      |
| `media`  | 文件保存、图片 WebP、EPUB、音频转码                       |
| `ai`     | DashScope/OpenAI 兼容审核客户端                           |
| `mail`   | SMTP/Resend 单封邮件发送                                  |

## 路由

顶层路由在 `backend/crates/server/src/routes.rs`：

- `/api/health`：进程存活。
- `/api/health/ready`：对 `core.db` 执行 `SELECT 1`，失败返回 503。
- `/api/auth/*`：登录和当前用户。
- `/api/admin/*`：超管控制台接口。
- `/api/<module>/*`：业务模块，由 `modules::mount()` 挂载。
- `/uploads/*`：本地开发由后端 `ServeDir` 服务；生产由 Nginx `alias` 直接服务。

业务模块公开接口通常直接挂在模块根下，后台接口使用 `/admin/*`，例如：

```text
GET  /api/novel/books
POST /api/novel/admin/upload
GET  /api/shop/products
PATCH /api/shop/admin/orders/{id}
```

Axum 0.8 路径参数使用 `{id}`，不要写旧式 `:id`。

模块列表端点的分页元信息统一用 `server::pagination::{total_pages, page_meta}` 生成 `{page, pageSize, total, totalPages}`，避免各处重复样板。带特殊分页语义的端点（如 exam 列表非搜索首页预留 1 位）保留各自实现，不强行套用。

## 前端

前端 workspace 由 `pnpm-workspace.yaml` 管理：

```text
apps/*
packages/*
```

当前实际被 app 依赖的共享包包括 `packages/api-client`、`packages/design-system`、`packages/ui` 和 `packages/auth-ui`。`packages/config` 没有 `package.json`，属于预留目录。

| app           | 子路径            | dev 端口 | 主要特点                         |
| ------------- | ----------------- | -------- | -------------------------------- |
| news          | `/news/`          | 5204     | Pinia，内容主站和新闻后台        |
| art           | `/art/`           | 5201     | Pinia，画廊、投稿、匿名互动      |
| exam          | `/exam/`          | 5202     | TypeScript，试卷编辑、导出和审核 |
| novel         | `/library/`       | 5203     | Tailwind，EPUB 阅读器和书库后台  |
| fiction       | `/fiction/`       | 5207     | Tiptap 富文本，同人小说创作与阅读 |
| shop          | `/shop/`          | 5205     | reactive store，商城前后台       |
| console       | `/console/`       | 5200     | TypeScript，超管台               |
| design-system | `/design-system/` | 5206     | 静态设计规范页，不依赖后端 API   |

所有 Vite 配置都代理：

```js
proxy: {
  '/api': { target: 'http://127.0.0.1:17777', changeOrigin: true },
  '/uploads': { target: 'http://127.0.0.1:17777', changeOrigin: true },
}
```

因此本地 `.env` 需要设置 `HARUHI_BIND=127.0.0.1:17777`。`deploy/env.sample` 已经包含该值。

## 数据库

`haruhi-db` 为每个模块开独立 SQLite：

```text
data/core.db      用户、角色、审计
data/<module>.db
```

连接选项在 `open_pool()` 中设置：

- `create_if_missing`
- WAL
- `synchronous=NORMAL`
- `busy_timeout=10s`
- `foreign_keys=ON`
- 连接池上限 8

schema SQL 在 `backend/migrations/<module>/`，由 `sqlx::migrate!` 编译进二进制。查询使用运行时校验的 `sqlx::query` / `query_as`，不使用 `query!` 宏。

> `sqlx::migrate!` 在编译期嵌入迁移，新增迁移文件默认不会让 cargo 失效旧产物（增量构建会静默漏掉新迁移）。`backend/crates/db/build.rs` 用 `cargo:rerun-if-changed=migrations` 跟踪整个目录，迁移一变即重建 `haruhi-db`。新增迁移后无需 `cargo clean`。

画廊访客统计的明细保存在 `art_visitors`，聚合展示值缓存在 `art_visitor_stats`，由迁移初始化并由 `/api/art/visitors` 增量更新，避免首页每次访问都扫描访客明细表。画廊作品的随机排序使用 `artworks.random_key` 作为持久随机键；请求里的 `seed` 只选择环形分页起点，避免每次随机列表都对全量作品做表达式排序。

art 公会委托保存在 `guild_quests` / `guild_quest_claims` / `guild_quest_events`。委托时间限制互斥：`deadline_days` 表示接取后按天数截止，`fixed_deadline_at` 表示固定北京时间截止，二者不能同时生效；未设置单独截止时，循环委托按 `cycle_days` 和 `cycle_reset_hour` 以北京时间刷新，默认刷新点为 4 点。`repeat_on_complete` 只用于“按天数截止”的委托：开启后不再使用 `cycle_days`，本轮完成并结算后立即恢复为可再次接取；自动接取委托会在结算后立即生成下一轮 active claim。相关 API 会返回 `deadlineDays`、`fixedDeadlineAt`、`cycleDays`、`cycleResetHour`、`repeatOnComplete` 和本轮 `deadlineAt`，前端只展示当前生效的时间模式。

公会委托迁移是向后兼容的增量列和 claim/event 表重建：旧 `deadline_hours` 会回填为 `deadline_days`，旧日常委托回填为北京时间 4 点刷新。线上更新顺序是先部署包含迁移的新后端，再发布前端；回滚前端不破坏数据，回滚后端前需确认新列未被旧二进制依赖或恢复数据库备份。

## 鉴权与 RBAC

登录流程：

1. `POST /api/auth/login` 校验用户名和 argon2 密码。
2. 后端签发 JWT，payload 包含 `sub`、`super`、`iat`、`exp`。
3. 前端把 token 放进 `Authorization: Bearer <token>`。
4. handler 用 `AuthUser` 提取身份。
5. 后台接口调用 `authorize()` 查 `core.db` 判权。

角色等级：

| 角色        | 等级 |
| ----------- | ---- |
| `viewer`    | 1    |
| `editor`    | 2    |
| `moderator` | 3    |
| `admin`     | 4    |

动作等级：

| Action     | 最低等级 |
| ---------- | -------- |
| `Read`     | 1        |
| `Write`    | 2        |
| `Moderate` | 3        |
| `Manage`   | 4        |

`authorize(core, user, app, action)` 的规则：

- 超管直接通过。
- 普通用户必须是 `active`。
- 查用户在 `app` 及其父作用域上的最高角色等级。
- 最高等级达到 action 要求才通过。

作用域用点号分层：

```text
scope_chain("news.activity") -> ["news.activity", "news"]
scope_chain("news")          -> ["news"]
```

所以拥有 `news` 角色的人也拥有 `news.*` 子作用域权限。反过来不成立。`news` 当前拆了 `news.blog`、`news.activity`、`news.store`、`news.points` 四个子作用域，其它模块使用单层作用域。

`/api/admin/*` 是超管接口，使用 `require_super()`。`backend/crates/server/src/admin_routes.rs` 的 `APPS` 常量决定 `/console/` 可分配的 app/子作用域列表。

## 共享前端包

`@haruhi/api-client` 提供：

- `createApiClient(base)`：封装 `get/post/put/patch/del/postForm`，自动带 JWT。
- `createAuth(apiBase)`：登录、`me()`、登出。
- `createAdminAuth(app)`：后台登录、恢复会话、检查模块权限。
- `hasScope(user, scope)`：前端侧作用域判断，规则与后端父级继承一致。
- `resolveUploadUrl(path, base)`：把上传相对路径拼成可访问 URL。

前端权限判断只负责界面显示。权限边界在后端 handler。

## 稳定性与运行约定

| 关注点       | 实现                                                                                                                                                                                                                                                                                     |
| ------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| readiness    | `/api/health/ready` 查询 `core.db`                                                                                                                                                                                                                                                       |
| 限流         | 单进程内存滑动窗口（`ratelimit::RateLimiter`）：登录单 IP 10 分钟 10 次；匿名上传 60 次                                                                                                                                                                                                  |
| 上传校验     | art/exam 上传过 `media::check_image`/`check_media` 类型大小白名单，详见 [SECURITY.md](../SECURITY.md)。画廊图片 multipart 字段边读边写入临时文件并累计大小，成功后原子改名，避免多图上传时把展示图和原图同时保存在后端内存。                                                                  |
| body 上限    | axum `DefaultBodyLimit` 设为 256 MiB，Nginx 同步设置                                                                                                                                                                                                                                     |
| 错误响应     | 4xx 文案给用户；所有 5xx 统一返回「服务器内部错误」，详情只入日志（`core::error`）                                                                                                                                                                                                       |
| CORS         | 显式 `HARUHI_CORS_ORIGINS` 时一律按白名单（debug 也锁紧）；未设时 debug 放开、release 收敛到 `PUBLIC_SITE_URL`                                                                                                                                                                           |
| 不安全默认值 | JWT/ART_COOKIE 密钥、`admin/admin123` 超管仅「debug + 绑定回环」启用；否则要求显式配置                                                                                                                                                                                                   |
| 邮件         | `haruhi-mail` 只发信；shop 模块维护 `email_jobs` 队列和重试                                                                                                                                                                                                                              |
| AI 审核      | 画廊上传先持久化为 `pending`；配置 `DASHSCOPE_API_KEY` 时后台异步审核，通过后自动发布并发放积分，违规转入 `flagged`，AI 异常保持 `pending` 供人工复核。其它模块仍按各自 handler 逻辑处理 AI 结果。                                                                                           |
| 图片缩略图   | art 画廊：nginx 静态直出 `uploads/art/.thumbs/<w>/`（宽度白名单 320/640/960），未命中回源 `GET /api/art/thumb`，后端用 **libvips 子进程**（流式、内存有界）生成并落盘。应用内信号量限并发=2；上传时预热、删除作品时清理；存量图用 `deploy/backfill-thumbs.sh` 预热。缓存可整目录删除重建 |

## 新模块入口

新增模块请按 [ADDING_MODULE.md](ADDING_MODULE.md) 操作。最重要的接线点：

- `backend/crates/server/src/modules/<module>.rs`
- `backend/crates/server/src/modules/mod.rs`
- `backend/crates/db/src/lib.rs`
- `backend/migrations/<module>/`
- `backend/crates/server/src/main.rs` 的关停连接池列表
- `backend/crates/server/src/admin_routes.rs` 的 `APPS`
- `apps/<module>/`
- `deploy/nginx.conf`
