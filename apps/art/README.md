# @haruhi/art · 绘画部画廊

凉宫春日应援团「绘画部」作品画廊前端：浏览/筛选/搜索画作、查看大图与多图轮播、匿名点赞与评论、创作者投稿上传、积分榜与积分查询、特别授权查询，以及一套面向管理员的审核后台。部署于子路径 `/art/`，所有接口走统一后端 `/api/art/*`。

## 技术栈与关键依赖

- Vue 3（`<script setup>` + Options/Composition 混用）、Vue Router 4（`createWebHistory`）、Pinia 3
- Vite 7 + `@vitejs/plugin-vue`，原生 CSS（无 UI 框架、无 Tailwind）
- `@haruhi/api-client`（workspace 共享包）：本 app 用到 `getToken`、`resolveUploadUrl`、`createAdminAuth('art')`
- 浏览器端图片压缩（Canvas 转 WebP）、轻量 WebAudio 点击音效（无需音频资源）

## 目录结构要点

```
src/
  main.js              # 先注册 Pinia 再注册 Router（顺序关键，见文件注释）
  router/index.js      # 5 个路由：/ 画廊、/upload 投稿、/admin 后台、/points 积分、/license 授权查询
  services/api.js      # 唯一的后端访问层：统一前缀 /api/art，封装全部公开/互动/管理接口
  stores/
    galleryStore.js    # 画廊筛选/搜索/排序/分页状态；接口失败时回退到 mock 种子数据
    adminStore.js      # 待审核作品列表与审核动作
  views/               # GalleryView / UploadView / AdminView / PointsView / LicenseView
  components/          # TopBar 导航、FilterPanel 筛选、ArtworkGrid 响应式网格、ArtworkModal 大图弹窗、TagPill
  config/ui.js         # 主题色（accentHue）与点击音效开关
  utils/
    imageCompressor.js # compressToWebP(file, quality, maxWidth)
    uiSound.js / Sound.js # UI 点击音效（Sound.js 为旧命名兼容层）
  mock/seedData.js     # 内联 SVG 种子作品/创作者/评论，仅作后端不可用时的本地兜底
  assets/              # logo / kon / lucky / upload-bg 等 webp 装饰图
public/bg.webp         # 画廊背景图
```

注：`AdminView.vue` / `ArtworkModal.vue` / `UploadView.vue` 均为千行级大文件（后台审核、多图缩放轮播、投稿表单），属已知技术债。

## 本地开发

前置：先在仓库根起统一后端 `cargo run -p haruhi-server`（监听 `127.0.0.1:17777`）。

```bash
pnpm --filter @haruhi/art dev      # Vite dev，端口 5201，base=/art/
pnpm --filter @haruhi/art build    # 产物输出 dist/
pnpm --filter @haruhi/art preview   # 预览 build 产物
```

- dev 端口 **5201**，部署子路径 **/art/**（见 `vite.config.js` 的 `base`）。
- dev 下 `/api` 与 `/uploads` 经 Vite proxy 转发到 `http://127.0.0.1:17777`（`changeOrigin`）。

## 关键特性与约定

- **统一接口前缀**：所有请求经 `services/api.js`，前缀固定 `/api/art`；静态资源根为 `/uploads`。
- **匿名互动**：点赞/评论由后端用签名 Cookie 维持匿名身份，故所有请求携带 `credentials: 'include'`，无需登录即可点赞、发评论。
- **管理员鉴权（统一 JWT）**：后台登录走 `createAdminAuth('art')`（用户名 + 密码 → 单点登录 JWT），不再用旧的 `x-admin-password`；请求层自动从 `getToken()` 读取 token 并加 `Authorization: Bearer <jwt>`。
- **AI 审核**：投稿提交后由后端 AI 审核，前台据返回提示「AI 审核通过/进入待审核队列」，并在通过时自动发放积分（审核逻辑在后端，前端不做判定）。
- **图片处理**：投稿前 `compressToWebP` 在浏览器端压缩转 WebP 再上传（FormData，`isForm`）。
- **作品维度**：`content_type`（春日/其他）、`source_type`（personal 个人原创 / network 网络搬运）、标签、创作者 uid——筛选、搜索、按作者/标签聚合均围绕这些字段。
- **本地兜底**：`galleryStore` 在接口异常时回退 `mock/seedData.js` 的种子数据，保证页面可用（排序键 `stableRandKey` 与后端 SQL 哈希对齐，保证随机序一致）。
- **积分体系**：PointsView 提供创作者 ID 搜索、积分榜、积分明细；约定 10 绘画部积分 = 1 应援团积分。

## 与共享层 / 后端的关系

- 仅通过 `@haruhi/api-client` 复用鉴权与 URL 拼接：上传图片字段（`image_url` / `original_url` / `avatar_url` 等存的是相对 uploads 根的路径）统一交给 `resolveUploadUrl(path, '/uploads')` 拼成可访问 URL（art 与 novel 已去重到此函数）。
- 后端为单进程 `haruhi-server`，本模块对应 `/api/art/*` 路由与 `data/art.db`、`uploads/art/`；后台接口受后端 RBAC（`art` 模块授权）保护。

## 更多

- 仓库总览与架构约定：[../../README.md](../../README.md)
- 协作与提交规范：[../../CONTRIBUTING.md](../../CONTRIBUTING.md)
- 新增模块流程：[../../docs/ADDING_MODULE.md](../../docs/ADDING_MODULE.md)
