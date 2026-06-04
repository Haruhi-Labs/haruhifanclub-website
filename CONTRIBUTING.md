# 贡献指南

欢迎为 **haruhifanclub**（凉宫春日应援团统一站点）贡献代码与文档！本仓库是一个 monorepo：
**前端 pnpm workspace + 后端 Cargo workspace**，单一 Rust 后端进程（`haruhi-server`）服务全部模块，
各前端 app 挂在 `haruyuki.cn` 的不同子路径下。

本指南帮助你从零跑起本地环境、按规范提交、走通 PR 流程并最终合并。**全仓提交、注释、文档一律用中文**
（项目硬性约定）。如果你只想改文档或修个小 bug，直接看 [快速上手](#快速上手) 与 [提交规范](#提交规范) 即可。

> 配套文档：
> - 架构总览 → [`README.md`](README.md)、[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)
> - 新增业务模块 → [`docs/ADDING_MODULE.md`](docs/ADDING_MODULE.md)
> - 部署上线 → [`docs/DEPLOYMENT.md`](docs/DEPLOYMENT.md)
> - 行为准则 → [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md)
> - 安全披露 → [`SECURITY.md`](SECURITY.md)

---

## 目录

- [快速上手](#快速上手)
- [项目结构速览](#项目结构速览)
- [环境准备](#环境准备)
- [本地开发](#本地开发)
- [分支与 PR 流程](#分支与-pr-流程)
- [提交规范（Conventional Commits）](#提交规范conventional-commits)
- [CI 关卡说明](#ci-关卡说明)
- [代码风格](#代码风格)
- [新增模块指引](#新增模块指引)
- [行为准则与安全](#行为准则与安全)

---

## 快速上手

```bash
# 1. 克隆并进入仓库
git clone <仓库地址> && cd haruhifanclub

# 2. 切到本仓约定的 Node 版本（见 .nvmrc，要求 Node >= 20）
nvm use            # 没装 nvm 的话，自行装好 Node 20

# 3. 安装前端依赖 + 编译后端
pnpm install
cargo build

# 4. 从 main 切一个 feature 分支
git switch -c feat/novel-bookmark

# 5. 改完后，本地自查（仓库不装本地 hooks，规范全靠你自觉 + CI 把关）
pnpm lint          # eslint + cargo fmt --check + cargo clippy -D warnings
cargo test --workspace

# 6. 按 Conventional Commits 提交，开 PR，等 CI 全绿 + 评审通过 → squash 合并
git commit -m "feat(novel): 支持 EPUB 章节书签"
```

---

## 项目结构速览

```
haruhifanclub/
├─ apps/                  前端 app（pnpm workspace，每个独立构建、独立子路径）
│  ├─ news/               @haruhi/news     → /news/
│  ├─ art/                @haruhi/art      → /art/
│  ├─ exam/               @haruhi/exam     → /exam/      (TypeScript)
│  ├─ novel/              @haruhi/novel    → /library/
│  ├─ shop/               @haruhi/shop     → /shop/
│  └─ console/            @haruhi/console  → /console/   (TypeScript, RBAC 超管台)
├─ packages/
│  ├─ api-client/         @haruhi/api-client  统一 fetch 客户端，被全部 6 个 app 依赖
│  ├─ ui/                 空 stub（未启用）
│  └─ config/             空 stub（未启用）
├─ backend/
│  ├─ crates/             后端 Cargo workspace（包名 haruhi-<name>）
│  │  ├─ server/          装配路由，单二进制 haruhi-server，监听 127.0.0.1:17777
│  │  ├─ core/            配置 / 错误 / 响应
│  │  ├─ db/              sqlite 连接池(WAL) + 迁移
│  │  ├─ auth/            JWT + argon2 + RBAC
│  │  ├─ media/           上传 / WebP / EPUB
│  │  ├─ ai/              DashScope 内容审核
│  │  └─ mail/            SMTP/Resend 邮件队列
│  └─ migrations/         各模块 SQL 迁移（migrations/<module>/）
├─ deploy/                systemd / Nginx / 数据迁移脚本
├─ docs/                  架构、部署、新增模块、协作文档
├─ data/                  运行时 SQLite（data/<module>.db + data/core.db）—— 不进 git
└─ uploads/               运行时上传文件（uploads/<module>/...）—— 不进 git
```

> **`data/` 与 `uploads/` 永不进 git**（见 `.gitignore`），它们是运行时产物，独立持久化与备份。
> `.env`（含密钥）同样被忽略，请勿提交。`packages/ui`、`packages/config` 目前是空 stub，尚未启用。

后端是**单进程多模块**：`backend/crates/server` 把每个模块装配到统一前缀 `/api/<module>/*`，
各模块用**独立 SQLite 库**（写锁隔离），用户/角色集中在 `core.db`。详见
[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)。

---

## 环境准备

| 工具 | 版本要求 | 用途 |
| --- | --- | --- |
| Node | **>= 20**（见 `.nvmrc`） | 前端构建 / lint |
| pnpm | **10.11.0**（见 `package.json` 的 `packageManager`） | 前端包管理 / workspace |
| Rust | **1.87.0**（见 `rust-toolchain.toml`，含 rustfmt + clippy） | 后端编译 / lint / 测试 |
| ffmpeg | 任意近期版本 | media 模块处理视频 / 转码 |
| sqlite3 | 任意近期版本 | 本地查库、数据迁移脚本 |

安装步骤：

```bash
# Node：推荐用 nvm，自动读取 .nvmrc
nvm install      # 首次：按 .nvmrc 装 Node 20
nvm use          # 切到 Node 20

# pnpm：随 Node 用 corepack 即可（或 npm i -g pnpm@10.11.0）
corepack enable
corepack prepare pnpm@10.11.0 --activate

# Rust：用 rustup，工具链由 rust-toolchain.toml 锁定
# rustup show 会自动按 rust-toolchain.toml 装好 1.87.0 + rustfmt + clippy

# 系统工具（macOS 示例，Linux 用对应包管理器）
brew install ffmpeg sqlite3
```

一键安装依赖并编译：

```bash
nvm use && pnpm install && cargo build
```

---

## 本地开发

### 起后端（必须先起，前端 dev 会代理到它）

```bash
cargo run -p haruhi-server      # 监听 127.0.0.1:17777
```

首次启动会自动跑数据库迁移并在 `data/` 下建库。如需准备环境变量与种子超管账号，参考
`deploy/env.sample`（可 `cp deploy/env.sample .env` 后填 `HARUHI_JWT_SECRET` 等，或直接 `export`），
详见 [`docs/DEPLOYMENT.md`](docs/DEPLOYMENT.md)。

### 起某个前端 app

前端 dev server 已配置把 `/api` 与 `/uploads` 代理到后端 `127.0.0.1:17777`，所以**先起后端再起前端**：

```bash
pnpm --filter @haruhi/novel dev      # 例：书库
# 或用根 package.json 的快捷脚本：
pnpm dev:novel
```

各 app 的包名、dev 端口与部署子路径：

| app | 包名 | dev 端口 | 部署子路径 | 语言 |
| --- | --- | --- | --- | --- |
| console | `@haruhi/console` | `5200` | `/console/` | TypeScript（RBAC 超管台） |
| art | `@haruhi/art` | `5201` | `/art/` | JS / Vue |
| exam | `@haruhi/exam` | `5202` | `/exam/` | TypeScript |
| novel | `@haruhi/novel` | `5203` | `/library/` | JS / Vue |
| news | `@haruhi/news` | `5204` | `/news/` | JS / Vue |
| shop | `@haruhi/shop` | `5205` | `/shop/` | JS / Vue |

> dev 访问地址带子路径，例如 `http://localhost:5203/library/`、`http://localhost:5200/console/`。

### 同时起前后端（可选）

```bash
APP=novel pnpm dev      # concurrently 同时起 cargo run + 指定 app 的 dev（默认 APP=news）
```

### 构建

```bash
pnpm -r --filter "./apps/*" build        # 各前端 → apps/*/dist
cargo build --release -p haruhi-server   # 后端单二进制 → target/release/haruhi-server
```

> `exam` 与 `console` 是 TypeScript app，其 `build` 内含 `vue-tsc --noEmit` 类型检查——
> 改这两个 app 的类型相关代码后，务必本地 build 一遍。

---

## 分支与 PR 流程

本仓托管在 **GitHub**，采用 PR + CI 把关 + CodeRabbit 自动评审 + 人工评审的协作模式。

1. **从 `main` 切 feature 分支**（不要直接推 `main`，`main` 受保护）：

   ```bash
   git switch -c feat/<scope>-<简述>     # 例：feat/novel-bookmark
   ```

   分支名建议 `<type>/<scope>-<简述>`，与提交类型/作用域呼应，便于辨认。

2. **本地自查**（**本仓不装任何本地 hooks**——没有 husky / lefthook / lint-staged，
   规范完全靠 CI 把关，所以请在推之前**自觉**先跑）：

   ```bash
   pnpm lint                 # eslint + cargo fmt --check + cargo clippy -D warnings
   cargo test --workspace    # 后端测试
   # 改了前端就再 build 一下相关 app（exam/console 会顺带 vue-tsc 类型检查）
   pnpm --filter @haruhi/<app> build
   ```

3. **按 [提交规范](#提交规范conventional-commits) 提交**。commit 信息、PR 标题都要符合 Conventional Commits。

4. **开 PR**：目标分支 `main`。
   - PR 标题**必须**符合 Conventional Commits（squash 合并时它就是最终提交信息）。
   - 描述清楚改动动机、影响范围、验证方式；涉及 UI 的附截图。
   - 关联 issue 用 `Closes #123`。

5. **等待关卡通过**：
   - **CI 全绿**（见 [CI 关卡说明](#ci-关卡说明)）——尤其是聚合 gate `ci-ok`。
   - **CodeRabbit 自动评审**给出意见，按需回应/修改。
   - **至少一名 reviewer** 批准（CODEOWNERS 会自动请求评审）。

6. **squash 合并**：本仓采用 **squash merge**，保持 `main` 线性历史。
   合并后该 feature 分支即可删除。

> 因为采用 squash 合并，**PR 标题即最终落到 `main` 的提交信息**——请认真写 PR 标题。
> 同时 PR 内的每个 commit 也会被 commitlint 逐条检查（见下），所以中途提交也请遵守规范。

---

## 提交规范（Conventional Commits）

全仓统一使用 **Conventional Commits**。**type 是唯一受约束项**（必须取自下方集合）；
**scope 可选、不限定取值**——用它点明本次改动的范围即可。

> 历史提交**不**符合该规范；规范**从现在起向前生效**。

### 格式

```
type(scope): subject
```

- **subject 允许中文**、不限大小写、不强制句号。
- `scope` **可选、不限定取值**：用它点明本次改动影响的范围（app / crate / 跨领域区域），见下方建议。
- header（首行）放宽到 **100** 字符；body / footer 行长不限。

### type（唯一合法集合）

```
feat fix perf refactor docs style test build ci chore revert
```

| type | 含义 |
| --- | --- |
| `feat` | 新功能 |
| `fix` | 修 bug |
| `perf` | 性能优化 |
| `refactor` | 重构（不改外部行为） |
| `docs` | 文档 |
| `style` | 格式 / 空白 / 不影响逻辑 |
| `test` | 测试 |
| `build` | 构建系统 / 依赖产物 |
| `ci` | CI 配置与脚本 |
| `chore` | 杂项（不属于上面任何一类） |
| `revert` | 回滚某次提交 |

### scope（可选、自由：点明修改范围）

scope **不强制、也不在代码里维护封闭集合**——请用最贴切的简短小写名点明这次动了哪块，让人扫一眼就知道范围。常用建议（**仅供参考、非限定**）：

- 前端 app：`news` `art` `exam` `novel` `shop` `console`
- 共享前端：`api-client`
- 后端 crate：`server` `core` `db` `auth` `media` `ai` `mail`
- 跨领域：`deploy` `ci` `docs` `deps` `repo`

跨多个范围或不好归类时，省略 scope 也可以（`feat: 一句话`）。

### 正例（照着写）

```text
feat(novel): 支持 EPUB 章节书签
fix(shop): 修正预售拆单运费重复计算
ci(repo): 路径过滤拆分前后端流水线
docs(api-client): 补充 createAdminAuth 用法示例
refactor(auth): 抽出 scope_chain 父级继承判定
chore(deps): 升级 vite 到 7.1.x
```

### 反例（会被 CI 拦下）

```text
更新了一些东西                  # 缺 type
feature(novel): 书签            # type 必须是 feat，不是 feature
改了点东西                      # 完全没有 type
```

> CI 只校验 **type 合法 + subject 非空**；**scope 不校验取值**，自由发挥但请写得贴切。
> 小贴士：scope 用**作用域名**（如 `novel`）比部署子路径（`/library/`）更直观。

---

## CI 关卡说明

> 注意：GitHub 侧自动化（CI / CodeRabbit）会在 PR 上自动运行；下列关卡最终以仓库
> `.github/workflows/` 与分支保护设置为准。

PR 会触发以下关卡（最终以仓库 `.github/workflows/` 与分支保护设置为准）：

| 关卡 | 作用 |
| --- | --- |
| **pr-checks** | ① PR 标题语义检查（`amannn/action-semantic-pull-request`，校验 **type**；scope 不限）；② 逐提交 commitlint（`wagoid/commitlint-github-action`，对中文友好：关闭 subject-case / subject-full-stop，header 放宽到 100） |
| **ci**（路径过滤 + 分层） | `dorny/paths-filter` 判断改了前端还是后端。前端：`pnpm install` + 各 app build（exam/console 含 `vue-tsc`）。后端**分两层**——**PR**（求快）：`cargo fmt --check` + `clippy`(lib/bins) + `cargo test --lib`（单测）；**合入 main**：`clippy --all-targets` + 完整 `cargo test`（含 `tests/` 集成）。后端**不再跑** `cargo build --release`（部署时用 Docker 交叉编译） |
| **audit** | 依赖漏洞审计（`pnpm audit` + `cargo audit`），**信息性**、不阻断构建；较慢，**仅合入 main 后**跑，不拖慢 PR |
| **ci-ok**（聚合 gate） | 用 `always()` 汇总上述各 job 的结论，是分支保护里**唯一的 required status check**。这样路径过滤导致某 job 被 skip 时也不会卡住 PR |
| **CodeRabbit** | PR 自动代码评审（`.coderabbit.yaml` 已就绪，需在 coderabbit.ai 授权 GitHub App） |

> **为什么 required check 只设 `ci-ok`**：路径过滤会让"只改前端"的 PR 跳过后端 job（反之亦然）。
> 被 skip 的 job 永不上报状态，若把它设为 required 会让 PR 永久卡在 pending。`ci-ok` 用 `always()`
> 运行并聚合所有上游 job 的结论，是唯一稳妥的 required check。

---

## 代码风格

### 前端

- **ESLint**（flat config，根 `eslint.config.js`，覆盖 `apps/*` 与 `packages/*`）+ **Prettier**
  （根 `.prettierrc.json`：无分号、单引号、printWidth 100、es5 trailing comma）。
- 本地：`pnpm lint:js`（仅 eslint）、`pnpm format`（prettier 写入 + cargo fmt）。
- `exam` / `console` 为 TypeScript，build 时 `vue-tsc --noEmit` 做类型检查——别引入类型错误。

### 后端

- **`cargo fmt`** 格式化（CI 跑 `cargo fmt --all --check`，不通过即失败）。
- **`cargo clippy -- -D warnings`**：**warning 即错误**，提交前务必本地清零。
- **sqlx 运行时校验查询**：用 `sqlx::query` / `query_as`（**不要** `query!` 宏），免 `DATABASE_URL`、
  免离线缓存。详见 [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) 的 sqlx 约定。
- 后台接口第一行做 RBAC：`haruhi_auth::authorize(&state.pools.core, &user, "<module>", Action::X)`；
  仅超管的端点用 `require_super`。

一条命令跑全部 lint（与 CI 一致）：

```bash
pnpm lint     # = eslint . && cargo fmt --check && cargo clippy -- -D warnings
```

---

## 新增模块指引

要加一个新业务模块（如 `widget`），完整步骤见 **[`docs/ADDING_MODULE.md`](docs/ADDING_MODULE.md)**，
以 `novel` 为端到端模板。简述要改/新增的点：

- **后端**：`backend/crates/server/src/modules/<m>.rs`（`router()` + handler + `authorize`）；
  在 `modules/mod.rs` 里 `mod <m>;` 并 `nest("/<m>", ...)`；新增 `backend/migrations/<m>/0001_init.sql`；
  在 `backend/crates/db/src/lib.rs` 的 `Pools` 加字段 + `connect()` + `migrate()`。
- **前端**：新建 `apps/<m>/`，`package.json` 依赖 `@haruhi/api-client`（`workspace:*`），
  `vite.config.ts` 配 `base: '/<m>/'` 与 `/api`、`/uploads` 代理；接入 `createApiClient` / `createAdminAuth`。
- **RBAC**：在 `admin_routes.rs` 的 `APPS` 常量加该作用域，让超管能在 `/console/` 分配角色。
- **部署**：`deploy/nginx.conf` 加一条 `/<m>/` location；如需迁旧数据改迁移脚本。

`apps/*` 已在 `pnpm-workspace.yaml` 通配，新建目录后 `pnpm install` 即纳入 workspace。

---

## 行为准则与安全

- 参与本项目即表示你同意遵守 [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md)（行为准则）。
- 发现安全漏洞**请勿公开提 issue**，按 [`SECURITY.md`](SECURITY.md) 的流程**私下负责任披露**
  （后端涉及 JWT / RBAC / 文件上传，敏感问题尤其要走私下渠道）。

感谢你的贡献，SOS 团欢迎你！
