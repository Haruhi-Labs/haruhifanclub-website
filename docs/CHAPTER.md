# Chapter 地方支部子站

Chapter 是与 `news`、`art` 平级的独立前端应用，生产 canonical 地址为 `https://chapter.haruyuki.cn/`。它使用同一套 axum 后端、core 账号库、上传目录以及 SOS / Parallel Design System，不在 `apps/news` 中复制页面。

## 本地开发

根目录 `.env` 使用 `HARUHI_BIND=127.0.0.1:17777`。本地不要设置 `HARUHI_COOKIE_DOMAIN`，否则浏览器不会接受 localhost Cookie。

```bash
pnpm dev:backend
pnpm dev:chapter
```

- 前端：`http://127.0.0.1:5208/`
- 健康检查：`http://127.0.0.1:17777/api/health`
- API：`/api/chapter/*`
- 数据库：`data/chapter.db`
- 上传目录：`uploads/chapter/`

首次创建支部时，使用开发超级管理员登录 `/login`，进入 `/admin/branches`。普通管理员必须获得具体 `branchId` 下的能力授权，一个普通管理员账号默认最多管理一个支部。只有当前登录的总负责人可以输入自己的超管密码，为账号增加第二个支部、授予 Chapter 平台级能力或完成跨支部管理交接。密码只用于当前请求校验，不保存在 Chapter 数据库或审计记录中。

## 管理界面

支部管理页通过可视化表单维护支部资料、页面导航、单一 Logo、封面、QQ群、负责人、成员、组织架构、活动相册、活动运营和实例权限，不要求运营人员编辑 JSON。

- `/branches/:branchSlug/manage/:panel/:subpanel`：管理栏目与子栏目；
- `/branches/:branchSlug/manage/events`：已有活动列表；
- `/branches/:branchSlug/manage/events/new/:section`：新建活动的一条龙流程；
- `/branches/:branchSlug/manage/events/:eventId/:section`：单个已有活动的编辑和运营；
- `/branches/:branchSlug/manage/events/invitations`：联合主办邀请；
- `/branches/:branchSlug/manage/albums`：按活动上传、搜索、编辑和删除相册照片；
- `/branches/:branchSlug/manage/merchandise`：维护、排序和发布支部特色周边；
- 旧管理地址 `/branches/:branchSlug/manage/timeline` 会重定向到相册管理。

活动步骤的 `section` 为 `details`、`registration`、`people`、`cohosts` 和 `review`。切换栏目或离开活动编辑流程前，如果存在尚未保存的修改，页面会要求运营人员确认。

包含多个子模块的栏目使用二级导航；记录型内容使用可搜索的折叠或选择列表，点击后再展开详细表单。成员页分为普通成员、在任成员、往届成员和退出申请。在任成员只能从已经加入该支部的账号中搜索选择，昵称和头像来自 core 账号资料。组织架构每次发布都会生成新版本。

## 活动时间线与活动相册

公开“活动时间线”完全根据已发布活动生成，不需要管理员另外维护动态记录。全国和支部时间线按活动去重，每项只返回：

- `id`
- `branchSlug`
- `branchLocalityName`
- `eventSlug`
- `title`
- `startsAt`
- `endsAt`

页面只展示活动开始时间、结束时间、主办支部地方名称和活动名称。`branchLocalityName` 为空时页面不显示地方胶囊；联合举办活动使用主办支部的地方名称。`event` 参数仅用于筛选同一份最小活动数据，不返回相册说明、地点或图片。兼容 SSE `/api/chapter/timeline/stream` 只发送刷新信号，公开页面不依赖它。

原现场记录存储现在作为“活动相册”使用。管理员必须把照片关联到本支部活动；草稿允许暂不上传图片，公开照片必须同时满足：

- 关联活动已经发布；
- 照片已上传；
- 照片状态为 `published`；
- 照片审核状态为 `normal`。

公开接口为 `/api/chapter/branches/:slug/events/:eventSlug/photos`。照片只在对应活动详情的“活动相册”中出现，不会混入活动时间线。管理端使用 `/api/chapter/admin/branches/:id/albums`；旧 `/timeline` 管理接口继续作为迁移兼容别名。数据库表名 `branch_event_timeline_entries` 和能力键 `branch.timeline.write`、`branch.timeline.publish` 暂时保持不变，避免破坏已有迁移与授权数据，但用户界面统一显示为活动相册。

