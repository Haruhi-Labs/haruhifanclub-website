# 安全策略

本项目涉及 JWT、RBAC、文件上传、订单和后台管理。安全问题请私下披露，不要公开 issue、PR 或 Discussion。

## 披露方式

优先使用 GitHub 的 Private vulnerability reporting：

```text
Repository -> Security -> Report a vulnerability
```

也可以联系维护者：GitHub 用户 [@Haruhiyuki](https://github.com/Haruhiyuki)。

报告中请尽量包含：

- 受影响范围，例如 `auth`、`media`、`shop`、`apps/<app>`。
- 复现步骤或 PoC。
- 影响说明，例如能读写哪些数据、是否能绕过登录或权限。
- 建议修复方式，可选。

## 响应

- 3 个工作日内确认收到报告。
- 评估严重程度后沟通修复计划。
- 修复发布后再协调披露。
- 如你愿意，可在致谢中署名。

漏洞修复并给使用方留出升级时间前，请不要公开细节。

## 支持范围

| 范围             | 支持                                   |
| ---------------- | -------------------------------------- |
| `main` 最新代码  | 是                                     |
| 旧标签、废弃分支 | 否，请先更新到 `main`                  |
| 第三方依赖漏洞   | 视情况转报上游；本仓依赖审计会提示 CVE |

## 现有基线防护

改动时不要削弱以下已就位的防护：

- **SQL**：值一律走 sqlx 参数绑定；动态拼接的列名（如 news `dynamic_update` 取自请求体 key）必须过合法标识符白名单。
- **匿名上传**：art 画廊、exam 试卷按设计保留匿名上传，但有类型/大小白名单（`media::check_image` / `check_media`，图片 ≤32MB、音频 ≤64MB，exam 仅接受图片/音频）+ per-IP 限流（`upload_limiter`，10 分钟 60 次）。
- **错误**：所有 5xx 统一返回「服务器内部错误」，详情仅入日志，不向客户端外泄库表结构/路径（`core::error`）。
- **不安全默认值**：JWT/ART_COOKIE 密钥与 `admin/admin123` 超管仅在「debug 构建 + 绑定回环地址」时启用；debug 绑非回环或 release 一律要求显式配置。
- **前端 XSS**：渲染外部/用户内容的 `v-html`（novel EPUB、news 活动详情）经 DOMPurify 净化；其余 `v-html` 已先 `escapeHtml`。
- **边缘**：nginx `/uploads/` 带 CSP `sandbox`（防恶意 SVG 直链 XSS）、TLS 仅 1.2/1.3。

## 贡献者自查

改动涉及以下内容时，在 PR 里写明安全考虑：

- `auth`：JWT 签发/校验、过期、密钥来源、密码哈希。
- RBAC：后台端点是否调用 `authorize()` 或 `require_super()`。
- `media`：上传大小、类型、路径穿越、落盘目录；公开上传口是否复用 `check_image`/`check_media` 与限流。
- SQL：是否使用 sqlx 参数绑定，避免字符串拼接 SQL；动态列名是否过白名单。
- 密钥和数据：不要提交 `.env`、`data/`、`uploads/` 或真实用户数据。

相关文档：[CONTRIBUTING.md](CONTRIBUTING.md) · [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
