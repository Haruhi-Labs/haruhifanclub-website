# @haruhi/ui （预留占位，尚未启用）

共享 UI 组件 / 主题 token 的**预留包**，目前**为空、未启用**——没有 `package.json`，未纳入 pnpm workspace，任何 app 都**不应** import 它。

## 为什么留着空目录

整合方案（见根 [`../../README.md`](../../README.md) 与计划）刻意让 6 个前端 app **各自保留技术栈与样式**（原生 CSS / CSS vars / SCSS / Tailwind 各不相同），**不强行统一 UI**——强抽公共组件属过度工程。此目录仅作为"将来若出现高复用、跨 app 同构的展示型组件（如 NavBar/Modal/Toast）再渐进抽取"的预留位。

真正已启用的共享前端代码在 [`../api-client`](../api-client)（鉴权 / fetch / `resolveUploadUrl`）。

## 启用时

补 `package.json`（name `@haruhi/ui`、`private: true`）、按需导出组件，再在目标 app 里 `"@haruhi/ui": "workspace:*"`。在此之前，请勿依赖本目录。
