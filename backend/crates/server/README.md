# haruhi-server

后端二进制。基于 axum，负责装配 API、上传静态目录、数据库 schema 更新、超管 seed、登录限流、shop 邮件 worker 和关停刷 WAL。

## 本地运行

从仓库根执行：

```bash
bash deploy/gen-secrets.sh
pnpm dev:backend
```

或直接：

```bash
cargo run -p haruhi-server
```

本地 Vite 代理要求后端监听 `127.0.0.1:17777`，该值来自 `.env` 的 `HARUHI_BIND`。如果没有 `.env`，`Config` 代码默认是 `127.0.0.1:8080`。

健康检查：

```bash
curl -s http://127.0.0.1:17777/api/health
curl -s http://127.0.0.1:17777/api/health/ready
```

## 目录

```text
src/
  main.rs          进程入口
  lib.rs           测试和二进制共用的库目标
  routes.rs        /api、/uploads、CORS、TraceLayer、body 上限
  state.rs         AppState 与 AuthSecret
  ratelimit.rs     限流（RateLimiter：登录 + 匿名上传）和客户端 IP 解析
  pagination.rs    列表端点公共分页助手 total_pages / page_meta
  auth_routes.rs   /api/auth/login、/api/auth/me
  admin_routes.rs  /api/admin/* 超管控制台接口
  seed.rs          首次启动 seed 超管
  notify.rs
  modules/
    mod.rs
    news.rs
    art.rs
    exam.rs
    novel.rs
    shop/
```

## 启动流程

1. 加载 `.env`。
2. 读取 `Config`。
3. 打开 SQLite 连接池。
4. 执行 schema 更新。
5. 在 `core.db` 无用户时 seed 超管。
6. 启动 shop 邮件 worker。
7. 装配路由。
8. 监听 `cfg.bind`。
9. 收到 `SIGTERM` 或 Ctrl-C 后等待请求结束，刷 WAL，关闭连接池。

## 路由

| 路径                | 说明                                    |
| ------------------- | --------------------------------------- |
| `/api/health`       | 存活探针                                |
| `/api/health/ready` | 查询 `core.db`，判断是否可服务          |
| `/api/auth/*`       | 登录、当前用户                          |
| `/api/admin/*`      | 超管用户和角色管理                      |
| `/api/news/*`       | news 模块                               |
| `/api/art/*`        | art 模块                                |
| `/api/exam/*`       | exam 模块                               |
| `/api/novel/*`      | novel 模块                              |
| `/api/shop/*`       | shop 模块                               |
| `/uploads/*`        | 本地静态上传目录；生产由 Nginx 直接服务 |

请求体上限为 256 MiB，用于 EPUB、图片、音频上传。

## 关键行为

- CORS：显式设 `HARUHI_CORS_ORIGINS` 时一律按白名单（debug 也锁紧）；未设时 debug 放开、release 收敛到 `PUBLIC_SITE_URL`。
- 限流：单进程内存滑动窗口（`RateLimiter`）。登录单 IP 10 分钟 10 次；art/exam 匿名上传 60 次，超限 429。
- 鉴权：`AuthUser` 解析 JWT；后台接口使用 `authorize()`；超管接口使用 `require_super()`。
- 错误：4xx 文案给用户；5xx 一律返回「服务器内部错误」，详情只入日志，不外泄内部细节。
- 上传：art/exam 匿名上传过 `media::check_image`/`check_media` 类型大小白名单。
- 分页：列表端点用 `pagination::page_meta` 生成统一 `{page,pageSize,total,totalPages}`（exam 特殊分页除外）。
- 数据库：每模块一个 SQLite 连接池，WAL 模式。
- SQL：使用运行时校验的 `sqlx::query` / `query_as`；动态拼接列名过标识符白名单。
- 邮件：shop 模块维护队列，`haruhi-mail` 只发送单封邮件。
- AI：art/exam 调 `haruhi-ai`，未配置 key 时放行。

## 开发

```bash
cargo build -p haruhi-server
cargo test -p haruhi-server
RUST_LOG=debug cargo run -p haruhi-server
```

改模块路由时，至少检查：

- 对应模块的公开接口。
- 后台接口的 `authorize()` 作用域和 action。
- 数据库 schema 是否已登记到 `haruhi-db`。
- 新连接池是否加入 `main.rs` 关停刷 WAL 列表。
