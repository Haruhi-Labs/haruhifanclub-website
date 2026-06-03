# @haruhi/novel · 长门有希的书架

凉宫春日应援团的在线小说书库前端：以书架形式展示馆藏书籍，点开即在浏览器内解析并阅读 EPUB，并提供管理员后台上传与编目。部署在 `haruyuki.cn` 的 `/library/` 子路径下。

## 技术栈与关键依赖

- Vue 3（`<script setup>`，单文件组件）+ Vue Router 4（`createWebHistory`），无 Pinia，少量页面级状态用 `ref` / `reactive` 局部管理。
- 样式用 Tailwind CSS 3（`tailwind.config.js` + `postcss.config.js`，配色为应援团米黄/赤陶主题 `#FAF9DE` / `#D97757`）。
- [`epubjs`](https://github.com/futurepress/epub.js) 解析 EPUB，配合 `jszip` 直接从压缩包内读取章节文件与图片。
- 简繁转换通过 `index.html` 里以 `<script>` 引入的 `opencc-js`（CDN，全局 `window.OpenCC`），未加载时自动回退原文。
- HTTP 请求：阅读/书架等公开接口直接用 `axios`；后台用共享 `@haruhi/api-client` 的封装客户端。
- 构建：Vite 7（`@vitejs/plugin-vue`）。

## 目录结构要点

```
apps/novel/
├─ index.html              # 入口；引入站点字体与 opencc-js CDN，标题“长门有希的书架”
├─ vite.config.js          # base=/library/，dev :5203，代理 /api 与 /uploads → 17777
├─ tailwind.config.js      # 扫描 index.html + src/**
└─ src/
   ├─ main.js              # 挂载 App、注册 router
   ├─ App.vue              # 外层布局 + 统一 FooterBar
   ├─ router/index.js      # 4 条路由（见下）
   ├─ components/
   │  └─ FooterBar.vue     # 全站页脚
   └─ views/
      ├─ Shelf.vue         # 书架首页：分栏展示 + 封面懒加载/WebP 压缩
      ├─ Reader.vue        # 阅读器：EPUB 解析、流式/翻页、简繁切换、目录锚点
      ├─ Admin.vue         # 管理后台：登录、EPUB 上传、编目（标题/作者/分栏/排序）
      └─ FeedbackView.vue  # 同人投稿 & 问题反馈（内嵌腾讯文档表单）
```

路由（`src/router/index.js`）：`/`（Shelf）、`/read/:id`（Reader，`props: true`）、`/admin`（Admin）、`/feedback`（FeedbackView）。

## 本地开发

前置：先在仓库根启动统一后端 `cargo run -p haruhi-server`（监听 `127.0.0.1:17777`），novel 模块对应 `/api/novel/*` 与 `/uploads/novel/*`。

```bash
pnpm --filter @haruhi/novel dev       # Vite 开发服务器，端口 5203，base=/library/
pnpm --filter @haruhi/novel build     # 产物输出 dist/
pnpm --filter @haruhi/novel preview    # 本地预览构建产物
```

dev 下 `/api` 与 `/uploads` 由 Vite 代理到 `http://127.0.0.1:17777`（见 `vite.config.js`），无需额外配置跨域。脚本仅 `dev` / `build` / `preview` 三个（来自 `package.json`），无 lint / 测试脚本。

## 关键特性与约定

- 书架分栏：`Shelf.vue` 内置 `CATEGORY_CONFIG`（`main` 正传小说 / `setting` 设定集 / `short` 官方短篇 / `fanfic` 社区同人），按书籍的 `category` 字段分桶，缺省归入 `main`；配置外出现的类别会追加在末尾。后台 `Admin.vue` 的分栏选项与之保持一致。
- 封面优化：列表封面先把后端返回路径的扩展名替换为 `.webp` 尝试加载，并在前端用 canvas 把封面预压缩为 WebP（最大宽 800px）缓存，加载失败时回退原图。
- 阅读器（`Reader.vue`）核心能力：
  - 拉取书籍元数据后，按 `file_path` 下载整本 EPUB（`arraybuffer`，带下载进度），用 epubjs 解析目录（navigation/spine）与章节。
  - 两种阅读模式：流式滚动与多栏翻页（`column-*` + transform 平移，支持 ←/→、PageUp/Down、空格翻页）。
  - 简繁三态切换（原文 / 简体 / 繁体），基于 OpenCC，仅重渲染文本不重置进度。
  - 目录支持章节 + 章内锚点：为避免多文件 id 冲突，给每个 spine 文档的 `id` 与 `#锚点` 统一加「文件名前缀」（`makeDomId`），并据此做高亮与跳转。
  - 章内图片从 EPUB 压缩包取出转 blob URL 注入，兼容 `<img>` 与 SVG `<image>`。
- 后台（`Admin.vue`）：上传 `.epub` 到 `/admin/upload`（后端自动提取封面与标题），可就地编辑标题/作者/分栏/排序值（数值越小越靠前）并保存，或删除书籍。

## 与共享层 / 后端的关系

- 依赖 `@haruhi/api-client`（`workspace:*`）：
  - `resolveUploadUrl(path)` 统一拼接 `/uploads/...` 静态资源 URL（封面、EPUB 文件），与 art 等 app 共用同一实现。
  - 后台用 `createApiClient('/api/novel')` 收发模块接口，`createAdminAuth('novel')` 处理登录 / 会话恢复 / `novel` 权限校验（`login` 不抛错，失败返回 `{ ok:false, error }`）。
- 后端为统一进程 `haruhi-server` 的 novel 模块：公开接口 `/api/novel/books`、`/api/novel/books/:id`；后台接口 `/api/novel/admin/*`（鉴权后由 `haruhi-auth` 校验 `novel` 权限），上传走 `haruhi-media` 的 EPUB 解析。novel 是仓库的端到端新增模块模板（见根 `docs/ADDING_MODULE.md`）。

## 更多

- 仓库总览与架构约定：根 [README](../../README.md)
- 协作与提交规范：[CONTRIBUTING](../../CONTRIBUTING.md)、[docs/COLLABORATION.md](../../docs/COLLABORATION.md)
- 新增模块指引：[docs/ADDING_MODULE.md](../../docs/ADDING_MODULE.md)（novel 为参考模板）
