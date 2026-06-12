# 贡献指南

本仓库是前端 pnpm workspace + 后端 Cargo workspace。贡献时优先保证三件事：本地能跑、PR 标题规范、相关文档同步更新。

## 本地环境

```bash
git clone <repo-url>
cd haruhifanclub-website

nvm use
corepack enable
corepack prepare pnpm@10.11.0 --activate

pnpm install
cargo build

# 生成 .env，包含本地后端端口 127.0.0.1:17777 和开发密钥。
bash deploy/gen-secrets.sh
```

需要的系统工具：

| 工具    | 版本/要求           | 用途                        |
| ------- | ------------------- | --------------------------- |
| Node    | `>=20`              | 前端 dev/build/lint         |
| pnpm    | `10.11.0`           | workspace 包管理            |
| Rust    | `1.87.0`            | 后端构建、fmt、clippy、测试 |
| ffmpeg  | 可执行文件在 `PATH` | 音频转 MP3                  |
| sqlite3 | 可执行文件在 `PATH` | 备份、查库                  |

## 启动项目

```bash
# 后端。Vite 代理要求它监听 127.0.0.1:17777。
pnpm dev:backend

# 前端。访问地址要带子路径。
pnpm dev:news      # http://localhost:5204/news/
pnpm dev:art       # http://localhost:5201/art/
pnpm dev:exam      # http://localhost:5202/exam/
pnpm dev:novel     # http://localhost:5203/library/
pnpm dev:shop      # http://localhost:5205/shop/
pnpm dev:console   # http://localhost:5200/console/
```

也可以同时启动后端和一个前端：

```bash
APP=shop pnpm dev     # 默认 APP=news
```

## 项目结构

```text
apps/                  前端 app，各自独立子路径和 Vite 配置
packages/api-client/   共享前端包：fetch、JWT、RBAC、上传 URL
packages/ui/           预留目录，当前未启用
packages/config/       预留目录，当前未启用
backend/crates/        Rust crate：server/core/db/auth/media/ai/mail
backend/migrations/    SQLite schema SQL
deploy/                部署、备份脚本
docs/                  架构、部署、新模块、协作说明
```

`data/`、`uploads/`、`.env` 是运行时数据或密钥，不提交。

## 分支与 PR

1. 从 `main` 切分支：`git switch -c feat/novel-bookmark`。
2. 改动保持聚焦。跨前端、后端、部署的改动请在 PR 里写清影响范围。
3. 推送前跑本地自查：

   ```bash
   pnpm lint
   cargo test --workspace                # 含 server 的模块级集成回归网
   pnpm -r --if-present test             # 前端单测（vitest，如 news / api-client）
   pnpm --filter @haruhi/<app> build     # 改了前端 app 时
   ```

   后端 `backend/crates/server/tests/integration.rs` 是模块级特征化回归网（列表/分页/RBAC/上传校验），重构公共逻辑（如分页层）时应保持全绿。前端单测用 vitest，配置见各 app 的 `vitest.config.js`；改动巨型组件时建议先抽纯函数/composable 并补单测再动模板。

4. PR 标题使用 Conventional Commits。由于仓库采用 squash 合并，PR 标题会成为 `main` 上的最终提交信息。
5. CI 通过、评审完成后 squash 合并。

本仓没有 husky、lefthook、lint-staged。本地检查靠命令，最终由 CI 拦截。

## 提交规范

格式：

```text
type(scope): subject
```

`scope` 可选，建议使用 app、crate 或领域名，例如 `news`、`shop`、`server`、`api-client`、`deploy`、`docs`。

合法 `type`：

```text
feat fix perf refactor docs style test build ci chore revert
```

示例：

```text
feat(novel): 支持 EPUB 章节书签
fix(shop): 修正预售订单运费计算
docs(readme): 补充本地 .env 启动步骤
refactor(auth): 简化 RBAC 作用域判定
```

CI 只强制校验 PR 标题的 `type` 合法且 subject 非空。中间提交会被 squash，仍建议按同一格式写。

## CI

`.github/workflows/ci.yml` 使用路径过滤：

| job             | 触发范围                             | 内容                                                                                       |
| --------------- | ------------------------------------ | ------------------------------------------------------------------------------------------ |
| `frontend-lint` | 前端范围                             | `pnpm install`、`pnpm lint:js --quiet`，全仓 ESLint 只跑一次                                |
| `frontend`      | `apps/**`、`packages/**`、前端锁文件 | `pnpm install`、相关前端 app build                                                         |
| `frontend-test` | 前端范围                             | `pnpm -r --if-present test`，覆盖 `packages/api-client` 与已接入 vitest 的 app（如 `news`） |
| `backend`       | 后端范围，PR                         | `cargo fmt --all --check`、`cargo clippy --workspace`、`cargo test --workspace --lib`        |
| `backend-full`  | 后端范围，push main                  | `cargo clippy --workspace --all-targets`、`cargo test --workspace`                          |
| `ci-ok`         | 总是运行                             | 聚合 gate，分支保护只 require 这个 check                                                    |

路径过滤会让无关 job 正常 skipped。分支保护不要直接 require `frontend` 或 `backend`，只 require `ci-ok`。

`.github/workflows/audit.yml` 是手动触发的依赖审计，执行 `pnpm audit` 和 `cargo audit`，信息性检查，不参与 PR/push 门禁。

## 代码风格

前端：

- ESLint flat config 在根 `eslint.config.js`。
- Prettier 配置在 `.prettierrc.json`：无分号、单引号、`printWidth: 100`。
- `exam` 和 `console` 是 TypeScript，构建时会跑 `vue-tsc --noEmit`。

后端：

- 提交前跑 `cargo fmt --all` 或 `pnpm format`。
- `cargo clippy --workspace -- -D warnings` 要无 warning。
- SQL 使用 `sqlx::query` / `query_as`，不使用 `query!` 宏。
- 后台 handler 先做授权：`authorize(&state.pools.core, &user, "<module>", Action::X).await?`。仅超管端点用 `require_super`。

## 新增模块

详细步骤见 [docs/ADDING_MODULE.md](docs/ADDING_MODULE.md)。核心改动包括：

- `backend/crates/server/src/modules/<module>.rs`
- `backend/crates/server/src/modules/mod.rs`
- `backend/crates/db/src/lib.rs`
- `backend/migrations/<module>/0001_init.sql`
- `backend/crates/server/src/admin_routes.rs` 的 `APPS`
- `apps/<module>/`
- `deploy/nginx.conf` 和测试站 nginx 配置

`novel` 是当前最适合作为端到端参考的模块。

## 文档与安全

- 文档和注释使用中文，写事实和操作步骤，少写口号。
- 改 API、环境变量、部署脚本或数据库 schema 时同步更新 README 或 `docs/`。
- 安全问题请按 [SECURITY.md](SECURITY.md) 私下披露。
