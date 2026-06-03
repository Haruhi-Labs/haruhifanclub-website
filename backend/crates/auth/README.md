# haruhi-auth

haruhi-server 的鉴权与授权库 crate：统一 **JWT 单点登录** + **argon2 密码** + **基于角色等级与作用域链的 RBAC**。本 crate 只是纯函数与提取器的工具库，不含二进制、不监听端口，由 `haruhi-server` 装配进路由后生效。

设计取向：**身份无状态、权限即时**。登录身份完全来自 JWT（无服务端会话）；而每次授权检查都现查 `core.db`，因此角色调整或账号停用立刻生效，无需等待令牌过期。

## 技术栈 / 关键依赖

- `jsonwebtoken` — JWT 签发/校验（HS256，默认 60s 时钟容差）
- `argon2` + `rand_core`(OsRng) — 密码哈希与校验（PHC 字符串格式）
- `sqlx`(SqlitePool) — 查询 `core.db` 的用户状态与角色等级
- `axum` — 实现 `FromRequestParts` / `OptionalFromRequestParts` 提取器
- `haruhi-core` — 复用 `AppError` / `AppResult`（鉴权失败映射为统一 JSON 错误）
- `serde`、`chrono`、`tracing`

## 结构要点

单文件库，全部 API 在 `src/lib.rs`，按职责分四段：令牌、密码、RBAC、axum 提取器。文件尾部含较完整的单元/集成测试（令牌往返、密码校验、作用域链、内存库上的 RBAC 隔离/继承/门控）。

## 公开 API

令牌
- `issue_token(secret, user_id, is_super, ttl_seconds) -> AppResult<String>`
- `decode_token(secret, token) -> AppResult<Claims>`（非法/过期一律 `AppError::Unauthorized`）
- `Claims { sub, is_super, iat, exp }`（`is_super` 序列化为 JSON 字段 `super`）

密码
- `hash_password(password) -> AppResult<String>`
- `verify_password(password, hash) -> bool`（哈希串本身非法时返回 `false`，不 panic）

RBAC
- `Action`：`Read`(1) < `Write`(2) < `Moderate`(3) < `Manage`(4)，`Action::level()` 取等级
- `role_level(core, user_id, app) -> AppResult<Option<i64>>`：查该用户在指定 app 的角色等级
- `authorize(core, user, app, action) -> AppResult<()>`：超管直通；否则要求账号 `status = 'active'` 且沿作用域链取到的最高角色等级 ≥ `action.level()`，不达标返回 `AppError::Forbidden`
- `scope_chain(app)`：由具体到顶层切分点号，如 `\"news.activity\"` → `[\"news.activity\", \"news\"]`
- `require_super(user) -> AppResult<()>`：仅超管通过

提取器
- `AuthUser { id, is_super }`：实现 `FromRequestParts`，从 `Authorization: Bearer <token>` 解码；缺头/非法 → `AppError::Unauthorized`
- `Option<AuthUser>`：实现 `OptionalFromRequestParts`，缺头或 token 非法时为 `None`，用于「可选登录」端点（游客投稿、公开详情）
- `AuthSecret(Arc<String>)`：JWT 密钥载体，需让 `AppState` 实现 `FromRef<AppState> for AuthSecret`，提取器才能取到密钥

## 关键约定

- **作用域链继承**：拥有父作用域角色（如 `news`）的用户，对其所有子作用域（`news.activity`、`news.points` …）自动有效；反之子作用域角色不上溯父级，同级子作用域之间互相隔离。
- **超管短路**：`is_super` 为真时 `authorize` / `require_super` 直接放行，不查 DB。
- **账号停用即拒**：非超管用户若 `status != 'active'`，即便持有有效令牌也被 `authorize` 拒绝。
- 依赖 `core.db` 中的 `users`、`roles`、`user_app_roles` 三张表（角色等级存于 `roles.level`）。

## 本地开发

本 crate 无独立运行入口，随后端工作区一起构建与测试：

```bash
# 在仓库根目录
cargo test -p haruhi-auth      # 运行本 crate 单元/集成测试
cargo build -p haruhi-auth     # 单独编译

cargo run -p haruhi-server     # 启动后端（:17777），auth 在此被装配
```

测试用内存 SQLite（`sqlite::memory:`，`max_connections=1`），无需外部数据库或 `DATABASE_URL`。

## 与后端的关系

- `haruhi-server` 在 `state.rs` 中为 `AppState` 实现 `FromRef<AppState> for AuthSecret`（密钥取自 `cfg.jwt_secret`），使各 handler 可直接以 `user: AuthUser` 作为提取器参数。
- 登录流程（`auth_routes.rs`）：`verify_password` 校验后 `issue_token` 签发 JWT。
- 超管台接口（`admin_routes.rs`）统一 `require_super(&user)?`，建/改用户走 `hash_password`。
- 各业务模块后台接口用 `authorize(&state.pools.core, &user, \"<module>\", Action::X).await?` 做按 app 的等级门控（如 `news.activity` 用 `Action::Manage`）。
- 鉴权失败经 `haruhi-core::AppError`（`Unauthorized` / `Forbidden`）统一渲染为 JSON。

## 更多

- 后端与各 crate 总览见仓库根 [README](../../../README.md)
- 协作规范、提交 scope 见 [CONTRIBUTING](../../../CONTRIBUTING.md)
- 新增带 RBAC 的业务模块参见 `docs/ADDING_MODULE.md`（`novel` 为端到端模板）
