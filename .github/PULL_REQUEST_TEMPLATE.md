<!--
PR 标题使用 Conventional Commits：type(scope): subject
type: feat fix perf refactor docs style test build ci chore revert
示例：feat(news): 支持公告置顶
-->

## 变更摘要

<!-- 做了什么，为什么改。 -->

## 关联 issue

<!-- 例如 Closes #123；没有就填“无”。 -->

无

## 影响范围

### 前端 app

- [ ] news
- [ ] chapter
- [ ] art
- [ ] exam
- [ ] novel
- [ ] shop
- [ ] console

### 共享前端

- [ ] api-client

### 后端

- [ ] server
- [ ] core
- [ ] db
- [ ] auth
- [ ] media
- [ ] ai
- [ ] mail

### 其它

- [ ] deploy / nginx / systemd
- [ ] CI
- [ ] 文档

## 验证

<!-- 勾选已执行项；不适用的项可说明原因。 -->

- [ ] `pnpm lint`
- [ ] `cargo test --workspace`
- [ ] `pnpm --filter @haruhi/<app> build`
- [ ] 手动验证：

## 数据库 / 部署影响

- [ ] 包含 schema / DDL 变更
- [ ] 包含部署配置变更
- [ ] 包含破坏性变更

<!-- 如勾选，请写明更新步骤、回滚方式和线上影响。 -->

无

## 评审提示

<!-- 需要重点看的文件、已知取舍、临时限制。 -->
