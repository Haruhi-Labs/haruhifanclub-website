# haruhi-core

后端基础库。提供配置读取、错误响应和通用解析工具，不包含业务逻辑、数据库连接或 HTTP 路由。

## 模块

```text
src/
  lib.rs        re-export Config、MailConfig、AppError、AppResult
  config.rs     环境变量 -> Config
  error.rs      AppError -> JSON response
  parse.rs      文本和数值解析工具
```

## Config

`Config::from_env()` 从环境变量读取配置。调用方需要先执行 `dotenvy::dotenv()`，当前由 `haruhi-server` 负责。

主要字段：

| 字段                       | 环境变量                                                             |
| -------------------------- | -------------------------------------------------------------------- |
| `bind`                     | `HARUHI_BIND`，代码默认 `127.0.0.1:8080`                             |
| `data_dir`                 | `HARUHI_DATA_DIR`，默认 `./data`                                     |
| `uploads_dir`              | `HARUHI_UPLOADS_DIR`，默认 `./uploads`                               |
| `jwt_secret`               | `HARUHI_JWT_SECRET`                                                  |
| `jwt_ttl_seconds`          | `HARUHI_JWT_TTL_SECONDS`                                             |
| `superadmin_user/password` | `HARUHI_SUPERADMIN_USER` / `HARUHI_SUPERADMIN_PASSWORD`              |
| AI                         | `DASHSCOPE_API_KEY`、`AI_API_URL`、`AI_TEXT_MODEL`、`AI_IMAGE_MODEL` |
| art cookie                 | `ART_COOKIE_SECRET`                                                  |
| shop/mail                  | `SHOP_FREE_SHIPPING_THRESHOLD`、`MAIL_*`、`RESEND_*`、`SMTP_*`       |
| CORS                       | `HARUHI_CORS_ORIGINS`、`PUBLIC_SITE_URL`                             |

注意：本地 Vite 代理和部署模板都使用 `127.0.0.1:17777`，请通过 `.env` 设置 `HARUHI_BIND=127.0.0.1:17777`。

release 构建下，缺少以下变量会启动失败：

- `HARUHI_JWT_SECRET`
- `ART_COOKIE_SECRET`

不安全默认值（JWT/ART_COOKIE 密钥、默认 seed `admin/admin123` 超管）只在「debug 构建 **且** 绑定回环地址」时启用，仅供本地开发。debug 绑非回环地址或 release 构建都要求显式配置，否则启动失败/不 seed。

## AppError

handler 约定返回 `AppResult<T>`。`AppError` 实现 `IntoResponse`，响应体格式：

```json
{ "error": "错误信息" }
```

状态码映射：

| 错误                         | 状态码 |
| ---------------------------- | ------ |
| `BadRequest`                 | 400    |
| `Unauthorized`               | 401    |
| `Forbidden`                  | 403    |
| `NotFound`                   | 404    |
| `Conflict`                   | 409    |
| `TooManyRequests`            | 429    |
| `Database(RowNotFound)`      | 404    |
| 其它 `Database` / `Internal` | 500    |

4xx 的 `error` 文案是有意给用户看的；所有 5xx（含 `Internal` 与 anyhow 转换）一律写日志，但对客户端统一返回「服务器内部错误」，不外泄库表结构/路径等内部细节。

## parse 工具

| API                 | 说明                             |
| ------------------- | -------------------------------- |
| `parse_int_radix10` | 模拟 JS `Number.parseInt(x, 10)` |
| `parse_int_or`      | 无效或 0 时返回默认值            |
| `num_i64`           | 从 JSON number/string 中取整数   |
| `clamp_int`         | 解析浮点后 floor，并限制范围     |
| `clamp_len`         | 按字符数截断                     |
| `safe_text`         | `trim`，`None` 转空字符串        |

## 开发

```bash
cargo test -p haruhi-core
cargo build -p haruhi-core
```

实际运行通过 `haruhi-server`：

```bash
cargo run -p haruhi-server
```
