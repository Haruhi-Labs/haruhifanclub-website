# 协作与仓库启用运行手册

本仓库当前是**本地 git 仓库，尚无 GitHub remote**，因此所有 GitHub 侧自动化（Actions、CODEOWNERS、
branch protection、CodeRabbit、Dependabot、Release）在你 **push 到 GitHub 并完成下方配置之前不会运行**。
仓库内的相关文件均已"就绪即用"，本文档是把它们**真正启用**的逐步运行手册。

> 配套：贡献流程见 [`../CONTRIBUTING.md`](../CONTRIBUTING.md)；安全披露见 [`../SECURITY.md`](../SECURITY.md)；
> 行为准则见 [`../CODE_OF_CONDUCT.md`](../CODE_OF_CONDUCT.md)。
>
> **占位提示**：本仓多处用 `@OWNER` 作为占位（CODEOWNERS、CODE_OF_CONDUCT、SECURITY 等），
> 启用时务必全部替换为真实 GitHub 用户名 / 组织名，详见 [步骤 6](#6-替换所有-owner-占位)。

---

## 前置检查

```bash
# 确认当前在 main、且没有 remote
git branch                 # 应显示 * main
git remote -v              # 应为空（尚无 origin）
```

确认本机已装 `git` 与 [`gh`（GitHub CLI）](https://cli.github.com/)（用 `gh` 建仓最省事，可选）。

---

## 1. 在 GitHub 建仓并推送

**方式 A：用 GitHub CLI（推荐）**

```bash
# 在仓库根执行；<你的账号> 替换为真实 GitHub 用户名/组织
gh repo create <你的账号>/haruhifanclub --private --source=. --remote=origin
git push -u origin main
```

**方式 B：先在 GitHub 网页建空仓，再手动接 remote**

```bash
# 在 GitHub 上建一个空仓库 haruhifanclub（不要勾选 README/.gitignore/license，避免冲突）
git remote add origin git@github.com:<你的账号>/haruhifanclub.git
git push -u origin main
```

> 推送前再确认一遍：`data/`、`uploads/`、`.env` 等已被 `.gitignore` 忽略，不会被推上去。

---

## 2. 设默认分支 + 开启分支保护

确保默认分支是 `main`，并对其开启保护规则。

**网页操作**：仓库 → Settings → Branches → Add branch ruleset / protection rule，目标 `main`，勾选：

- ✅ **Require a pull request before merging**（合并必须走 PR）
  - ✅ Require approvals：**至少 1 个** reviewer
  - ✅ Require review from Code Owners（启用 CODEOWNERS 自动请求评审）
- ✅ **Require status checks to pass before merging**
  - ✅ Require branches to be up to date before merging
  - **Required status check 只选 `ci-ok` 这一个**（聚合 gate）
- ✅ **Require linear history**（配合 squash 合并，保持线性历史）
- （可选）✅ Require conversation resolution before merging

> **关键坑（务必照做）**：本仓 CI 用路径过滤（`dorny/paths-filter`）拆分前后端流水线，"只改前端"的 PR 会
> **skip 后端 job**（反之亦然）。被 skip 的 job 永不上报状态——若把它设为 required check，PR 会永久卡在
> pending。因此 **required check 有且只有 `ci-ok`**：它用 `always()` 运行并聚合所有上游 job 的结论。

**用 gh CLI 设 squash-only 合并（可选但推荐）**：

```bash
gh repo edit <你的账号>/haruhifanclub \
  --enable-squash-merge \
  --enable-merge-commit=false \
  --enable-rebase-merge=false \
  --delete-branch-on-merge
```

> 也可在 Settings → General → Pull Requests 里勾选 "Allow squash merging" 并取消其余两项、
> 勾选 "Automatically delete head branches"。

---

## 3. 授权 CodeRabbit 自动评审

`.coderabbit.yaml` 已就绪，只需把 CodeRabbit GitHub App 授权到本仓：

1. 打开 [coderabbit.ai](https://coderabbit.ai)，用 GitHub 账号登录。
2. 安装 **CodeRabbit GitHub App**，在仓库选择里授权到 `haruhifanclub`（可只授权单仓）。
3. 之后**新开的 PR** 会自动触发 CodeRabbit 评审；按其意见回应/修改即可。

> 评审规则以仓库根的 `.coderabbit.yaml` 为准，无需在网页端重复配置。

---

## 4. 启用 Dependabot

让 Dependabot 自动给 npm（pnpm）与 Cargo 依赖、以及 GitHub Actions 开升级 PR。

**网页操作**：仓库 → Settings → Code security and analysis：

- ✅ Dependabot alerts
- ✅ Dependabot security updates
- （如仓库提供 `.github/dependabot.yml`）✅ Dependabot version updates

> 同时建议开启 Settings → Code security → **Private vulnerability reporting**，
> 与 [`../SECURITY.md`](../SECURITY.md) 的私下披露流程配套。

### 4.1 先创建标签（labeler / Dependabot 会用到）

`.github/labeler.yml`（PR 自动打标签）、Dependabot 与 issue 模板都会给 issue/PR 贴标签，但 **GitHub 不会自动创建不存在的标签**——标签缺失时打标签会静默失败。启用前先在 **Settings → Labels** 建好（或用 `gh` 批量建）：

```bash
# 路径标签（与 .github/labeler.yml 对应）
for l in app:news app:art app:exam app:novel app:shop app:console api-client backend ci docs deps; do
  gh label create "$l" --force >/dev/null 2>&1 || true
done
# issue 模板与 Dependabot 默认用到的标签
for l in bug enhancement dependencies; do gh label create "$l" --force >/dev/null 2>&1 || true; done
```

---

## 5. 发布首个版本（触发 release）

发布走**轻量**模式：[git-cliff](https://github.com/orhun/git-cliff) 从 Conventional Commits 生成
`CHANGELOG.md`，打 `v*` tag 触发 `release` workflow 建 GitHub Release。**不发 npm、不做版本号联动。**

```bash
# 确保本地 main 与远端同步、且 CI 全绿
git switch main && git pull --ff-only

# 打首个版本 tag 并推送 → 触发 release workflow
git tag v0.1.0
git push origin v0.1.0
```

推送 tag 后，到仓库的 **Actions** 标签确认 release workflow 跑通，并在 **Releases** 看到
`v0.1.0` 及自动生成的 CHANGELOG 条目。

> **关于回写 `CHANGELOG.md` 到 main**：`release.yml` 末尾有一个**可选**步骤，把全量 CHANGELOG 提交回 `main`。
> 若你按 [步骤 2](#2-设默认分支--开启分支保护) 对 `main` 开了「合并必须走 PR」，`github-actions[bot]` 的直推会被拒，
> 导致该步失败——但 **Release 与 tag 已成功创建，不受影响**。两种处理方式：① 把这步当可选，失败即忽略，需要时本地
> 跑 `git cliff -o CHANGELOG.md` 再开 PR；② 在分支保护里放行 `github-actions[bot]`（或用具备绕过权限的 token）。

---

## 6. 替换所有 `@OWNER` 占位

仓库里使用了占位 `@OWNER`，启用时必须全部替换为真实 GitHub 用户名 / 组织名。先列出所有出现位置：

```bash
# 在仓库根执行，列出还残留 OWNER 占位的文件。
# 注意用 @?OWNER 同时匹配「@OWNER」与「裸 OWNER」——后者出现在
# .github/ISSUE_TEMPLATE/config.yml 的 github.com/OWNER/... 链接里，单匹配 "@OWNER" 会漏掉。
grep -rnE "@?OWNER" . \
  --include="*.md" \
  --include="*.yml" \
  --include="*.yaml" \
  --include="CODEOWNERS" \
  --exclude-dir=node_modules \
  --exclude-dir=target \
  --exclude-dir=target-linux
```

至少检查以下文件并替换：

- `.github/CODEOWNERS`（若存在）—— 改成真实负责人，CODEOWNERS 才能在 PR 自动请求评审。
- [`../CODE_OF_CONDUCT.md`](../CODE_OF_CONDUCT.md) —— 举报联系人。
- [`../SECURITY.md`](../SECURITY.md) —— 安全披露联系人。
- `.github/ISSUE_TEMPLATE/config.yml` —— 文档 / 讨论区链接里的 `github.com/OWNER/...`（**裸 OWNER**）。
- 本文档及其它文档内的 `@OWNER` 引用。

替换后提交（遵守 Conventional Commits）：

```bash
git switch -c chore/replace-owner-placeholder
# 编辑替换 @OWNER ……
git commit -am "chore(repo): 替换 @OWNER 占位为真实维护者账号"
# 开 PR → CI 全绿 + 评审 → squash 合并
```

---

## 启用后核对清单

- [ ] `git remote -v` 有 `origin`，`main` 已推送到 GitHub
- [ ] 默认分支为 `main`，分支保护已开，**required status check = `ci-ok`（仅此一个）**
- [ ] 合并方式限定为 **squash**，开启 "合并后删除分支" 与 "线性历史"
- [ ] PR 评审要求：至少 1 名 reviewer + Require review from Code Owners
- [ ] 已创建 labeler / Dependabot / issue 模板用到的标签（app:* / backend / ci / docs / deps / bug / enhancement / dependencies）
- [ ] CodeRabbit GitHub App 已授权到本仓，新 PR 能触发自动评审
- [ ] Dependabot alerts / security updates 已开；Private vulnerability reporting 已开
- [ ] 打 `v0.1.0` tag 能触发 release，Releases 里有自动生成的 CHANGELOG
- [ ] 全仓 `@OWNER` 占位已替换为真实账号（`grep -rn "@OWNER"` 无残留）

---

相关文档：[贡献指南](../CONTRIBUTING.md) · [新增模块](ADDING_MODULE.md) · [架构](ARCHITECTURE.md) · [部署](DEPLOYMENT.md)
