# haruhi-mail

邮件发送库。只负责把一封已渲染好的邮件发出去，支持 Resend API 和 SMTP。

邮件队列、模板、重试退避在 `haruhi-server` 的 shop 模块中实现，不在本 crate。

## 主要 API

| API                                     | 说明                                 |
| --------------------------------------- | ------------------------------------ |
| `Mailer::from_config(&Config)`          | `MAIL_ENABLED=false` 时返回 `None`   |
| `Mailer::send(to, subject, html, text)` | 发送一封同时包含 HTML 和纯文本的邮件 |

内部驱动：

- Resend：`POST {RESEND_API_BASE_URL}/emails`
- SMTP：`lettre::AsyncSmtpTransport<Tokio1Executor>`

## 驱动选择

`MAIL_PROVIDER`：

| 值       | 行为                                                      |
| -------- | --------------------------------------------------------- |
| `resend` | 强制使用 Resend                                           |
| `smtp`   | 强制使用 SMTP                                             |
| `auto`   | 有 `RESEND_API_KEY` 用 Resend，否则有 `SMTP_HOST` 用 SMTP |

`auto` 找不到可用配置时返回错误。

## 配置

| 变量                      | 默认                     | 说明                                            |
| ------------------------- | ------------------------ | ----------------------------------------------- |
| `MAIL_ENABLED`            | `false`                  | 是否启用邮件                                    |
| `MAIL_PROVIDER`           | `auto`                   | `resend` / `smtp` / `auto`                      |
| `MAIL_FROM_NAME`          | `春日商城`               | 发件人显示名                                    |
| `MAIL_FROM_ADDRESS`       | 空                       | 发件地址，启用邮件后必填                        |
| `MAIL_REPLY_TO`           | 空                       | 回复地址                                        |
| `RESEND_API_KEY`          | 空                       | Resend key                                      |
| `RESEND_API_BASE_URL`     | `https://api.resend.com` | Resend base URL                                 |
| `SMTP_HOST`               | 空                       | SMTP 主机                                       |
| `SMTP_PORT`               | `465`                    | SMTP 端口                                       |
| `SMTP_SECURE`             | `true`                   | `true` 使用 465 隐式 TLS；`false` 使用 starttls |
| `SMTP_USER` / `SMTP_PASS` | 空                       | SMTP 认证                                       |

## 使用位置

`haruhi-server` 的 shop 模块调用：

```rust
let mailer = Mailer::from_config(&cfg);
```

当 `mailer` 为 `None` 时，shop 邮件 worker 不发送邮件，队列记录留待配置后处理。

## 开发

```bash
cargo build -p haruhi-mail
```

本 crate 没有独立二进制。联调邮件需要在 `.env` 中设置 `MAIL_ENABLED=true` 并配置 Resend 或 SMTP。
