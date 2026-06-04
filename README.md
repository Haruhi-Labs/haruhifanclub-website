# haruhifanclub

凉宫春日应援团统一站点 monorepo。把原先分散、各自为政的 **5 个旧子站**（新闻 / 画廊 / 考试 / 书库 / 商城）整合为
**一个 Rust 后端进程 + 多个独立前端 app** 的单仓库：后端统一鉴权、统一 API 前缀、按模块隔离数据，前端各 app 独立构建、分别挂在
`haruyuki.cn` 的不同子路径下。一套工程化与协作基建覆盖整仓，新增一个站点有端到端模板可循。

---

## 架构总览

单一 axum 二进制 `haruhi-server` 监听 `127.0.0.1:17777`，对外统一暴露 `/api/<module>/*` 接口与 `/uploads/<module>/*` 静态资源。
鉴权统一走 JWT + RBAC，每个业务模块各用一个独立的 SQLite 库（外加共享的 `core.db` 存用户 / 角色）。

```
前端（pnpm workspace）              后端（Cargo workspace，单一 axum 进程 :17777）
apps/*  → 各自子路径 + dev 端口      backend/crates/*  装配 /api/<module>/* 与 /uploads
packages/api-client 统一 fetch       data/<module>.db + data/core.db    uploads/<module>/
```

### 前端 apps

| App | 包名 | 子路径 | dev 端口 | 技术栈 / 关键点 | README |
| --- | --- | --- | --- | --- | --- |
| 新闻 / 学报 | `@haruhi/news` | `/news/` | 5204 | Vue3 + Pinia3 + 原生 CSS；feature 布局（博客·学报 / 积分商城 / 活动 / 奖品 / 积分） | [apps/news/README.md](apps/news/README.md) |
| 画廊 | `@haruhi/art` | `/art/` | 5201 | Vue3 + Pinia3 + CSS vars；画廊 / 评论 / 点赞 / 创作者 / 积分；AI 内容审核；匿名 Cookie 互动 | [apps/art/README.md](apps/art/README.md) |
| 考试 | `@haruhi/exam` | `/exam/` | 5202 | Vue3 + **TS** + SCSS；试卷编辑（edit_token）；图片 WebP / 音频转 MP3；qrcode 分享 | [apps/exam/README.md](apps/exam/README.md) |
| 书库 | `@haruhi/novel` | `/library/` | 5203 | Vue3 + **Tailwind**（无 Pinia）；epubjs 阅读器；EPUB 上传 + 封面 | [apps/novel/README.md](apps/novel/README.md) |
| 商城 | `@haruhi/shop` | `/shop/` | 5205 | Vue3 + 纯 reactive store（最复杂）；JWT / 订单 / 子订单 / 预售 / 优惠券 / 邮件队列 / 统计 / CSV 物流 | [apps/shop/README.md](apps/shop/README.md) |
| 超管台 | `@haruhi/console` | `/console/` | 5200 | Vue3 + **TS**；RBAC 超管台，走 `/api/admin/*`；用户 / 角色 / 按 app 授权 | [apps/console/README.md](apps/console/README.md) |

每个 app 的脚本都是标准的 `dev` / `build` / `preview`（`vite`）；`exam` 与 `console` 的 `build` 前置 `vue-tsc --noEmit` 做类型检查。

### 后端 crates（`backend/crates/<name>`，包名 `haruhi-<name>`）

| Crate | 职责 | README |
| --- | --- | --- |
| `server` | axum 二进制：装配 `/api/<module>/*` 路由、挂 `/uploads`、优雅停机（WAL checkpoint）、登录限流、CORS、readiness 探针 | [backend/crates/server/README.md](backend/crates/server/README.md) |
| `core` | `Config`（env 映射，release 下 fail-fast）、`AppError → 统一 JSON 响应`、`parse`（数值 / 文本解析工具） | [backend/crates/core/README.md](backend/crates/core/README.md) |
| `db` | sqlx `SqlitePool`：每模块一库 + `core.db`，WAL，迁移执行 | [backend/crates/db/README.md](backend/crates/db/README.md) |
| `auth` | JWT 签发 / 解码、argon2、RBAC `authorize` / `scope_chain` / `require_super`、axum 提取器（`AuthUser`） | [backend/crates/auth/README.md](backend/crates/auth/README.md) |
| `media` | 上传保存、base64 图片、image → WebP、EPUB 解析 | [backend/crates/media/README.md](backend/crates/media/README.md) |
| `ai` | DashScope（OpenAI 兼容）文本 / 图像审核；无 key 时 fail-open 放行 | [backend/crates/ai/README.md](backend/crates/ai/README.md) |
| `mail` | lettre（SMTP）+ Resend（reqwest）双驱动、邮件队列 worker（带重试） | [backend/crates/mail/README.md](backend/crates/mail/README.md) |

### 统一约束

- **统一 API 前缀**：`/api/<module>/*`，消灭旧站 `/blog-api`、`/art-api`、`/exam/api`、`/shop-api` 等多套命名。
- **统一鉴权**：`/api/auth/login` 拿 JWT；后台接口按 RBAC 鉴权，超管在 `/console/` 按应用为管理员分配角色（viewer / editor / moderator / admin）。
- **每模块独立 SQLite**：写锁隔离、迁移零转换；`core.db` 存用户 / 角色。
- **静态上传**：`/uploads/<module>/...`，生产由 Nginx 直接 alias。

---

