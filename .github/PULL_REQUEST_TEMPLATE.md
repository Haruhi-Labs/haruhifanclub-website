<!--
  感谢贡献！请用中文填写本模板。
  PR 标题必须符合 Conventional Commits：type(scope): subject
  type ∈ {feat fix perf refactor docs style test build ci chore revert}（受约束）
  scope 可选、不限定取值：用它点明改动范围（如 news / server / deploy …，建议见 CONTRIBUTING）
  例：feat(news): 支持公告置顶 / fix(server): 修复登录态过期判断
  squash 合并时本 PR 标题即最终提交信息，请务必规范。
-->

## 变更摘要

<!-- 简要说明本次改动做了什么、为什么这么做。 -->

## 关联 issue

<!-- 用关键字自动关闭：Closes #123 / Fixes #123 / Refs #123；无则填「无」。 -->

Closes #

## 影响范围

<!-- 勾选本次改动涉及的模块（可多选）。 -->

### 前端 app

- [ ] news（/news/）
- [ ] art（/art/）
- [ ] exam（/exam/，TS）
- [ ] novel（/library/）
- [ ] shop（/shop/）
- [ ] console（/console/，TS，RBAC 超管台）

### 共享前端包

- [ ] api-client（@haruhi/api-client）

### 后端 crate

- [ ] server
- [ ] core
- [ ] db
- [ ] auth
- [ ] media
- [ ] ai
- [ ] mail

### 跨领域

- [ ] deploy（部署 / nginx / systemd）
- [ ] ci（CI / 工作流）
- [ ] docs（文档）

## 自检清单

<!-- 完成的项请勾选；不适用的项请保留未勾并在说明里注明原因。 -->

- [ ] 本地已跑 `pnpm lint`（eslint + cargo fmt --check + cargo clippy -D warnings）
- [ ] 本地已跑 `cargo test`（涉及后端时）
- [ ] 已构建相关 app：`pnpm -r --filter "./apps/*" build`（涉及前端时；exam/console 含 vue-tsc 类型检查）
- [ ] PR 标题 / 各提交遵守 Conventional Commits 规范
- [ ] 已同步更新相关文档（README / docs/）

## 给评审者的说明

<!-- 评审重点、已知取舍、需要特别注意的地方、临时占位或 TODO。 -->

## 破坏性变更 / 数据库迁移

- [ ] 含破坏性变更（breaking change）
- [ ] 含数据库迁移（backend/migrations/）

<!-- 若以上任一勾选，请在此描述迁移步骤、回滚方案与对线上数据的影响。 -->
