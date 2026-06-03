# @haruhi/news — 春日团报 / 京都学报主站

凉宫春日应援团的内容主站与门户：博客·学报投稿、团员手册、积分商城、活动中心、入坑小测试，以及一个统一登录、按权限门控的后台。本应用是 monorepo 中 6 个前端 app 之一，独立构建后挂载在 `haruyuki.cn` 的 `/news/` 子路径下。

## 技术栈与关键依赖

- Vue 3（`<script setup>`）+ Vue Router 4 + Pinia 3
- 构建：Vite 7（`@vitejs/plugin-vue`），样式为**原生 CSS**（全局 `src/style.css` + 组件 scoped 样式 + PostCSS/Autoprefixer），无 CSS 框架
- `@haruhi/api-client`（workspace 共享包）：统一 API 客户端、JWT、后台鉴权
- `html-to-image`：后台「新闻总览」海报导出为 PNG
- `index.html` 中按运行环境（仅桌面、非移动端）随机注入 Live2D 桌宠脚本，并加载 PixiJS（外链 CDN）

## 目录结构要点

```
src/
  main.js               入口：挂载 Pinia + Router
  App.vue               外壳：NavBar（按 route.meta.hideNavbar 隐藏）+ router-view + 全局搜索/详情弹窗
  router/index.js       汇总各 feature 的路由数组
  stores/main.js        唯一 Pinia store：文章/奖品/活动列表、管理员态、搜索态
  shell/                NavBar.vue、SiteFooter.vue
  utils/masonry.js      首页瀑布流卡片高度估算
  features/             按特性切分（每个特性自带 routes / views / 可选 admin / 可选 components）
    blog/               主站核心：首页 HomeView、投稿 EditorView、详情 BlogDetailView；
                        admin/ArticleAdmin（审核/已发布）、admin/GeneratorAdmin（新闻总览海报生成）
    handbook/           团员手册（HandbookView 静态长文 + 锚点侧栏）
    store/              积分商城 StoreView + admin/PrizeAdmin
    activity/           活动中心 ActivityView + admin/ActivityAdmin
    quiz/               入坑小测试 QuizView（meta.hideNavbar，整页自管布局）
    points/             积分管理 admin/PointsAdmin（无前台页，仅后台 tab）
    admin/              后台壳 AdminView：登录 gate + RBAC tab 门控 + 懒加载各特性 admin 子组件
public/                 站点图片、字体、quiz 素材、Live2D 模型脚本（model_*.js）
```

## 本地开发

前置：在仓库根目录 `pnpm install`，并确保后端 `haruhi-server` 已在 `127.0.0.1:17777` 运行（`cargo run -p haruhi-server`）。

```bash
# 开发（端口 5204，base 子路径 /news/）
pnpm --filter @haruhi/news dev

# 生产构建 / 本地预览
pnpm --filter @haruhi/news build
pnpm --filter @haruhi/news preview
```

`vite.config.js` 已配置 `base: '/news/'`，并把 `/api`、`/uploads` 代理到 `http://127.0.0.1:17777`；`@` 别名指向 `src/`。Router 用 `createWebHistory(import.meta.env.BASE_URL)`，与子路径部署对齐。

## 关键特性与约定

- **特性化路由**：`router/index.js` 仅做组合，各特性在自己的 `routes.js` 中声明 path/name/meta，视图统一懒加载以按 feature 分包。
- **统一后端约定**（见 `stores/main.js` 注释）：所有业务 API 走 `createApiClient('/api/news')` 统一前缀 `/api/news/*`；图片字段库里已是绝对路径 `/uploads/news/<md5>.<ext>`，前端直接使用，**不再拼接前缀**（故本 app 未用到 `resolveUploadUrl`）；已移除旧的 `X-Admin-Token` / `VITE_*` 环境变量逻辑。
- **后台壳 + RBAC**：`AdminView` 用 `createAdminAuth('news')` 完成登录、会话恢复、登出；用 `hasScope(user, scope)` 按子作用域门控 tab —— `activities` → `news.activity`、`prizes` → `news.store`、`pending`/`published`/`generator` → `news.blog`、`points` → `news.points`（持有父级 `news` 或超管者可见全部）。每个 tab 对应的 admin 子组件按需 `defineAsyncComponent` 懒加载。
- **新闻总览生成器**：`GeneratorAdmin` 按时间尺度筛选文章渲染海报，再用 `html-to-image` 导出 PNG。
- **Quiz 整页接管**：`/quiz-game` 设 `meta.hideNavbar`，App 外壳不套默认布局容器。
- **技术债提醒**：`features/blog`（约 8k 行）及各 `admin/*.vue`（活动/奖品/积分/文章审核/生成器）为大体量单文件组件，样式与逻辑高度集中，是已知重构目标；新增内容时优先沿特性边界拆分，避免继续堆积。

## 与共享层 / 后端的关系

- 依赖共享包 `@haruhi/api-client`：`createApiClient`（自动注入 `Authorization: Bearer <jwt>`）、`createAdminAuth`、`hasScope`。
- 登录走统一 `/api/auth`，业务走 `/api/news/*`，静态资源走 `/uploads/news/*`，均由后端单进程 `haruhi-server` 提供。
- 后端 news 模块的后台接口受 `haruhi_auth::authorize(..., "news", Action::X)` 保护，前端 tab 的可见性与之对应。

## 更多

- 仓库总览与架构约定：根目录 [`../../README.md`](../../README.md)
- 协作与提交规范（scope 集合含 `news`）：[`../../CONTRIBUTING.md`](../../CONTRIBUTING.md)、[`../../docs/`](../../docs/)
