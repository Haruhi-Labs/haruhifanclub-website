# @haruhi/api-client

春日应援团 monorepo 中**唯一启用的共享前端包**：把「统一 fetch 封装 + JWT 存取 + 后台 RBAC 判定（镜像后端 authorize）+ 上传资源 URL 解析」收敛到一处，被全部 6 个前端 app 依赖，避免各站各写一份鉴权与路径拼接逻辑。

## 技术栈与形态

- 纯原生 ESM JavaScript（`"type": "module"`），**零运行时依赖**，仅用浏览器原生 `fetch` / `localStorage` / `atob`。
- 手写 `index.d.ts` 提供类型声明，供 TS app（exam、console）直接 `import type` 使用。
- `package.json` 标 `private`、`workspace:*` 被消费，**不发布、无构建/测试脚本**——改动即生效，无需打包。

## 目录结构

整个包只有三个文件，无 `src/` 子目录：

```
packages/api-client/
├── index.js      # 全部实现（约 240 行）
├── index.d.ts    # 类型声明（手写，与 index.js 一一对应）
└── package.json  # 入口 main/module/types 均指向上述文件，exports 仅 "."
```

## 导出一览

| 导出 | 作用 |
| --- | --- |
| `getToken()` / `setToken(t)` / `clearToken()` | 读写/清除 `localStorage` 中的 admin JWT（键名 `haruhi_admin_token`），包了 try/catch 容错 |
| `createApiClient(base='/api')` | 创建模块客户端，返回 `{ base, get, post, put, patch, del, postForm }` |
| `createAuth(apiBase='/api')` | 通用鉴权助手：`login` / `me` / `logout` / `getToken` / `isLoggedIn`，走 `/api/auth/*` |
| `createAdminAuth(app, apiBase='/api')` | 后台管理员鉴权：登录并校验本模块权限、会话恢复、同步 token 校验、注入鉴权头 |
| `hasScope(user, scope)` | 前端细粒度作用域判定，镜像后端 `authorize` 的层级（持有该作用域或其父级 / 超管均放行） |
| `resolveUploadUrl(path, base='/uploads')` | 把后端返回的上传相对路径解析为可访问 URL |

## 关键特性与约定

- **统一请求语义**：`createApiClient` 的 `request` 自动注入 `Authorization: Bearer <token>`；非表单请求设 `Content-Type: application/json` 并序列化 body，`postForm` 传 `FormData`（由浏览器自动设 multipart 边界）。响应统一 `res.text()` 后尽量 `JSON.parse`。
- **统一错误**：非 2xx 时抛出带 `status` / `data` 的 `Error`，message 优先取后端约定的 `data.error`（对齐后端 `AppError→统一 JSON`）；**遇 401 自动 `clearToken()`**。
- **单点 JWT + 按 app 授权**：`createAdminAuth('news')` 把「登录校验本模块权限 / `restore` 会话恢复 / `hasValidToken` 同步校验 exp / 登出 / `buildHeaders` 注入头」收敛，各站不再各写 `hasXPerm`。`login` / `restore` **永不抛错**，分别返回 `{ ok, user?, error? }` 与 `user | null`。
- **本地解码不验签**：`hasValidToken` 只本地解码 JWT payload 校验 `exp`（供路由守卫快速判断），**真正验签由后端做**；payload 缺失或过期即 `clearToken()`。
- **权限层级（两个方向）**：`hasScope(user, 'news.activity')` 自下而上检查该作用域及其各级**父作用域**（超管/持有 `news.activity` 或父级 `news` 即放行）；`createAdminAuth` 内的 `hasPerm` 判断能否进入某 app 后台，自上而下匹配该 app 名及其任一**子作用域**前缀（超管/持有 `news` 或任一 `news.*` 即放行，故 `news.activity` 可进入 `news` 后台）。二者均与后端 RBAC 层级一致。
- **上传路径解析**（统一各 app 此前各写的 art `fixPath` / novel `${ASSET_BASE}/${path}`）：空值→`''`；`http`/`blob:`/`data:` 与站内绝对路径（`/` 开头）原样返回；否则去掉可能的前导 `uploads/` 再拼 `<base>/<rel>`，避免 `/uploads/uploads/...` 双前缀。

## 在 app 中使用

按 `workspace:*` 依赖，无需安装单独命令；直接具名导入：

```js
import { createApiClient, createAdminAuth, resolveUploadUrl } from '@haruhi/api-client'

const api = createApiClient('/api/news')   // 模块前缀
const auth = createAdminAuth('news')        // 后台鉴权（按 app 校验）

const { ok, user, error } = await auth.login(username, password)
const list = await api.get('/posts')
const cover = resolveUploadUrl(post.cover)  // → /uploads/news/...
```

本包自身无 dev/build/test 脚本，验证改动随消费它的 app 一起跑，例如：

```bash
pnpm --filter @haruhi/news dev          # news :5204 /news/
pnpm -r --filter "./apps/*" build       # 全量构建（含 exam/console 的 vue-tsc）
```

## 与共享层 / 后端的关系

- **消费方**：news、art、exam、novel、shop、console 六个 app 全部依赖本包（JS 与 TS app 通用，类型来自 `index.d.ts`）。`packages/` 下另有 `ui` / `config` 为空 stub，**未启用**——本包是当前唯一在用的共享前端包。
- **后端契约**：约定对接单一 Rust 后端 `haruhi-server`（`127.0.0.1:17777`）的统一 `/api/<module>/*` 与静态 `/uploads/<module>/*`；JWT、`/api/auth/*` 登录、`{ error }` 错误体、RBAC 层级均与后端 `auth` / `core` crate 对齐。

## 更多

- 仓库总览与架构：[`../../README.md`](../../README.md)
- 协作与提交规范（scope 含 `api-client`）：[`../../CONTRIBUTING.md`](../../CONTRIBUTING.md)
