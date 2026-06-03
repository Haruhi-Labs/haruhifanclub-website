# @haruhi/config （预留占位，尚未启用）

共享构建 / lint 预设（vite、eslint、tsconfig 等）的**预留包**，目前**为空、未启用**——没有 `package.json`，未纳入 pnpm workspace。

## 现状

工程化基线目前**集中在根目录**、已足够：根 `eslint.config.js`（flat config，覆盖 `apps/*` 与 `packages/*`）、根 `.prettierrc.json`、`rust-toolchain.toml`、`.nvmrc`。各 app 的 `vite.config.*` / `tsconfig.json` 体量很小、彼此差异有意保留，暂无抽公共预设的必要。

此目录仅为"将来若预设重复到值得收敛时再抽取"的预留位。详见根 [`../../README.md`](../../README.md) 与 [`../../CONTRIBUTING.md`](../../CONTRIBUTING.md)。

## 启用时

补 `package.json`（name `@haruhi/config`、`private: true`），导出可被各 app `extends` / import 的预设，再逐个接入。在此之前，请勿依赖本目录。
