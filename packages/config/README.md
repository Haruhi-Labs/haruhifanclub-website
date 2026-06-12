# packages/config

预留目录，当前未启用。

现状：

- 没有 `package.json`。
- 不在 pnpm workspace 中作为可 import 包使用。
- 没有导出的 Vite、ESLint、TypeScript 或 Prettier 预设。

当前配置分布：

- ESLint：根 `eslint.config.js`
- Prettier：根 `.prettierrc.json`
- Node：根 `.nvmrc`
- Rust：根 `rust-toolchain.toml`
- Vite：各 app 自己的 `vite.config.*`
- TypeScript：`apps/exam`、`apps/console` 各自维护

只有当多个 app 出现需要共同维护的构建或 lint 预设时，再为本目录补 `package.json` 和导出入口。