## 快速上手

依赖：**Node ≥ 20**（`.nvmrc`）、**pnpm 10.11.0**、**Rust 1.87.0**（`rust-toolchain.toml`）、**ffmpeg**、**sqlite3**。

```bash
# 1) 安装依赖并构建后端
nvm use && pnpm install
cargo build

# 2) 准备环境变量（二选一）
bash deploy/gen-secrets.sh       # 推荐：openssl 生成强密钥，渲染填好的 .env（已存在则不覆盖）
cp deploy/env.sample .env        # 或手动复制模板，自行填 HARUHI_JWT_SECRET / 超管账号 等

# 3)（可选）迁移历史数据：把旧站 sqlite + uploads 拷进来
bash deploy/migrate-data.sh             # 全部模块
bash deploy/migrate-data.sh novel art   # 指定模块

# 4) 起后端（一个后端服务所有 app）
pnpm dev:backend                 # = cargo run -p haruhi-server，监听 127.0.0.1:17777

# 5) 起某个前端（dev 自动把 /api、/uploads 代理到后端）
pnpm dev:novel                   # 书库 → http://localhost:5203/library/
                                 # 其它：dev:news / dev:art / dev:exam / dev:shop / dev:console
```

> 想**一条命令同时起后端 + 某个前端**：`APP=novel pnpm dev`（concurrently 同开，默认 `APP=news`）。
> 各 app 的 dev 端口/子路径见上方「前端 apps」表。

---

## 构建与部署

**部署到生产（推荐，一条命令）**：`deploy/deploy.sh` 会构建前端 + **用 Docker 把后端交叉编译到
`linux/amd64`**（生产服务器架构）+ rsync 推送（原子替换二进制 + 备份旧版可回滚）+ 重启 systemd：

```bash
HARUHI_DEPLOY_HOST=root@<服务器> bash deploy/deploy.sh
```

> ⚠️ 本机 `cargo build --release` 出的是**你本机平台**的二进制（如 macOS/arm），**装不到 linux 服务器上**——
> 所以生产一律走 `deploy.sh` 的交叉编译。下面的原生构建只用于**本地运行 / 测试**。

只想本地产出物（不部署）：

```bash
pnpm build:apps                          # 各前端 → apps/*/dist
cargo build --release -p haruhi-server   # 后端二进制（仅本机平台，本地跑/测用）
```

部署架构：后端单二进制由 **systemd** 守护（`deploy/haruhifanclub.service`），**Nginx** 反代 `/api` + `/uploads`、
静态托管各 `apps/*/dist`（`deploy/nginx.conf`）；`data/` 与 `uploads/` 独立持久化并备份（`deploy/backup.*`），永不进 git。
完整步骤见 [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md)。

---

## 工程化与协作

提交遵循 **Conventional Commits**：**type 受约束**、**scope 可选且自由**（点明改动范围，写法见 CONTRIBUTING）。
因用 **squash 合并**，CI **只校验 PR 标题**（=最终提交信息），中间提交不校验——**不装本地 git hooks，规范靠 CI 把关**。
CI（`.github/workflows/`）用 `dorny/paths-filter` 做**路径过滤**，frontend / backend job 在无关改动时被跳过；
底部聚合 gate job **`ci-ok`**（`if: always()`）汇总各 job 结论，是 branch protection 唯一需要 require 的 status check。
另接入 **CodeRabbit**（`.coderabbit.yaml`）做 PR 评审、**Dependabot**（`.github/dependabot.yml`）跟进依赖。
贡献流程见 [CONTRIBUTING.md](CONTRIBUTING.md)。

---

## 共享层

- **`packages/api-client`（`@haruhi/api-client`）**：被全部 6 个 app 依赖的前端共享层。导出
  `getToken` / `setToken` / `clearToken`、`createApiClient(base)`（封装 `get/post/put/patch/del/postForm` 与 JSON / 表单 / 错误处理）、
  `createAuth`、`createAdminAuth(app)`、`hasScope`，以及统一的上传 URL 拼接 `resolveUploadUrl(path, base?)`（art / novel 去重后的共用实现）。
- **`core::parse`**：后端各模块共用的数值 / 文本解析工具。
- **`packages/ui`、`packages/config`**：预留的共享包，仅含一份说明性 `README.md`、无 `package.json`，故 pnpm 不纳入 workspace、当前**未启用**。

---

## 关键约定

- 后端 sqlx 使用**运行时校验**查询（非 `query!` 宏，免 `DATABASE_URL`）。
- 后台接口用 `haruhi_auth::authorize(&pools.core, &user, "<module>", Action::X)` 鉴权；超管路径用 `require_super`。
- **新增模块**：照 [docs/ADDING_MODULE.md](docs/ADDING_MODULE.md) 走，`novel` 是端到端复刻模板（后端 `router()` + `migrations/<m>/` + `db` 迁移登记 + 前端 `apps/<m>` 接入 `@haruhi/api-client`）。
- 文档、注释、README **一律中文**。

---

## 文档导航

- [CONTRIBUTING.md](CONTRIBUTING.md) — 贡献指南（分支 / 提交 / PR / 本地校验）
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — 整体架构与设计取舍
- [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md) — systemd + Nginx 部署、数据持久化与备份
- [docs/ADDING_MODULE.md](docs/ADDING_MODULE.md) — 新增一个模块 / 站点的端到端步骤
- 另见 [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)、[SECURITY.md](SECURITY.md)
