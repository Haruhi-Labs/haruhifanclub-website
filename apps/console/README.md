# @haruhi/console — 超级管理员控制台

春日应援团的 RBAC 超管台：登录后供**超级管理员**集中管理全站后台账号——新建管理员、按 app 分配角色、启停账号、重置密码、删除用户。后台接口统一走 `/api/admin/*`，由后端 `haruhi-auth` 的 `require_super` 鉴权。

部署在 `haruyuki.cn/console/`，dev 端口 `5200`。

## 技术栈

- Vue 3（`<script setup lang="ts">`，无 Pinia，无路由）+ TypeScript
- Vite 7 构建，`@vitejs/plugin-vue`
- `vue-tsc` 做构建期类型检查
- 唯一业务依赖 `@haruhi/api-client`（共享 HTTP 客户端 + 登录态）
- 原生 CSS（`src/style.css` 定义 CSS 变量主题，组件内 `scoped` 样式）

## 目录结构

整个 app 是一个极薄的单文件应用，无 router/store 分层：

```
index.html        # 挂载点，标题「春日应援团 · 控制台」
vite.config.ts    # base=/console/，dev 端口 5200，代理 /api、/uploads 到 17777
tsconfig.json     # strict，moduleResolution=Bundler，noEmit
env.d.ts          # vite/client 与 *.vue 模块声明
src/
  main.ts         # createApp(App).mount('#app')
  App.vue         # 全部逻辑与界面：登录、用户表格、角色分配
  style.css       # 全局主题（CSS 变量）
```

所有交互逻辑都在 `src/App.vue` 内，约 150 行脚本 + 一个表格视图。

## 本地开发

需先启动后端（`cargo run -p haruhi-server`，监听 `127.0.0.1:17777`）。

```bash
pnpm --filter @haruhi/console dev      # http://localhost:5200/console/
pnpm --filter @haruhi/console build    # vue-tsc --noEmit && vite build
pnpm --filter @haruhi/console preview   # 预览产物
```

- `dev`：纯 `vite`；`/api`、`/uploads` 经 `vite.config.ts` 代理到 17777。
- `build`：先 `vue-tsc --noEmit` 类型检查，再 `vite build`，产物 base 为 `/console/`。

## 关键特性与约定

- **登录与启动**：`createAuth('/api')` 提供 `login/me/logout/isLoggedIn`。挂载时若已登录则拉取 `me()`，失败即登出。
- **超管门禁**：以 `me.isSuperAdmin` 为唯一准入条件。
  - 未登录 → 登录卡片；
  - 已登录但非超管 → 提示「控制台仅限超级管理员访问」，并列出该用户被授权的 app 与角色名（取自 `me.apps[app].roleName`）；
  - 超管 → 进入用户管理。
- **数据加载**：超管登录后 `Promise.all` 请求 `GET /admin/users` 与 `GET /admin/roles`，拿到用户列表、可授权的 app 列表与可选角色（`{ key, name, level }`）。
- **用户管理操作**（均走 `/api/admin/*`）：
  - 新建管理员 `POST /admin/users`（用户名必填、密码 ≥ 6 位，前端校验）；
  - 改角色 `PUT /admin/users/:id/roles`，按 app 维度增删 `roles` 映射；
  - 启停 `PATCH /admin/users/:id`（`active` ⇄ `disabled`）；
  - 重置密码 `POST /admin/users/:id/password`；
  - 删除 `DELETE /admin/users/:id`。
- **角色矩阵 UI**：表格列为各 app，每格一个角色下拉；对超管行显示「全部」，`console` 这一列不可改（控制台权限不通过此处分配）。
- **app 中文名映射**：`appNames` 把 `news/art/exam/novel/shop/console` 及 `news` 的子模块（`news.blog`/`news.activity`/`news.store`/`news.points`）映射为中文展示名。
- **轻量反馈**：操作结果用 2.5s 自动消失的 toast（`flash()`）；改密/删除用浏览器原生 `prompt`/`confirm`。

## 与共享层 / 后端的关系

- 仅依赖共享包 `@haruhi/api-client`：`createApiClient('/api')` 发起带 JWT 的 `get/post/put/patch/del`，`createAuth('/api')` 管理登录态与当前用户；`CurrentUser`/`AppRole` 类型也由该包导出。
- 后端为单进程 `haruhi-server`，本 app 只调用其 `/api/admin/*` 管理接口，鉴权由 `haruhi-auth` 的超管校验（`require_super`）兜底——前端的超管门禁仅作体验层，真正的权限边界在后端。

## 更多

- 仓库总览与架构：[根 README](../../README.md)
- 协作与提交规范（scope 用 `console`）：[CONTRIBUTING](../../CONTRIBUTING.md)、[docs/COLLABORATION.md](../../docs/COLLABORATION.md)
- 新增后台模块流程：`docs/ADDING_MODULE.md`（`novel` 为端到端模板）
