# @haruhi/fiction

同人小说站 app：用户创作、发布、阅读凉宫春日同人文。包含书库、作品页、沉浸阅读器、富文本创作编辑器和创作中心，接入统一身份系统与个人控制台。

## 入口

- 子路径：`/fiction/`
- dev 端口：`5207`
- 后端接口：`/api/fiction/*`
- 上传资源：`/uploads/fiction/*`（作品封面）
- 技术栈：Vue 3、Vue Router、Vite、Tiptap（`@tiptap/*` 富文本编辑器）、`dompurify`、`@haruhi/design-system`、`@haruhi/ui`、`@haruhi/auth-ui`、`@haruhi/api-client`

## 本地运行

```bash
bash deploy/gen-secrets.sh
pnpm dev:backend

pnpm --filter @haruhi/fiction dev
pnpm --filter @haruhi/fiction build
pnpm --filter @haruhi/fiction preview
```

访问 `http://localhost:5207/fiction/`。

## 路由

| 路径                          | 视图                  | 说明                     |
| ----------------------------- | --------------------- | ------------------------ |
| `/`                           | `HomeView`            | 首页：精选、分类、最新   |
| `/library`                    | `LibraryView`         | 书库：分类/标签/排序筛选 |
| `/story/:id`                  | `StoryView`           | 作品页：简介、目录、评论 |
| `/story/:id/chapter/:cid`     | `ReadView`            | 沉浸阅读器（隐藏页头）   |
| `/bookmarks`                  | `BookmarksView`       | 我的书架（需登录）       |
| `/write`                      | `WriteDashboardView`  | 创作中心（需登录）       |
| `/write/:id`                  | `StoryEditorView`     | 作品设置与章节管理       |
| `/write/:id/chapter/:cid`     | `ChapterEditorView`   | 章节富文本编辑（自动保存）|
| `/login` `/reset-password` `/verify-email` | `@haruhi/auth-ui` | 统一登录/找回/验证    |
| `/account/*`                  | `UserConsoleLayout`   | 个人控制台（需登录）     |

## 目录

```text
src/
  main.js
  App.vue
  api.js                 # 接口封装 + session + 封面 URL
  router/index.js
  lib/
    format.js            # 分类、分级、字数/时长格式化
    reader.js            # 阅读器主题/宽度设置（localStorage 持久化）
  components/
    CoverImage.vue       # 封面，缺图时按分类生成占位
    StoryCard.vue        # 作品卡片（网格/紧凑两态）
    CommentSection.vue   # 两级评论
    RichEditor.vue       # Tiptap 编辑器封装 + 工具栏
    SiteFooter.vue
  views/
    HomeView.vue LibraryView.vue StoryView.vue ReadView.vue
    BookmarksView.vue WriteDashboardView.vue
    StoryEditorView.vue ChapterEditorView.vue NotFoundView.vue
```

## 功能范围

- 书库按分类、标签、连载状态、分级筛选，支持最新/更新/人气/阅读/字数排序与搜索。
- 作品页展示封面、简介、字数/章节/阅读/点赞/收藏统计、标签、目录和评论。
- 沉浸阅读器：上一章/下一章、目录跳转、纸张/护眼/夜间主题与版心宽度切换，登录用户自动记录阅读进度。
- 创作中心：作品草稿/发布/隐藏三态，章节草稿/发布两态；作品发布要求至少一个已发布章节。
- 富文本编辑器基于 Tiptap，支持标题、加粗/斜体/下划线/删除线、引用、列表、分割线、链接、插图；编辑器 1.5s 防抖自动保存。
- 收藏（书架）、点赞、评论等互动均强制登录。

## 后端契约

- 公开读取：`/api/fiction/stories`、`/api/fiction/stories/{id}`、`/api/fiction/stories/{id}/chapters/{cid}`、`/api/fiction/categories`、`/api/fiction/tags`。
- 创作接口：`/api/fiction/me/stories/*`、`/api/fiction/me/covers`，均需登录且校验作者归属。
- 互动 + 个人数据：`/api/fiction/stories/{id}/like|bookmark`、`/api/fiction/comments`、`/api/fiction/me/*`。
- 后台审核：`/api/fiction/admin/*`，按 RBAC scope `fiction`（`Action::Moderate/Manage`）授权。
- UGC 署名走 `author_user_id`（逻辑关联 `core.users.id`）+ `require_verified_member`；封面 URL 用 `resolveUploadUrl(path)`。

## 维护注意

- Vite `base` 使用 `/fiction/`，部署（`deploy/nginx.conf`、`deploy/test.haruyuki.cn.nginx.conf`）和路由都按这个子路径配置。
- 章节正文为富文本，写入时后端用 `ammonia` 白名单清洗，读取时前端再经 `dompurify` 二次防御，改动编辑器允许标签时两端需同步。
- 表现模式复用 `library`（奶油/衬线/琥珀），并在 app 根覆盖为「书卷酒红」强调色，`src/style.css` 内的覆盖选择器同时作用于 app 根与内嵌的 `data-sos-site="library"` 区块（账号中心）。
- 数据库为 `data/fiction.db`，迁移在 `backend/migrations/fiction/`，迁移文件只增不改（sqlx 校验哈希）。
- 新增/改结构时同步检查 [../../docs/ADDING_MODULE.md](../../docs/ADDING_MODULE.md)。
