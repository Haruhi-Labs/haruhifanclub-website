# 新增一个业务模块

> 以 `novel` 为端到端模板，加一个新模块 `<m>`（下文用 `widget` 举例占位，替换成你的模块名）。
> 模块名要在后端路由、数据库、迁移、RBAC、前端 `base`、Nginx、提交 scope 之间保持一致。
> 参考实现：后端 `backend/crates/server/src/modules/novel.rs`、前端 `apps/novel/`。

## 总览（要改/新增的文件）

```
后端
  backend/crates/server/src/modules/widget.rs        新增  router + handler + authorize
  backend/crates/server/src/modules/mod.rs           改    pub mod widget; + nest("/widget", ...)
  backend/migrations/widget/0001_init.sql            新增  建表
  backend/crates/db/src/lib.rs                        改    Pools 加字段 + connect + migrate
  backend/crates/server/src/main.rs                   改    优雅关停的 WAL checkpoint 数组加 &pools.widget
前端
  apps/widget/package.json                            新增  workspace 包 + @haruhi/api-client
  apps/widget/vite.config.(js|ts)                     新增  base + dev 代理（端口）
  apps/widget/src/...                                 新增  接入 createApiClient/createAdminAuth
RBAC / 部署 / 协作
  backend/crates/server/src/admin_routes.rs          改    APPS 加 "widget"（及子作用域）
  deploy/nginx.conf  (+ deploy/test.haruyuki.cn.nginx.conf)  改  /widget/ location
  deploy/migrate-data.sh / migrate-live.py            改    （若需迁旧数据）
```

> 工具链：所有前端 app 已对齐**单仓库统一版本**——Vite 7（`^7.1.11`）、`@vitejs/plugin-vue ^6.0.1`、
> Vue 3.5、Pinia 3（用到状态管理时）。新模块照抄即可，不要引入别的大版本。

---

## 1. 后端模块：`modules/widget.rs`

照抄 `modules/novel.rs` 结构：`router()` + 公开/后台 handler + `authorize`。

```rust
// backend/crates/server/src/modules/widget.rs
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use haruhi_auth::{authorize, Action, AuthUser};
use haruhi_core::{AppError, AppResult};
use serde_json::{json, Value};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/items", get(list_items))            // 公开
        .route("/items/{id}", get(get_item))         // axum 0.8 路径参数用 {id}（见 novel.rs）
        .route("/admin/items", post(create_item))     // 后台：需 widget 写权限
}

async fn list_items(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let rows: Vec<(i64, String)> = sqlx::query_as("SELECT id, name FROM items ORDER BY id")
        .fetch_all(&state.pools.widget)               // 用专属连接池（见第 3 步）
        .await?;
    Ok(Json(json!(rows.into_iter()
        .map(|(id, name)| json!({"id": id, "name": name})).collect::<Vec<_>>())))
}

async fn create_item(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "widget", Action::Write).await?;  // RBAC
    // ... INSERT INTO items ...
    Ok(Json(json!({ "ok": true })))
}
```

约定（与现有模块一致）：
- 公开接口 `GET /xxx`，后台接口前缀 `/admin/xxx`。
- 路径参数用 axum 0.8 的 `{id}` 写法（`backend/crates/server` 用 `axum = "0.8"`），不是旧的 `:id`。
- 后台 handler 第一行 `authorize(&state.pools.core, &user, "widget", Action::X)`；
  动作分级：读=`Read`、日常写=`Write`、审核=`Moderate`、删除/高危=`Manage`。
- 仅超管的端点用 `require_super(&user)?`。
- “可选登录”端点用提取器 `Option<AuthUser>`（游客可读、登录可见更多）。
- 用运行时校验查询 `sqlx::query`/`query_as`（不要 `query!` 宏，免 `DATABASE_URL`）；上传走 `haruhi_media::*`。
- **数值/文本解析别自己造轮子**：用 `haruhi_core::parse`（`num_i64` / `clamp_int` / `clamp_len` /
  `safe_text` / `parse_int_or` 等），各模块已统一抽到这里。
- 路径/上传目录用 `Config` 的助手：`state.cfg.db_path("widget")`（库路径，一般 db crate 用）、
  `state.cfg.uploads_subdir("widget")`（`uploads/widget/`）、`state.cfg.uploads_dir`（uploads 根，删文件时拼相对路径用）。

## 2. 迁移：`migrations/widget/0001_init.sql`

```sql
-- backend/migrations/widget/0001_init.sql
CREATE TABLE IF NOT EXISTS items (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL
);
```