## 特色周边

每个支部可以维护自己的特色周边展示，字段包括单张展示图片、名称、简介、标签、状态和排序。周边只作支部文化展示，不包含价格、库存、购买链接或商城数据关联。

管理接口为 `/api/chapter/admin/branches/:id/merchandise`，沿用 `branch.profile.manage` 能力；保存操作写入支部审计日志。草稿可以暂不上传图片，公开状态必须提供展示图片。公开接口为 `/api/chapter/branches/:slug/merchandise`，只返回已发布项目；支部至少有一件已发布周边时，公开导航才显示“特色周边”入口。

## 数据与权限边界

Chapter 保存支部资料、品牌、栏目、支部归属、正式成员、组织版本、QQ群、经同意公开的负责人联系方式、活动、活动相册、报名和签到记录。普通成员列表只公开 core 账号的昵称和头像，不公开邮箱、用户名、电话或其他个人资料；活动管理员可在受保护后台查看报名人的真实账号信息。

支部正文保存为纯文本，不接受自定义 HTML 或 CSS。品牌只允许上传受检图片并选择允许列表内的强调色。支部默认可以自行发布；公开内容只有在 `published + public + normal` 时可访问，全国聚合还要满足内容聚合开关。平台可以隐藏异常内容、暂停或归档支部，操作写入 `branch_audit_log`。

一个账号同时只能加入一个线下支部。加入操作会提示不可自行退出，并要求再次输入当前账号密码；退出必须提交申请，由所属支部管理员批准。活动支持站内、站外或混合报名，支持自动确认或审核、人数上限、候补、受控问题和签到。参与者名单默认公开昵称和头像；用户可选择匿名，匿名时公开页只显示默认头像和该活动内稳定编号，活动管理员仍可查看真实资料。

新发布活动必须提供晚于开始时间的结束时间；旧活动缺少结束时间时公开页显示“结束时间待定”。全国活动与活动时间线支持 `page`、`pageSize` 分页，响应同时返回 `total`、`page` 和 `pageSize`，旧筛选参数继续兼容。

权限由 `core.capability_grants` 提供，作用域为 `platform/chapter` 或 `branch/<id>`。后端鉴权是最终边界，前端能力判断只控制入口显示。

## 子域部署

1. 为 `chapter.haruyuki.cn` 和 `chapter.test.haruyuki.cn` 创建 DNS 记录。
2. 为两个域名单独签发 TLS 证书。
3. 安装 `deploy/chapter.haruyuki.cn.nginx.conf` 与 `deploy/chapter.test.haruyuki.cn.nginx.conf`，运行 `nginx -t` 后 reload。
4. 生产环境设置：

```dotenv
CHAPTER_SITE_URL=https://chapter.haruyuki.cn
HARUHI_COOKIE_DOMAIN=.haruyuki.cn
```

测试环境使用 `CHAPTER_SITE_URL=https://chapter.test.haruyuki.cn` 和 `HARUHI_COOKIE_DOMAIN=.test.haruyuki.cn`，使测试与生产会话隔离。修改 Cookie 域后需要重启后端并重新登录。

Chapter 子域的 `/sitemap.xml` 由 Nginx 映射到后端 `/sitemap-chapter.xml`。详情页请求由后端注入 canonical、Open Graph 和 JSON-LD，其余路由由静态 SPA 处理。

## 上线与回滚

迁移随后端启动自动执行。部署前同时备份 `core.db` 和 `chapter.db`，发布顺序为后端二进制与迁移、前端构建、环境变量、Nginx/DNS。`0003_membership_timeline_events.sql` 为活动增加报名字段，并新增归属、相册存储和活动运营表；`0005_merchandise.sql` 新增支部特色周边表与公开查询索引；`core/0008` 保留旧能力到当前兼容能力键的迁移。

回滚应用代码不会自动回滚 SQLite schema，旧二进制会忽略新增结构。验收至少包括跨子域登录、支部归属唯一性、成员账号绑定、匿名报名、候补与签到、活动时间线、活动相册、支部权限隔离、公开联系方式授权、聚合排除、暂停归档、移动端和 sitemap。
