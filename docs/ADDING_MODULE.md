# 新增业务模块

本文用 `<m>` 表示新模块名，例如 `widget`。模块名应同时用于 API 前缀、数据库名、RBAC 作用域、前端包名、Vite `base`、Nginx 子路径和提交 scope。

推荐参考：

- 后端：`backend/crates/server/src/modules/novel.rs`
- 前端：`apps/novel/`

## 需要改的文件

```text
backend/crates/server/src/modules/<m>.rs
backend/crates/server/src/modules/mod.rs
backend/crates/db/src/lib.rs
backend/crates/server/src/main.rs
backend/crates/server/src/admin_routes.rs
backend/migrations/<m>/0001_init.sql

apps/<m>/package.json
apps/<m>/vite.config.(js|ts)
apps/<m>/src/...

deploy/nginx.conf
deploy/test.haruyuki.cn.nginx.conf
```

## 1. 后端模块

新增 `backend/crates/server/src/modules/<m>.rs`：

```rust
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use haruhi_auth::{authorize, Action, AuthUser};
use haruhi_core::AppResult;
use serde_json::{json, Value};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/items", get(list_items))
        .route("/items/{id}", get(get_item))
        .route("/admin/items", post(create_item))
}

async fn list_items(State(state): State<AppState>) -> AppResult<Json<Value>> {
    let rows: Vec<(i64, String)> = sqlx::query_as("SELECT id, name FROM items ORDER BY id")
        .fetch_all(&state.pools.widget)
        .await?;

    Ok(Json(json!({
        "items": rows.into_iter().map(|(id, name)| json!({ "id": id, "name": name })).collect::<Vec<_>>()
    })))
}

async fn get_item(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<Value>> {
    let row: (i64, String) = sqlx::query_as("SELECT id, name FROM items WHERE id = ?")
        .bind(id)
        .fetch_one(&state.pools.widget)
        .await?;
    Ok(Json(json!({ "id": row.0, "name": row.1 })))
}

async fn create_item(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<Value>,
) -> AppResult<Json<Value>> {
    authorize(&state.pools.core, &user, "widget", Action::Write).await?;
    let name = body.get("name").and_then(|v| v.as_str()).unwrap_or("").trim();
    let id: i64 = sqlx::query_scalar("INSERT INTO items (name) VALUES (?) RETURNING id")
        .bind(name)
        .fetch_one(&state.pools.widget)
        .await?;
    Ok(Json(json!({ "id": id })))
}
```

约定：

- 公开接口放在模块根下。
- 后台接口放在 `/admin/*` 下。
- axum 0.8 路径参数写 `{id}`。
- 后台 handler 先调用 `authorize()`；只允许超管的接口用 `require_super()`。
- SQL 用 `sqlx::query` / `query_as` / `query_scalar`，不要使用 `query!` 宏。
- 文本和数值解析优先用 `haruhi_core::parse`。
- 上传和媒体处理优先用 `haruhi_media`。

## 2. 挂载路由

修改 `backend/crates/server/src/modules/mod.rs`：

```rust
pub mod widget;

pub fn mount(api: Router<AppState>) -> Router<AppState> {
    api
        .nest("/novel", novel::router())
        .nest("/art", art::router())
        .nest("/news", news::router())
        .nest("/exam", exam::router())
        .nest("/shop", shop::router())
        .nest("/widget", widget::router())
}
```

挂载后模块 API 为 `/api/widget/*`。

## 3. 数据库 schema

新增 `backend/migrations/<m>/0001_init.sql`：

```sql
CREATE TABLE IF NOT EXISTS items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL
);
```

SQL 应幂等，优先使用 `IF NOT EXISTS`。

## 4. 连接池与 schema 登记

修改 `backend/crates/db/src/lib.rs`。

`Pools` 加字段：

```rust
pub widget: SqlitePool,
```

`Pools::connect()` 打开库：

```rust
widget: open_pool(&cfg.db_path("widget")).await?,
```

`Pools::migrate()` 执行 schema 更新：

```rust
sqlx::migrate!("../../migrations/widget")
    .run(&self.widget)
    .await?;
```

如果模块不需要独立库，可复用 `state.pools.core`，但现有业务模块默认都是一模块一库。

## 5. 关停刷 WAL

修改 `backend/crates/server/src/main.rs` 的连接池数组：

```rust
for pool in [
    &pools.core,
    &pools.news,
    &pools.art,
    &pools.exam,
    &pools.novel,
    &pools.shop,
    &pools.widget,
] {
    let _ = sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
        .execute(pool)
        .await;
    pool.close().await;
}
```

## 6. RBAC 作用域

修改 `backend/crates/server/src/admin_routes.rs` 的 `APPS`：

```rust
const APPS: &[&str] = &[
    "news",
    "news.blog",
    "news.activity",
    "news.store",
    "news.points",
    "art",
    "exam",
    "novel",
    "shop",
    "widget",
    "console",
];
```

如果需要子作用域，添加 `"widget.foo"`、`"widget.bar"`，handler 中使用 `authorize(..., "widget.foo", ...)`。父作用域 `widget` 会覆盖所有 `widget.*` 子作用域。

## 7. 前端 app

新增 `apps/<m>/package.json`：

```json
{
  "name": "@haruhi/widget",
  "version": "1.0.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "@haruhi/api-client": "workspace:*",
    "vue": "^3.5.22",
    "vue-router": "^4.6.3"
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^6.0.1",
    "vite": "^7.1.11"
  }
}
```

TypeScript app 可参考 `apps/exam` 或 `apps/console`，把 build 改为：

```json
"build": "vue-tsc --noEmit && vite build"
```

新增 `apps/<m>/vite.config.js`：

```js
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  base: '/widget/',
  plugins: [vue()],
  server: {
    port: 5210,
    proxy: {
      '/api': { target: 'http://127.0.0.1:17777', changeOrigin: true },
      '/uploads': { target: 'http://127.0.0.1:17777', changeOrigin: true },
    },
  },
})
```

已占用端口：

| app     | 端口 |
| ------- | ---- |
| console | 5200 |
| art     | 5201 |
| exam    | 5202 |
| novel   | 5203 |
| news    | 5204 |
| shop    | 5205 |

接入共享客户端：

```js
import { createApiClient, createAdminAuth, resolveUploadUrl } from '@haruhi/api-client'

const api = createApiClient('/api/widget')
const auth = createAdminAuth('widget')

const items = await api.get('/items')
const uploadUrl = resolveUploadUrl(item.image_path)
```

## 8. Nginx

生产和测试站都要加子路径：

```nginx
location /widget/ {
    alias /var/www/haruhifanclub/apps/widget/dist/;
    try_files $uri $uri/ /widget/index.html;
}
```

测试站路径改成 `/var/www/haruhifanclub-test/apps/widget/dist/`。

`/api/` 和 `/uploads/` 已经覆盖所有模块，不需要新增 location。

## 验收

```bash
cargo build -p haruhi-server
cargo test --workspace
pnpm --filter @haruhi/widget build
pnpm lint
```

手动检查：

- `data/widget.db` 已生成。
- `GET /api/widget/items` 返回正常。
- 后台接口无 token 返回 401，越权 token 返回 403。
- `/console/` 能给用户分配 `widget` 角色。
- `pnpm --filter @haruhi/widget dev` 可访问 `/widget/`。
- Nginx 配置 `nginx -t` 通过。
