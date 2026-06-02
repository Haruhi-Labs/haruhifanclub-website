# haruhifanclub

凉宫春日应援团统一站点 monorepo。把原先分散的 5 个子站（新闻 / 画廊 / 考试 / 书库 / 商城）
整合为**一个 Rust 后端 + 多个独立前端 app**，统一鉴权与工程化，全部挂在 `haruyuki.cn` 子路径下。

## 架构总览

```
前端（pnpm workspace）            后端（Cargo workspace, 单一 axum 进程 :17777）
apps/news    → /news/             backend/crates/server   装配路由 /api/<module>/*
apps/art     → /art/              backend/crates/core     配置 / 错误 / 响应
apps/exam    → /exam/             backend/crates/db       sqlite 连接池(WAL) + 迁移
apps/novel   → /library/          backend/crates/auth     JWT + argon2 + RBAC
apps/shop    → /shop/             backend/crates/media    上传 / WebP / EPUB
apps/console → /console/  (RBAC)   backend/crates/ai       DashScope 内容审核
packages/api-client  统一 fetch    backend/crates/mail     SMTP/Resend 邮件队列
                                   data/<module>.db + data/core.db    uploads/<module>/
```

- **统一 API 前缀**：`/api/<module>/*`（消灭旧的 `/blog-api` `/art-api` `/exam/api` `/shop-api` 多套命名）。
- **统一鉴权**：`/api/auth/login` 拿 JWT；后台按 RBAC 鉴权，超管在 `/console/` 按应用给管理员分配角色（viewer/editor/moderator/admin）。
- **每模块独立 SQLite**：写锁隔离，迁移零转换；`core.db` 存用户/角色。
- **静态上传**：`/uploads/<module>/...`，生产由 Nginx 直接 alias。

## 本地开发

```bash
# 依赖：Node 20 (nvm)、pnpm 10、Rust stable、ffmpeg、sqlite3
nvm use && pnpm install
cargo build

# 1) 准备环境变量（.env 写入受限，可直接 export 或复制模板）
cp deploy/env.sample .env   # 填写 HARUHI_JWT_SECRET / 超管账号 等

# 2) 迁移历史数据（可选，把旧站 sqlite + uploads 拷进来）
bash deploy/migrate-data.sh            # 全部模块
bash deploy/migrate-data.sh novel art  # 指定模块

# 3) 起后端
cargo run -p haruhi-server             # 监听 127.0.0.1:17777

# 4) 起某个前端（dev 代理 /api 与 /uploads 到 17777）
pnpm --filter @haruhi/novel dev        # http://localhost:5203/library/
pnpm --filter @haruhi/console dev      # http://localhost:5200/console/
```

## 构建与部署

```bash
pnpm -r --filter "./apps/*" build      # 各前端 → apps/*/dist
cargo build --release -p haruhi-server # 后端单二进制 → target/release/haruhi-server
```

部署：单二进制由 systemd 守护（`deploy/haruhifanclub.service`），Nginx 反代 `/api`+`/uploads`、
静态托管各 `apps/*/dist`（`deploy/nginx.conf`）。`data/` 与 `uploads/` 独立持久化并备份，永不进 git。

## 工程约定

- 后端 sqlx 用**运行时校验**查询（非 `query!` 宏，免 DATABASE_URL）。
- 新增模块：`backend/crates/server/src/modules/<m>.rs`（`router()`）+ `migrations/<m>/` +
  在 `modules/mod.rs` nest + 在 `db/lib.rs::migrate()` 加一行 + `apps/<m>` 前端接入 `@haruhi/api-client`。
- 后台接口用 `haruhi_auth::authorize(&pools.core, &user, "<module>", Action::X)`；超管用 `require_super`。
- 详见 `docs/` 与计划文件。novel 模块是端到端复刻模板。
