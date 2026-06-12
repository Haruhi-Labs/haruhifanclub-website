# haruhi-auth

后端鉴权和授权库。它只提供函数和 axum 提取器，不监听端口；由 `haruhi-server` 在路由中使用。

## 职责

- JWT 签发和校验。
- argon2 密码哈希和校验。
- RBAC 角色等级判定。
- `AuthUser` / `Option<AuthUser>` axum 提取器。

身份来自 JWT，权限每次查 `core.db`，所以禁用账号或修改角色会立即影响后续请求。

## 主要 API

令牌：

| API                                                   | 说明                         |
| ----------------------------------------------------- | ---------------------------- |
| `issue_token(secret, user_id, is_super, ttl_seconds)` | 签发 JWT                     |
| `decode_token(secret, token)`                         | 校验并解析 JWT               |
| `Claims`                                              | `sub`、`super`、`iat`、`exp` |

密码：

| API                               | 说明                           |
| --------------------------------- | ------------------------------ |
| `hash_password(password)`         | 生成 argon2 PHC 哈希           |
| `verify_password(password, hash)` | 校验密码，非法哈希返回 `false` |

RBAC：

| API                                  | 说明                                  |
| ------------------------------------ | ------------------------------------- |
| `Action`                             | `Read`、`Write`、`Moderate`、`Manage` |
| `role_level(core, user_id, app)`     | 查询用户在某作用域的角色等级          |
| `authorize(core, user, app, action)` | 校验用户是否满足 action 要求          |
| `scope_chain(app)`                   | 展开点号父作用域                      |
| `require_super(user)`                | 只允许超管                            |

提取器：

| API                | 说明                                        |
| ------------------ | ------------------------------------------- |
| `AuthUser`         | 从 `Authorization: Bearer <token>` 提取用户 |
| `Option<AuthUser>` | 无 token 或非法 token 时为 `None`           |
| `AuthSecret`       | 从 `AppState` 取 JWT secret                 |

## 角色和作用域

角色等级：

```text
viewer(1) < editor(2) < moderator(3) < admin(4)
```

动作等级：

```text
Read(1) < Write(2) < Moderate(3) < Manage(4)
```

`authorize()` 规则：

1. 超管直接通过。
2. 普通用户必须在 `users` 表中为 `active`。
3. 按作用域链查询最高角色等级。
4. 最高等级达到 action 要求才通过。

作用域链示例：

```text
scope_chain("news.activity") -> ["news.activity", "news"]
scope_chain("news")          -> ["news"]
```

父作用域角色覆盖子作用域，子作用域不会反向覆盖父作用域。

## 使用位置

- `auth_routes.rs`：登录时 `verify_password()`，成功后 `issue_token()`。
- `admin_routes.rs`：超管接口调用 `require_super()`。
- 各业务模块后台接口：调用 `authorize(&state.pools.core, &user, "<module>", Action::X).await?`。
- `state.rs`：为 `AppState` 实现 `FromRef<AppState> for AuthSecret`。

## 开发

```bash
cargo test -p haruhi-auth
cargo build -p haruhi-auth
```

测试使用内存 SQLite，不需要外部数据库或 `DATABASE_URL`。
