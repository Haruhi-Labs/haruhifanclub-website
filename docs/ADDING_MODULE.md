# 新增一个业务模块

> 以 `novel` 为端到端模板，加一个新模块 `<m>`（下文用 `widget` 举例占位，替换成你的模块名）。
> 模块名要在后端路由、数据库、迁移、RBAC、前端 `base`、Nginx 之间保持一致。
> 参考实现：后端 `backend/crates/server/src/modules/novel.rs`、前端 `apps/novel/`。

## 总览（要改/新增的文件）

```
后端
  backend/crates/server/src/modules/widget.rs        新增  router + handler + authorize
  backend/crates/server/src/modules/mod.rs           改    mod widget; + nest("/widget", ...)
  backend/migrations/widget/0001_init.sql            新增  建表
  backend/crates/db/src/lib.rs                        改    Pools 加字段 + connect + migrate
前端
  apps/widget/package.json                            新增  workspace 包 + @haruhi/api-client
  apps/widget/vite.config.ts                          新增  base + dev 代理
  apps/widget/src/...                                 新增  接入 createApiClient/createAdminAuth
RBAC / 部署
  backend/crates/server/src/admin_routes.rs          改    APPS 加 "widget"（及子作用域）
  deploy/nginx.conf  (+ test.haruyuki.cn.nginx.conf)  改    /widget/ location
  deploy/migrate-data.sh / migrate-live.py            改    （若需迁旧数据）
```

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
- 后台 handler 第一行 `authorize(&state.pools.core, &user, "widget", Action::X)`；
  动作分级：读=`Read`、日常写=`Write`、审核=`Moderate`、删除/高危=`Manage`。
- 仅超管的端点用 `require_super(&user)?`。
- “可选登录”端点用提取器 `Option<AuthUser>`（游客可读、登录可见更多）。
- 用运行时校验查询 `sqlx::query`/`query_as`（不要 `query!` 宏）；上传走 `haruhi_media::*`。

## 2. 迁移：`migrations/widget/0001_init.sql`

```sql
-- backend/migrations/widget/0001_init.sql
CREATE TABLE IF NOT EXISTS items (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL
);
```

> 用 `IF NOT EXISTS`，与 novel/core 迁移一致，保证幂等。

## 3. 接线：`mod.rs` + `db/lib.rs::Pools`

### 3a. 路由挂载 —— `backend/crates/server/src/modules/mod.rs`

```rust
pub mod widget;                       // 加这行

pub fn mount(api: Router<AppState>) -> Router<AppState> {
    api.nest("/novel", novel::router())
        // ...
        .nest("/widget", widget::router())   // 加这行 → /api/widget/*
}
```

### 3b. 连接池 + 迁移 —— `backend/crates/db/src/lib.rs`

模块需要**独立库**时（绝大多数情况），三处都要加 `widget`：

```rust
pub struct Pools {
    pub core: SqlitePool,
    // ...
    pub widget: SqlitePool,                          // 1) 加字段
}

// connect()
let pools = Pools {
    core: open_pool(&cfg.db_path("core")).await?,
    // ...
    widget: open_pool(&cfg.db_path("widget")).await?,   // 2) 打开 data/widget.db
};

// migrate()
sqlx::migrate!("../../migrations/widget").run(&self.widget).await?;   // 3) 跑迁移
```

> 若新模块不需要独立库（如只读 core），可不加 `Pools` 字段、直接用 `state.pools.core`——
> 但默认每模块一库（写锁隔离、迁移零转换，见 ARCHITECTURE.md §4）。
> 别忘了 `main.rs` 优雅关闭里遍历连接池刷 WAL 的数组——加库后把 `&pools.widget` 也补进去。

## 4. 前端 app：`apps/widget/`

最小骨架（参考 `apps/novel/`、`apps/exam/`）：

### 4a. `apps/widget/package.json`

```jsonc
{
  "name": "@haruhi/widget",
  "private": true,
  "type": "module",
  "scripts": { "dev": "vite", "build": "vite build" },
  "dependencies": {
    "@haruhi/api-client": "workspace:*",   // 共享客户端，workspace 协议
    "vue": "^3.5.22",
    "vue-router": "^4.6.3"
  },
  "devDependencies": { "@vitejs/plugin-vue": "^6.0.1", "vite": "^7.1.11" }
}
```

> `apps/*` 已在 `pnpm-workspace.yaml` 通配，新建目录后跑 `pnpm install` 即纳入 workspace。

### 4b. `apps/widget/vite.config.ts`

```ts
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
export default defineConfig({
  base: '/widget/',                       // 子路径，要和 Nginx location 对齐
  plugins: [vue()],
  server: {
    port: 5210,                            // 选一个未占用端口（已用 5200/5202/5203…）
    proxy: {
      '/api':     { target: 'http://127.0.0.1:17777', changeOrigin: true },
      '/uploads': { target: 'http://127.0.0.1:17777', changeOrigin: true },
    },
  },
})
```

### 4c. 接入 `@haruhi/api-client`

```ts
import { createApiClient, createAdminAuth } from '@haruhi/api-client'

const api = createApiClient('/api/widget')               // 自动带 Bearer token
const items = await api.get('/items')

const adminAuth = createAdminAuth('widget')              // app 名 = 后端作用域
await adminAuth.login(username, password)                // 登录拿 token
if (adminAuth.hasPerm(await adminAuth.me())) { /* 显示后台 */ }
// 细粒度判断：hasScope(user, 'widget')（与后端父级继承一致）
```

`createApiClient(base)` 提供 `get/post/put/patch/del/postForm`；`createAdminAuth(app)` 管登录、
`me()`、token、按 app 判权。类型见 `packages/api-client/index.d.ts`。

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
> 的角色覆盖所有 `widget.*` 子作用域（父级继承，见 ARCHITECTURE.md §5）。无需改 `auth` crate。

## 6. Nginx + 数据迁移

### 6a. Nginx —— `deploy/nginx.conf`（及 `deploy/test.haruyuki.cn.nginx.conf`）

在前端 app 段加一条 location（base 与 vite `base` 一致）：

```nginx
location /widget/ { alias /var/www/haruhifanclub/apps/widget/dist/; try_files $uri $uri/ /widget/index.html; }
```

`/api/` 与 `/uploads/` 已通配，无需改。`nginx -t && systemctl reload nginx`。

### 6b. 数据迁移（仅当有旧数据要迁）

在 `deploy/migrate-data.sh` 加 `migrate_widget()`（拷库 → `rsync uploads/widget/` → `sqlite3` 重写旧 URL 前缀
为 `/uploads/widget/`），并把 `widget` 加进 `MODULES` 默认列表与 `case` 分支；服务器抓 live 数据则在
`deploy/migrate-live.py` 加对应函数。无旧数据可跳过——首启会按迁移建空库。

---

## 验收清单

- [ ] `cargo build -p haruhi-server` 通过；启动日志有“数据库迁移完成”，`data/widget.db` 已生成
- [ ] `curl /api/widget/items` 返回；后台接口未带 token → 401，带越权 token → 403
- [ ] `pnpm --filter @haruhi/widget dev` 起得来，`/widget/` 能访问、`/api` 代理通
- [ ] `/console/` 里能给用户分配 `widget` 角色，分配后该用户后台权限即时生效
- [ ] `pnpm -r --filter "./apps/*" build` 出 `apps/widget/dist`；Nginx `/widget/` 可访问
- [ ] `cargo fmt --check` + `cargo clippy -- -D warnings` 通过（CI 门禁，见 `.github/workflows/ci.yml`）
```