> 用 `IF NOT EXISTS`，与 novel/core 迁移一致，保证幂等。迁移目录在仓库根的
> `backend/migrations/<module>/`，`sqlx::migrate!` 在 `db` crate 里以相对路径 `../../migrations/<module>` 引用。

## 3. 接线：`mod.rs` + `db/lib.rs::Pools`

### 3a. 路由挂载 —— `backend/crates/server/src/modules/mod.rs`

```rust
pub mod widget;                       // 加这行（与 art/exam/news/novel/shop 并列）

pub fn mount(api: Router<AppState>) -> Router<AppState> {
    api.nest("/novel", novel::router())
        // ...已有 art / news / exam / shop...
        .nest("/widget", widget::router())   // 加这行 → /api/widget/*
}
```

> `mount()` 由 `routes.rs` 统一挂在 `/api` 下，`/uploads` 静态由 `ServeDir` 提供，二者无需在模块里再配。

### 3b. 连接池 + 迁移 —— `backend/crates/db/src/lib.rs`

模块需要**独立库**时（绝大多数情况），三处都要加 `widget`：

```rust
pub struct Pools {
    pub core: SqlitePool,
    pub news: SqlitePool,
    pub art: SqlitePool,
    pub exam: SqlitePool,
    pub novel: SqlitePool,
    pub shop: SqlitePool,
    pub widget: SqlitePool,                          // 1) 加字段
}

// Pools::connect()
let pools = Pools {
    core: open_pool(&cfg.db_path("core")).await?,
    // ...已有各库...
    widget: open_pool(&cfg.db_path("widget")).await?,   // 2) 打开 data/widget.db
};

// Pools::migrate()
sqlx::migrate!("../../migrations/widget").run(&self.widget).await?;   // 3) 跑迁移
```

> 若新模块不需要独立库（如只读 core），可不加 `Pools` 字段、直接用 `state.pools.core`——
> 但默认每模块一库（写锁隔离、迁移零转换，见 `ARCHITECTURE.md` §4）。

### 3c. 优雅关停刷 WAL —— `backend/crates/server/src/main.rs`

加了 `Pools` 字段后，把新池补进关停时 `wal_checkpoint(TRUNCATE)` 的数组（否则退出时该库 WAL 不强制 checkpoint）：

```rust
for pool in [
    &pools.core, &pools.news, &pools.art, &pools.exam, &pools.novel, &pools.shop,
    &pools.widget,                 // 加这行
] {
    let _ = sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)").execute(pool).await;
    pool.close().await;
}
```

## 4. 前端 app：`apps/widget/`

最小骨架（纯 JS 参考 `apps/novel/`；要 TS 参考 `apps/exam/` 或 `apps/console/`）：

### 4a. `apps/widget/package.json`

```jsonc
{
  "name": "@haruhi/widget",
  "version": "1.0.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",          // 若用 TS：改成 "vue-tsc --noEmit && vite build"（exam/console 即此）
    "preview": "vite preview"
  },
  "dependencies": {
    "@haruhi/api-client": "workspace:*",   // 共享客户端，workspace 协议
    "vue": "^3.5.22",
    "vue-router": "^4.6.3"
    // 需要状态管理再加 "pinia": "^3.0.4"（全仓 Pinia 3 单版本）
  },
  "devDependencies": { "@vitejs/plugin-vue": "^6.0.1", "vite": "^7.1.11" }
}
```

> `apps/*` 已在 `pnpm-workspace.yaml` 通配（`packages: ["apps/*", "packages/*"]`），
> 新建目录后跑 `pnpm install` 即纳入 workspace。
> 现有 app 端口已占用：console 5200、art 5201、exam 5202、novel 5203、news 5204、shop 5205——新模块另选（如 5210）。

### 4b. `apps/widget/vite.config.(js|ts)`

```ts
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
export default defineConfig({
  base: '/widget/',                       // 子路径，要和 Nginx location 对齐
  plugins: [vue()],
  server: {
    port: 5210,                            // 选一个未占用端口（5200–5205 已用）
    proxy: {
      '/api':     { target: 'http://127.0.0.1:17777', changeOrigin: true },
      '/uploads': { target: 'http://127.0.0.1:17777', changeOrigin: true },
    },
  },
})
```

### 4c. 接入 `@haruhi/api-client`

