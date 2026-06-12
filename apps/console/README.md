# @haruhi/console

超级管理员控制台。用于管理后台用户、启停账号、重置密码，并按 app/子作用域分配 RBAC 角色。

## 入口

- 子路径：`/console/`
- dev 端口：`5200`
- 后端接口：`/api/admin/*`、`/api/auth/*`
- 技术栈：Vue 3、TypeScript、Vite、`@haruhi/api-client`

## 本地运行

先在仓库根准备 `.env` 并启动后端：

```bash
bash deploy/gen-secrets.sh
pnpm dev:backend
```

再启动控制台：

```bash
pnpm --filter @haruhi/console dev
pnpm --filter @haruhi/console build
pnpm --filter @haruhi/console preview
```

访问 `http://localhost:5200/console/`。

## 目录

```text
index.html
vite.config.ts      base=/console/，代理 /api 和 /uploads 到 127.0.0.1:17777
tsconfig.json
env.d.ts
src/
  main.ts
  App.vue           登录、用户表格、角色分配都在这里
  api.ts            admin/auth API 封装
  router.ts
  style.css
```

## 权限模型

- 登录使用 `createAuth('/api')`。
- 进入控制台要求 `me.isSuperAdmin === true`。
- 后端所有 `/api/admin/*` 接口都调用 `require_super()`，前端判断只用于界面显示。
- 可分配 app 列表来自 `GET /api/admin/roles`，后端来源是 `admin_routes.rs` 的 `APPS` 常量。
- 角色为 `viewer`、`editor`、`moderator`、`admin`。

## 维护注意

- `build` 会先跑 `vue-tsc --noEmit`。
- 新增业务模块后，需要在后端 `APPS` 常量加入作用域，控制台才会显示。
- 涉及用户创建、删改、改密、角色调整时，后端会写 `audit_log`。
