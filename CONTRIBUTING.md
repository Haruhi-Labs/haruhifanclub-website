# 贡献指南

本文只说明如何参与贡献、如何协作、以及提交和评审需要遵守的规范。

本地启动、依赖安装、调试命令和部署步骤不在本文重复维护，请看：

- [README.md](README.md)：项目入口、快速上手、常用命令
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)：架构和运行时约定
- [docs/ADDING_MODULE.md](docs/ADDING_MODULE.md)：新增业务模块
- [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md)：部署、环境变量、备份

## 协作流程

1. 从最新 `main` 切出短生命周期分支。
2. 分支命名按改动目的表达，例如 `feat/novel-bookmark`、`fix/shop-order-total`、`docs/contributing-scope`。
3. 改动保持聚焦。跨前端、后端、部署或数据结构的改动，需要在 PR 里明确影响范围。
4. 提交 PR 时填写模板里的变更摘要、影响范围、验证方式、数据库/部署影响和评审提示。
5. PR 标题使用 Conventional Commits。仓库采用 squash merge，PR 标题会成为 `main` 上的最终提交信息。
6. CI 通过、评审完成后由维护者合并。

不完整但需要提前讨论的工作请开 draft PR。不要把无关格式化、依赖升级、重命名和业务改动混在同一个 PR。

## PR 规范

PR 描述应回答四件事：

- 改了什么，为什么改。
- 影响哪些 app、crate、接口、数据库、部署配置或文档。
- 做过哪些验证；未验证的部分要说明原因。
- 是否有兼容性、迁移、回滚或上线顺序要求。

涉及下列内容时，必须在 PR 中显式说明：

- API 请求/响应、鉴权、RBAC 权限语义变化。
- 数据库 schema、迁移脚本、默认数据或回填逻辑变化。
- 环境变量、部署脚本、Nginx/systemd 配置变化。
- 上传、媒体处理、邮件、AI 审核等外部依赖行为变化。
- 安全相关修复或可能影响权限边界的改动。

## 提交规范

PR 标题和建议的提交信息格式：

```text
type(scope): subject
```

`scope` 可选，建议使用 app、crate 或领域名，例如 `news`、`shop`、`server`、`api-client`、`deploy`、`docs`、`workflows`。

合法 `type`：

```text
feat fix perf refactor docs style test build ci chore revert
```

示例：

```text
feat(novel): 支持 EPUB 章节书签
fix(shop): 修正预售订单运费计算
docs(readme): 补充本地 .env 启动步骤
ci(workflows): 拆分前端检查并改为手动审计
```

规范要求：

- subject 使用中文，写清具体行为，不写空泛描述。
- PR 标题必须符合上述格式，因为 squash 后它就是最终提交信息。
- 中间提交会被 squash 丢弃，但仍建议按同一格式写，便于评审。

## 评审规范

评审优先看行为正确性、回归风险、权限边界、数据兼容性和测试覆盖。代码风格问题由工具优先处理，不把格式偏好当作主要评审意见。

作者处理评审意见时应：

- 直接修复明确的问题。
- 对有取舍的建议，在 PR 评论里说明选择和理由。
- 如果发现原 PR 范围过大，拆出后续 issue 或后续 PR，不在当前 PR 里继续扩张。

评审者提出阻塞意见时应说明具体文件、行为风险和期望结果，避免只给抽象判断。

## 自查与 CI

本地自查按改动范围选择执行，具体命令以 README 和 package scripts 为准。PR 模板中的验证清单用于记录实际跑过的检查。

CI 规则：

- `ci-ok` 是 CI 聚合门禁。PR 合并前应确认它通过。
- `frontend`、`backend`、`backend-full` 等路径过滤 job 未命中路径时会 skipped，这是正常结果。
- `frontend-lint` 只跑一次全仓 ESLint，避免在 app matrix 中重复输出。
- `audit.yml` 是手动依赖审计，只作信息参考，不参与 PR/push 门禁。
- PR 标题校验由 `pr-checks.yml` 执行，只校验 PR 标题，不逐条校验中间提交。

如果 CI 失败，先修复失败原因，再请求复审。不要通过改 workflow 来绕过真实失败；只有当 CI 规则本身过期或误判时，才修改 workflow。

## 代码规范

通用要求：

- 文档、注释、PR 标题和提交 subject 使用中文。
- 保持改动局部化，优先沿用现有目录结构、命名方式和工具链。
- 不提交运行时数据、上传文件、密钥或本地配置。
- 改公共接口、环境变量、部署脚本或数据库 schema 时，同步更新 README 或 `docs/`。

前端：

- 遵守根目录 ESLint flat config 和 Prettier 配置。
- 共享请求、鉴权、上传 URL、RBAC 等能力优先放在 `packages/api-client`，不要在各 app 里复制协议逻辑。
- `exam` 和 `console` 是 TypeScript app，涉及类型边界时要保证 `vue-tsc` 能通过。

后端：

- handler 先做授权，再执行业务逻辑。普通后台端点使用 `authorize`，仅超管端点使用 `require_super`。
- SQL 使用 `sqlx::query` / `query_as`，不使用 `query!` 宏。
- 需要持久化结构变化时，在对应 `backend/migrations/<module>/` 下补迁移。
- 公共逻辑调整要关注 `backend/crates/server/tests/integration.rs` 的模块级回归覆盖。

## 新增模块

新增业务模块不要只提交单侧代码。PR 至少需要说明这些接线点是否涉及：

- 后端模块、数据库 schema 和 RBAC 权限。
- 前端 app、路由子路径和构建配置。
- 管理后台入口、上传路径、Nginx/部署配置。
- 文档、测试和上线/回滚说明。

具体步骤看 [docs/ADDING_MODULE.md](docs/ADDING_MODULE.md)，不要在贡献指南里重复维护操作清单。

## 安全

安全问题不要直接公开提交可利用细节。请按 [SECURITY.md](SECURITY.md) 私下披露或先联系维护者确认处理方式。