```ts
import { createApiClient, createAdminAuth, resolveUploadUrl } from '@haruhi/api-client'

const api = createApiClient('/api/widget')               // 自动带 Bearer token
const items = await api.get('/items')

// 渲染后端返回的上传相对路径（如 "widget/foo.webp"）时，用共享拼接，别手写前缀
const url = resolveUploadUrl(item.image_path)            // → /uploads/widget/foo.webp

const adminAuth = createAdminAuth('widget')              // app 名 = 后端作用域
await adminAuth.restore()                                // 应用挂载时尝试用已存 token 恢复会话
const res = await adminAuth.login(username, password)    // 登录拿 token（返回 {ok, user?, error?}）
if (res.ok && adminAuth.hasPerm(res.user)) { /* 显示后台 */ }
// 细粒度判断：hasScope(user, 'widget')（与后端父级继承一致）
```

`createApiClient(base)` 提供 `get/post/put/patch/del/postForm`；
`createAdminAuth(app)` 管登录、`me()`、`restore()`、token、按 app 判权（`hasPerm` / `hasValidToken` / `buildHeaders`）；
`resolveUploadUrl(path, base?)` 统一拼接 `/uploads/...`（art/novel 已去重到这里，新模块沿用）。
全部签名与类型见 `packages/api-client/index.d.ts`。

> 说明：`packages/ui`、`packages/config` 目前为空 stub、未启用，不要依赖它们。

## 5. console RBAC：`admin_routes.rs` 的 `APPS`

让超管能在 `/console/` 给该模块分配角色——把作用域加进 `APPS` 常量：

```rust
// backend/crates/server/src/admin_routes.rs
const APPS: &[&str] = &[
    "news", "news.blog", "news.activity", "news.store", "news.points",
    "art", "exam", "novel", "shop",
    "widget",            // 加这行
    "console",
];
```

> 若模块要分**子作用域**（像 news 那样 `news.blog` / `news.activity`），就加 `"widget.foo"` 等，
> 并在 handler 里 `authorize(..., "widget.foo", ...)`。`scope_chain` 会自动让父作用域 `widget`
> 的角色覆盖所有 `widget.*` 子作用域（父级继承，见 `ARCHITECTURE.md` §5）。无需改 `auth` crate。

## 6. 提交 scope、Nginx + 数据迁移

### 6a. 提交 scope —— 无需改配置

scope 是**自由的、不在代码里维护集合**，所以新增模块**不用动 `commitlint.config.mjs`**——
直接用模块名当 scope 即可（如 `feat(widget): 新增挂件模块`）。提交规范见 `CONTRIBUTING.md`。

### 6b. Nginx —— `deploy/nginx.conf`（及 `deploy/test.haruyuki.cn.nginx.conf`）

在前端 app 段加一条 location（base 与 vite `base` 一致）：

```nginx
location /widget/ { alias /var/www/haruhifanclub/apps/widget/dist/; try_files $uri $uri/ /widget/index.html; }
```

`/api/` 与 `/uploads/` 已通配，无需改。`nginx -t && systemctl reload nginx`。

### 6c. 数据迁移（仅当有旧数据要迁）

在 `deploy/migrate-data.sh` 加 `migrate_widget()`（拷库 → `rsync uploads/widget/` → `sqlite3` 重写旧 URL 前缀
为 `/uploads/widget/`），并把 `widget` 加进 `MODULES` 默认列表（当前为 `novel art news exam shop`）与 `case` 分支；
服务器抓 live 数据则在 `deploy/migrate-live.py` 加对应函数。无旧数据可跳过——首启会按迁移建空库。

---

## 验收清单

- [ ] `cargo build -p haruhi-server` 通过；启动日志有“数据库迁移完成”，`data/widget.db` 已生成
- [ ] `curl /api/widget/items` 返回；后台接口未带 token → 401，带越权 token → 403
- [ ] `pnpm --filter @haruhi/widget dev` 起得来，`/widget/` 能访问、`/api` 代理通
- [ ] `/console/` 里能给用户分配 `widget` 角色，分配后该用户后台权限即时生效
- [ ] `pnpm -r --filter "./apps/*" build` 出 `apps/widget/dist`；Nginx `/widget/` 可访问
- [ ] `cargo fmt --all --check` + `cargo clippy --workspace --all-targets -- -D warnings` 通过
- [ ] `pnpm lint:js` 通过；commit 用模块名作 scope（`feat(widget): ...`）
- [ ] 以上即 CI（`.github/workflows/ci.yml`）的 frontend / backend 两个 job，最终由聚合 gate `ci-ok` 汇总

---

更多：根 `README.md`、`CONTRIBUTING.md`、`docs/ARCHITECTURE.md`（数据隔离 §4 / RBAC §5）、`docs/DEPLOYMENT.md`。
