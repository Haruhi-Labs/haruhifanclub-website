# 安全策略

感谢你帮助 **haruhifanclub** 变得更安全。本项目后端涉及 **JWT 鉴权、RBAC 权限、文件上传** 等敏感面，
我们非常重视安全问题，并承诺对负责任披露的报告者表示感谢。

## 请负责任地披露

**请不要通过公开的 GitHub Issue / PR / Discussion 披露安全漏洞，也不要公开 0day。**
公开披露会让尚未修复的漏洞被恶意利用，伤害到所有用户。

请改为**私下**联系维护者：

> **安全联系：GitHub 用户 [@Haruhiyuki](https://github.com/Haruhiyuki)。**
> 优先使用 GitHub 的 **Private vulnerability reporting**（仓库 → Security → Report a vulnerability）。

报告时请尽量提供：

- 漏洞类型与受影响的模块 / crate（如 `auth`、`media`、某个 `apps/<app>`）。
- 复现步骤或 PoC（最小可复现示例最佳）。
- 影响评估（可读取/篡改的数据、可绕过的权限、是否需登录等）。
- 你认为可行的修复或缓解思路（可选）。

## 我们的响应承诺

- **确认**：在 **3 个工作日**内确认收到你的报告。
- **评估**：尽快评估严重程度并与你沟通修复计划与时间线。
- **修复与披露**：修复发布后，在征得你同意的前提下进行协调披露，并在致谢中署名（如你愿意）。
- 在漏洞被修复并给用户留出合理升级时间之前，**请对漏洞细节保密**。

## 支持范围

| 范围 | 是否受支持 |
| --- | --- |
| `main` 分支最新代码 | ✅ 是 |
| 旧的历史标签 / 已废弃分支 | ❌ 否（请先更新到 `main` 最新代码） |
| 第三方依赖自身的漏洞 | 视情况转报上游；本仓 CI 的 `audit` 关卡会持续暴露依赖 CVE |

## 重点关注面（贡献者自查）

提交涉及以下方面的改动时，请额外当心，并在 PR 中说明你的安全考量：

- **鉴权 / 令牌（`auth`）**：JWT 签发与校验、过期、密钥（`HARUHI_JWT_SECRET`）来源；避免把密钥写进代码或日志。
- **权限 / RBAC（`auth` + `console`）**：后台端点务必首行 `authorize(&pools.core, &user, "<module>", Action::X)`；
  仅超管端点用 `require_super`；注意父级作用域继承（见 [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)）。
- **文件上传 / 媒体（`media`）**：校验类型与大小，注意路径穿越，上传只落到 `uploads/<module>/`。
- **SQL（各模块 + `db`）**：用 sqlx 参数化查询（运行时校验的 `query` / `query_as`），杜绝字符串拼接 SQL。
- **密钥与运行时数据**：`.env`、`data/`、`uploads/` 已在 `.gitignore` 中——**切勿提交**任何密钥或真实用户数据。

---

相关文档：[贡献指南](CONTRIBUTING.md) · [行为准则](CODE_OF_CONDUCT.md)
