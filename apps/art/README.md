# @haruhi/art

绘画部画廊 app，包含作品浏览、投稿、匿名点赞/评论、创作者积分、授权查询和审核后台。

## 入口

- 子路径：`/art/`
- dev 端口：`5201`
- 后端接口：`/api/art/*`
- 上传资源：`/uploads/art/*`
- 技术栈：Vue 3、Vue Router、Pinia、Vite、原生 CSS、`@haruhi/api-client`

## 本地运行

```bash
bash deploy/gen-secrets.sh
pnpm dev:backend

pnpm --filter @haruhi/art dev
pnpm --filter @haruhi/art build
pnpm --filter @haruhi/art preview
```

访问 `http://localhost:5201/art/`。

## 目录

```text
src/
  main.js
  router/index.js
  services/api.js        art 模块 API 封装
  stores/
    galleryStore.js
    adminStore.js
  views/
    GalleryView.vue
    UploadView.vue
    AdminView.vue
    PointsView.vue
    LicenseView.vue
  components/
    TopBar.vue
    FilterPanel.vue
    ArtworkGrid.vue
    ArtworkModal.vue
    TagPill.vue
  utils/
    imageCompressor.js
    uiSound.js
    Sound.js             旧命名兼容层
  mock/seedData.js       后端不可用时的本地兜底数据
```

## 功能范围

- 按标签、作者、内容类型、来源、时间、热度、随机排序筛选作品。
- 投稿时浏览器侧压缩 WebP，再提交后端审核。
- 匿名点赞和评论，身份由后端签名 Cookie 维护。
- 创作者积分榜和积分查询。
- 特别授权查询。
- 管理后台：审核、编辑、创作者管理、评论管理。

## 后端契约

- 请求集中在 `src/services/api.js`，前缀是 `/api/art`。
- 后台登录使用 `createAdminAuth('art')`。
- 后台请求带 `Authorization: Bearer <jwt>`。
- 匿名互动请求需要携带 Cookie，接口层保持 `credentials: 'include'`。
- 上传路径用 `resolveUploadUrl(path, '/uploads')` 转成可访问 URL。
- AI 审核结果由后端决定，前端只展示返回状态。

## 维护注意

- Vite `base` 是 `/art/`。
- `AdminView.vue`、`ArtworkModal.vue`、`UploadView.vue` 体量较大，改动时优先保持局部变更。
- `galleryStore` 在接口失败时会使用 `mock/seedData.js`，排查“数据为何还能显示”时注意这层兜底。
