# @haruhi/news

内容主站和门户 app，包含团内新闻（春日团报）、团员手册、活动、积分商城、入坑测试和后台管理。

## 入口

- 子路径：`/news/`
- dev 端口：`5204`
- 后端接口：`/api/news/*`
- 上传资源：`/uploads/news/*`
- 技术栈：Vue 3、Vue Router、Pinia、Vite、原生 CSS、`@haruhi/api-client`

## 本地运行

```bash
bash deploy/gen-secrets.sh
pnpm dev:backend

pnpm --filter @haruhi/news dev
pnpm --filter @haruhi/news build
pnpm --filter @haruhi/news preview
pnpm --filter @haruhi/news test     # vitest 单元/组件测试
```

访问 `http://localhost:5204/news/`。

本 app 已接入 vitest（`vitest.config.js`，jsdom 环境）。纯逻辑优先抽成可测模块，例如段落内联渲染助手 `features/blog/inlineMarkdown.js`（`escapeHtml`/`parseInlineStyles`/`renderBlockMarkdown`，含同目录 `*.test.js`）。拆巨型组件时，建议先抽纯函数/composable 并补单测，再动模板。

## 目录

```text
src/
  main.js
  App.vue
  router/index.js
  stores/main.js
  shell/
    NavBar.vue
    SiteFooter.vue
  features/
    blog/
    handbook/
    activity/
    store/
    quiz/
    admin/
  utils/masonry.js
  style.css
public/
  图片、logo、Live2D model 脚本等静态资源
```

`router/index.js` 负责组合 feature 路由，各 feature 在自己的 `routes.js` 中声明路径和懒加载视图。

## 功能范围

- 团内新闻（春日团报）的投稿、列表、详情和搜索。
- 团员手册长文。
- 活动展示。
- 积分商城和奖品管理。
- 入坑测试。
- 新闻总览海报导出。
- 后台审核和管理，按 `news.*` 子作用域控制菜单和操作。

## 后端契约

- 公共和后台业务接口都走 `createApiClient('/api/news')`。
- 登录和当前用户走 `/api/auth/*`。
- 后台权限使用 `createAdminAuth('news')` 和 `hasScope()`。
- `news` 拆分为 `news.blog`、`news.activity`、`news.store`、`news.points` 四个子作用域。
- 图片字段通常已是 `/uploads/news/...` 绝对路径，前端直接使用。

## 维护注意

- Vite `base` 是 `/news/`，Router 使用 `createWebHistory(import.meta.env.BASE_URL)`。
- `index.html` 中包含桌面端 Live2D 相关外链脚本，改 CSP 或 Nginx 安全策略时要一并检查。
- 改后台 tab、按钮或接口时，同时确认后端 `authorize()` 的作用域和前端 `hasScope()` 一致。
