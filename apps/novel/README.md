# @haruhi/novel

书库 app，包含 EPUB 书架、阅读器、投稿反馈页和后台上传/编目。

## 入口

- 子路径：`/library/`
- dev 端口：`5203`
- 后端接口：`/api/novel/*`
- 上传资源：`/uploads/novel/*`
- 技术栈：Vue 3、Vue Router、Tailwind CSS、Vite、`epubjs`、`jszip`、`axios`、`@haruhi/api-client`

## 本地运行

```bash
bash deploy/gen-secrets.sh
pnpm dev:backend

pnpm --filter @haruhi/novel dev
pnpm --filter @haruhi/novel build
pnpm --filter @haruhi/novel preview
```

访问 `http://localhost:5203/library/`。

## 路由

| 路径        | 视图           | 说明             |
| ----------- | -------------- | ---------------- |
| `/`         | `Shelf`        | 书架首页         |
| `/read/:id` | `Reader`       | EPUB 阅读器      |
| `/admin`    | `Admin`        | 上传、编目、删除 |
| `/feedback` | `FeedbackView` | 投稿和反馈表单   |

## 目录

```text
src/
  main.js
  App.vue
  router/index.js
  components/FooterBar.vue
  views/
    Shelf.vue
    Reader.vue
    Admin.vue
    FeedbackView.vue
  assets/
```

## 功能范围

- 书架按分类展示：正传、设定集、官方短篇、社区同人。
- 阅读器下载 EPUB，解析 spine、目录、章节和图片。
- 支持滚动和多栏翻页。
- 支持原文、简体、繁体三态切换；OpenCC 从 `index.html` 外链加载，失败时显示原文。
- 目录支持章节和章内锚点。
- 后台上传 `.epub`，后端提取标题、作者、封面和文件路径。
- 后台可编辑标题、作者、分类、排序并删除书籍。

## 后端契约

- 公开接口：`/api/novel/books`、`/api/novel/books/{id}`。
- 后台接口：`/api/novel/admin/*`。
- 后台登录使用 `createAdminAuth('novel')`。
- 上传资源路径用 `resolveUploadUrl(path)`。
- EPUB 解析和封面处理由后端 `haruhi-media` 负责。

## 维护注意

- Vite `base` 使用 `/library/`，部署和路由都按这个子路径配置。
- 公开阅读接口部分代码使用 `axios`，后台接口使用 `@haruhi/api-client`。
- `Reader.vue` 处理 EPUB 解析、翻页、锚点和简繁转换，改动时注意阅读进度和 blob URL 清理。
- `novel` 是新增模块流程的参考实现，改结构时同步检查 [../../docs/ADDING_MODULE.md](../../docs/ADDING_MODULE.md)。
