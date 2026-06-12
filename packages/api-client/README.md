# @haruhi/api-client

共享前端包。前端 app 通过它复用请求封装、JWT 存取、后台权限判断和上传 URL 解析。

## 形态

- 原生 ESM JavaScript。
- 零运行时依赖。
- 手写 `index.d.ts` 供 TypeScript app 使用。
- 不发布 npm，使用 `workspace:*` 依赖。
- 包内没有构建步骤，改 `index.js` 后由消费方 app 直接使用。

```text
packages/api-client/
  index.js
  index.d.ts
  index.test.js
  package.json
```

## 导出

| API                                         | 说明                                     |
| ------------------------------------------- | ---------------------------------------- |
| `getToken()`                                | 从 `localStorage` 读取 admin JWT         |
| `setToken(token)`                           | 写入 admin JWT                           |
| `clearToken()`                              | 清除 admin JWT                           |
| `createApiClient(base = '/api')`            | 返回 `get/post/put/patch/del/postForm`   |
| `createAuth(apiBase = '/api')`              | 登录、`me()`、登出、登录态判断           |
| `createAdminAuth(app, apiBase = '/api')`    | 模块后台登录、恢复会话、权限检查、鉴权头 |
| `hasScope(user, scope)`                     | 判断用户是否拥有作用域或父作用域权限     |
| `resolveUploadUrl(path, base = '/uploads')` | 把上传路径转成可访问 URL                 |

## 请求行为

- 自动带 `Authorization: Bearer <token>`。
- JSON 请求自动设置 `Content-Type: application/json` 并序列化 body。
- `postForm` 接收 `FormData`，由浏览器设置 multipart boundary。
- 响应先读 `text()`，再尝试 `JSON.parse`。
- 非 2xx 抛出 `Error`，附带 `status` 和 `data`。
- 401 会自动 `clearToken()`。

## 权限行为

`hasScope(user, 'news.activity')` 会检查：

```text
news.activity
news
```

用户持有任一父作用域即通过；超管直接通过。该规则与后端 `haruhi-auth::scope_chain()` 保持一致。

`createAdminAuth(app)` 的 `hasPerm()` 用于判断能否进入某 app 后台：超管、持有该 app 角色、或持有其子作用域角色都可进入。

前端权限只控制显示和跳转。后端 handler 仍必须调用 `authorize()` 或 `require_super()`。

## 上传 URL

`resolveUploadUrl(path, base)` 的规则：

- 空值返回空字符串。
- `http`、`blob:`、`data:` 原样返回。
- `/` 开头的站内绝对路径原样返回。
- 其它路径会去掉可能的前导 `uploads/`，再拼到 `base` 下。

示例：

```js
resolveUploadUrl('art/a.webp') // /uploads/art/a.webp
resolveUploadUrl('uploads/art/a.webp') // /uploads/art/a.webp
resolveUploadUrl('/uploads/art/a.webp') // /uploads/art/a.webp
```

## 使用

```js
import { createApiClient, createAdminAuth, resolveUploadUrl } from '@haruhi/api-client'

const api = createApiClient('/api/news')
const auth = createAdminAuth('news')

const login = await auth.login(username, password)
if (login.ok) {
  const posts = await api.get('/posts')
  const cover = resolveUploadUrl(posts[0].cover)
}
```

## 验证

```bash
pnpm --filter @haruhi/api-client test
pnpm --filter @haruhi/news build
pnpm --filter @haruhi/console build
```

改类型声明时，至少构建一个 TypeScript app，例如 `console` 或 `exam`。
