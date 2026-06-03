# haruhi-mail

统一邮件发送 crate：把一封已经渲染好的邮件发出去，支持 **Resend API** 与 **SMTP** 双驱动。替代旧 shop 站内置的 nodemailer / Resend 发信逻辑。

定位很窄——本 crate **只负责发送单封邮件**。邮件队列（`email_jobs` 入队、重试退避、后台 worker）以及模板渲染都在 `server` 的 shop 模块里实现（`server/src/modules/shop/email.rs`），不在这里。

## 技术栈 / 关键依赖

- `lettre`（`AsyncSmtpTransport` + `Tokio1Executor`）——SMTP 异步发送
- `reqwest`——调用 Resend `POST /emails` 接口
- `haruhi-core`——复用 `Config` / `MailConfig`（环境变量映射）
- `tokio` / `serde` / `serde_json` / `anyhow` / `tracing`

## 结构要点

单文件 crate，全部逻辑在 `src/lib.rs`：

- `Mailer`——发信句柄，持有 `MailConfig` 与一个复用的 `reqwest::Client`，`Clone` 廉价
- `Mailer::from_config(&Config) -> Option<Self>`——`mail.enabled == false` 时返回 `None`，业务层据此跳过发送
- `Mailer::send(to, subject, html, text)`——异步发送，同时携带纯文本与 HTML 两个 part
- `Provider`（内部枚举 `Resend` / `Smtp`）+ `resolve_provider()`——根据配置挑选驱动
- `send_resend` / `send_smtp`——两个驱动的具体实现

## 关键特性与约定

- **驱动选择**（`MAIL_PROVIDER`）：
  - `resend` / `smtp`：强制指定
  - `auto`（默认）：有 `RESEND_API_KEY` 优先 Resend，否则有 `SMTP_HOST` 用 SMTP，都没有则报错
- **SMTP TLS**：`SMTP_SECURE=true`（465 隐式 TLS）走 `relay`，否则走 `starttls_relay`；端口由 `SMTP_PORT` 指定（默认 465）。配了 `SMTP_USER` + `SMTP_PASS` 才带认证
- **发件人**：`"{MAIL_FROM_NAME} <{MAIL_FROM_ADDRESS}>"`，缺 `MAIL_FROM_ADDRESS` 直接报错；`MAIL_REPLY_TO` 存在时给 Resend 带上 `reply_to`
- **正文**：调用方传入已渲染好的 `html` 与 `text`，本 crate 不做模板渲染。SMTP 用 `MultiPart::alternative`（text + html），Resend 同时传 `html` 与 `text` 字段
- **错误**：统一 `anyhow::Result`；Resend 非 2xx 会把状态码与响应体一并 `bail!`

## 配置（环境变量，经 `core::MailConfig`）

| 变量 | 默认 | 说明 |
| --- | --- | --- |
| `MAIL_ENABLED` | `false` | 关闭时 `from_config` 返回 `None` |
| `MAIL_PROVIDER` | `auto` | `resend` / `smtp` / `auto` |
| `MAIL_FROM_NAME` | `春日商城` | 发件人显示名 |
| `MAIL_FROM_ADDRESS` | 无 | 发件地址（发送时必填） |
| `MAIL_REPLY_TO` | 无 | 可选回信地址 |
| `RESEND_API_KEY` | 无 | Resend 鉴权（Bearer） |
| `RESEND_API_BASE_URL` | `https://api.resend.com` | Resend 基址 |
| `SMTP_HOST` | 无 | SMTP 主机 |
| `SMTP_PORT` | `465` | SMTP 端口 |
| `SMTP_SECURE` | `true` | 隐式 TLS / starttls |
| `SMTP_USER` / `SMTP_PASS` | 无 | SMTP 认证（成对生效） |

## 本地开发

本 crate 是库，无独立二进制，随 `haruhi-server` 一起编译运行：

```bash
# 仅类型检查 / 编译本 crate
cargo build -p haruhi-mail

# 在 server 中实际使用（监听 127.0.0.1:17777）
cargo run -p haruhi-server
```

不设 `MAIL_ENABLED=true` 时邮件整体关闭：shop 的 worker 不启动，`email_jobs` 留在队列等配置就绪。本地想真正发信，按上表配好 Resend 或 SMTP 并置 `MAIL_ENABLED=true`。

## 与后端其它部分的关系

- **被谁用**：`server` 的 shop 模块（`modules/shop/email.rs`）。worker 用 `Mailer::from_config(&cfg)` 构造（`None` 即不启动 worker），逐条取队列后调用 `mailer.send(...)`，并按其结果在 shop 库里更新 `email_jobs` 状态与退避
- **依赖谁**：`haruhi-core`（配置）
- **不含**：队列、重试、模板、HTTP 路由——这些都在 shop 模块

## 更多

- 仓库总览与架构：[根 README](../../../README.md)
- 协作与提交规范（scope 用 `mail`）：[CONTRIBUTING](../../../CONTRIBUTING.md)
