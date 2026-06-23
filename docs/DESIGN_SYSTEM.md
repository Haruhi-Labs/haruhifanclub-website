# SOS / Parallel Design System 项目设计规范

版本：`0.2.0`

适用范围：`news`、`shop`、`art`、`novel`、`exam` 以及项目内新增页面。`console` 是运维管理界面，默认采用克制的中性界面，不单独建立角色化 Expression Mode。

规范展示页：`apps/design-system`，部署子路径为 `/design-system/`。

共享样式包：`packages/design-system`，包名为 `@haruhi/design-system`。

共享 Vue 基础组件包：`packages/ui`，包名为 `@haruhi/ui`。

## 0. 文档定位

这份文档首先是项目内的设计手册，其次才是 monorepo 接入说明。它要回答“页面应该怎么长、组件应该怎么用、信息应该怎么组织”，再回答“这些规则如何在当前工程里落地”。

当前版本曾经把很多篇幅放在接入结构、包治理和迁移策略上，这对工程落地有必要，但不足以对标成熟设计系统。后续增厚优先级如下：

1. **设计对象前置**：颜色、排版、按钮、导航、卡片、表单、间距、圆角、深度、动效、响应式先讲清楚。
2. **真实业务优先**：规范示例来自 `shop` 商品/订单、`art` 作品/上传、`news` 文章/发布、`novel` 阅读、`exam` 答题等真实路径。
3. **工程治理后置**：包结构、bridge、版本规则、迁移证据保留，但不抢占设计规范主线。
4. **UI 库谨慎扩张**：先让基础组件和真实站点互相校验，业务卡片继续作为 recipe 验证，不把尚未稳定的信息结构抽成共享组件。

### 0.1 成熟案例对标结论

`getdesign.md` 的 Airbnb 与 Apple/Lumora 预览页提供的主要启发不是某种具体视觉风格，而是内容组织方式：

- Airbnb 案例以 18 个 section 展开：颜色、排版、按钮、顶部导航、搜索、分类、房源卡、体验卡、评分、设施、预订栏、日期、表单、城市链接、间距、圆角、深度、响应式。
- Apple/Lumora 案例以 12 个 section 展开：颜色、排版、按钮、产品 tile、商店卡片、配置器/搜索/粘性栏、导航、表单、间距、圆角、深度、响应式。
- 两个案例都把“设计对象”放在主线，把组件放进真实页面语境，并给出具体尺寸、状态和响应式变化。

因此本项目后续的规范页应减少纯架构表格的占比，增加真实组件、真实页面模式、真实内容字段和真实响应式行为的展示。

### 0.2 站点审计和试点选择

已审计入口、壳层、主要视图和样式文件后，第一批真实接入选择为 `shop` 与 `art`：

| App               | 真实 UI 主语                         | 当前问题                                               | 第一阶段动作                                       |
| ----------------- | ------------------------------------ | ------------------------------------------------------ | -------------------------------------------------- |
| `@haruhi/shop`    | 商品卡、筛选、购物车、订单、后台表单 | 旧变量完整但散落 raw color、椭圆按钮、临时阴影和间距   | 已接入 `tokens/components/bridges` 与 `shop` scope |
| `@haruhi/art`     | 作品网格、筛选、上传、审核、弹窗     | 媒体比例、磨砂面板和上传表单较成熟，但颜色和状态分散   | 已接入 `tokens/components/bridges` 与 `art` scope  |
| `@haruhi/news`    | 导航、文章卡、搜索、发布后台         | 内容结构清楚，但历史页面和活动/积分/商店模块跨度较大   | 第二批，先迁移 NavBar 与 NewsCard recipe           |
| `@haruhi/exam`    | 试卷、题目、批阅、音频、编辑后台     | 表达很强但状态复杂，适合在基础规范稳定后验证动效和状态 | 第二批或第三批，先做答题页状态矩阵                 |
| `@haruhi/novel`   | 书架、阅读页、反馈                   | Tailwind 和阅读体验已有独立节奏，适合验证阅读规范      | 第三批，先接阅读 token 与 HeaderBrand              |
| `@haruhi/console` | 控制台、表格、审核、通知             | 中性后台，不需要角色化 expression                      | 独立按 compact density 接入                        |

## 1. 设计命题

SOS / Parallel Design System 的核心命题是：

> 一套骨架，五个平行世界。

它不是把五个站点统一套成同一种黄色主题，也不是再做第六套孤立视觉。系统统一的是结构、状态、可访问性、响应式和组件契约；系统允许变化的是每个业务站点的表达音量、主行动色、媒体比例、阅读字体和有限材质。

设计识别来自以下稳定线索：

- **Signal Yellow**：少量、稳定、可记忆的品牌信号。
- **Confident Rhythm**：强标题、稳定网格和足够留白。
- **Tactile Feedback**：明确但克制的按压、上浮和进度反馈。
- **Content Structure**：内容、数据和操作关系清楚，装饰不得改变阅读路径。
- **Project Fit**：规范必须能被现有 monorepo 渐进接入，而不是要求业务站点重写。

### 1.1 品牌标志

设计系统不重新定义站群 logo。项目默认使用现有 `haruhi-logo-192.png` 作为品牌标志资产，规范只约束它的使用方式：

- 导航和工具栏中的 logo 建议显示为 `40-48px`。
- 应用图标、PWA icon、分享图标优先使用原始 `192px` 资产。
- 不重绘、不描边、不套不规则几何外框、不拉伸变形。
- 不把 logo 当作页面装饰重复铺满；一个视口内只保留必要的品牌识别。
- 若某个 app 暂未放置该文件，先复制现有 `public/haruhi-logo-192.png`，不要临时生成新标志。

Header 中使用“logo + 标题文字”的组合：

- logo 放左侧，标题文字放右侧，二者间距使用 `12px` 或 `16px`。
- 常规导航使用 `44-48px` logo；紧凑工具栏可降到 `40px`。
- 标题文字可以按站点气质变化，例如 `春日团报`、`春日商城`、`长门有希的书架`，但行高保持 `1.2-1.45`。
- 双行组合第一行是站点名，第二行是短描述；第二行可省略，但不要超过一行。
- 标题文字可使用当前 Expression Mode 的 `--sos-link` 或 `--sos-text-primary`，不要给每个站点临时发明新色值。

## 2. 项目接入判断

这份规范的来源材料并不了解本仓库的多 app 结构；本仓库此前的说明也没有预设要接入一套正式设计系统。因此落地时不照搬任一边的表述，而以当前 monorepo 的真实约束为准：

- 现有业务 app 已经分别承担生产能力，不能把设计系统接入变成一次性重写。
- `packages/design-system` 继续承担框架无关 token 和 class contract；`packages/ui` 已启用 Vue wrapper MVP，但只封装稳定基础件。
- 外部 preview 中的 React 示例只作为规范演示参考，不作为本仓库必须采用的技术栈。
- 设计系统先稳定 token、布局原语、基础组件 anatomy、状态矩阵和文档页；业务卡片要等跨 app 形态稳定后再抽象。

## 3. 项目接入约束

当前仓库是多前端应用 monorepo，现有 app 以 Vue 为主，样式技术包含原生 CSS、SCSS、Tailwind 和 CSS 变量。设计系统的第一阶段交付不是共享 Vue 组件库，而是框架无关的 CSS Token、基础 class contract 和兼容 bridge。

正式约束：

- `packages/design-system` 是当前设计系统落地点，导出 CSS。
- `packages/ui` 是 Vue 基础组件 MVP，依赖 `@haruhi/design-system` 的 class contract，不重新定义视觉。
- 业务 app 可以先直接消费 CSS；引入 `@haruhi/ui` 必须以替换稳定基础件为目标，不为了单页特例新增 wrapper。
- 新页面优先消费 Semantic Token，例如 `--sos-bg-page`、`--sos-accent`、`--sos-text-primary`。
- 业务代码不得新增散落的 Hex、RGB、任意阴影、任意圆角和非 4px 网格间距。
- 现有页面迁移从 Token Bridge 开始，第一阶段不强制改变视觉。
- 设计规范页是静态文档 app，不依赖后端 API，不参与 RBAC。
- v0.2 先补齐 layout primitives、component contract、state matrix、responsive gates 和 UI MVP；业务 recipe 继续在规范页验证。

CSS 入口：

```js
import '@haruhi/design-system/tokens.css'
import '@haruhi/design-system/components.css'
import '@haruhi/design-system/bridges.css'
```

`bridges.css` 只用于渐进迁移。新页面不应把 bridge 变量当成新的设计系统变量。

### 3.1 Monorepo 接入矩阵

每个 app 先在入口样式和一个代表性业务路径接入 Token、布局原语和基础组件，再根据证据扩大范围。

| App               | Mode             | 入口                                      | 首批对象                                   | 验收路径                         |
| ----------------- | ---------------- | ----------------------------------------- | ------------------------------------------ | -------------------------------- |
| `@haruhi/news`    | `news`           | `apps/news/src/main.js` + `style.css`     | NavBar、SiteFooter、文章列表、后台发布表单 | 发布/阅读路径，列表与详情截图    |
| `@haruhi/shop`    | `shop`           | `apps/shop/src/main.js` + `assets/*.css`  | 商品卡、预售进度、订单状态、管理后台表单   | 下单、订单查看、库存编辑路径     |
| `@haruhi/art`     | `art`            | `apps/art/src/main.js` + `style.css`      | TopBar、FilterPanel、ArtworkGrid、上传审核 | 上传、筛选、审核、作品详情路径   |
| `@haruhi/novel`   | `library`        | `apps/novel/src/main.js` + `assets/*.css` | 书架、Reader、目录、阅读位置和反馈状态     | 书架进入阅读、目录跳转、阅读恢复 |
| `@haruhi/exam`    | `exam`           | `apps/exam/src/main.ts` + `style.css`     | HomeView、ExamPaper、QuestionRenderer      | 开始答题、恢复进度、提交和批阅   |
| `@haruhi/console` | `base + compact` | `apps/console/src/main.ts` + `style.css`  | Dashboard、Audit、Notify、Users 表格表单   | 审核、通知、用户管理路径         |

`console` 是管理后台，不强行套某个业务表达模式。它默认使用基础语义 token，可按页面密度加 `data-sos-density="compact"`。

### 3.2 包治理

| Package                 | Level             | 规则                                                                            |
| ----------------------- | ----------------- | ------------------------------------------------------------------------------- |
| `@haruhi/design-system` | L0 · CSS Contract | 新增 token/class 走 minor；删除或改名必须先 deprecate，并保留 bridge 删除计划。 |
| `@haruhi/ui`            | L1 · Vue Wrapper  | 新增 props/variant 前先更新规范页、状态矩阵和 a11y 证据。                       |
| `@haruhi/auth-ui`       | Auth Domain       | 继续维护登录/会话 UI；可以消费基础件，但不合并进通用 UI 包。                    |
| `@haruhi/api-client`    | Data Contract     | 不依赖视觉样式；只和内容数据格式、错误文案和状态字段对齐。                      |

版本规则：

- Minor：新增 token、class、wrapper、文档 section 或非破坏性 recipe。
- Patch：修复样式 bug、补状态、补文档、修响应式和 a11y 问题。
- Breaking：删除 token/class、改变 anatomy、改变 variant 语义；必须给迁移步骤和回滚边界。

### 3.3 接入 Playbook

最小迁移切片按以下顺序执行：

1. 入口导入：在目标 app 入口样式或 main 文件导入 `tokens.css` 和 `components.css`；旧变量多的页面才临时导入 `bridges.css`。
2. 根节点加 scope：给页面根容器加 `class="sos-scope"` 和对应 `data-sos-site`；`console` 可只用 `sos-scope` 加 `data-sos-density="compact"`。
3. 先换布局原语：用 `Stack / Inline / Cluster / Grid / Surface / MediaFrame` 处理间距、对齐和媒体比例。
4. 再换基础控件：按 `Button -> Badge -> Field -> Notice -> Progress -> EmptyState -> Card anatomy` 的顺序替换。
5. 保留业务 recipe：商品卡、作品卡、书封卡、试卷卡先留在业务 app，用真实数据验证后再评估抽象。
6. 提交证据：PR 附 390 / 768 / 1280px 截图、状态矩阵、a11y 检查、bridge 删除计划和回滚步骤。

入口 import 顺序：

```js
import '@haruhi/design-system/tokens.css'
import '@haruhi/design-system/components.css'
// 仅渐进迁移旧变量时加载
import '@haruhi/design-system/bridges.css'
```

业务站点根 scope：

```vue
<template>
  <main class="sos-scope" data-sos-site="shop">
    <RouterView />
  </main>
</template>
```

管理后台：

```vue
<template>
  <main class="sos-scope" data-sos-density="compact">
    <RouterView />
  </main>
</template>
```

Bridge 规则：

- 允许：旧页面变量多、需要先接入 token 但不改变外观；必须记录 owner、范围和删除计划。
- 禁止：新页面、新组件和新 wrapper 不能把 bridge 变量当成正式接口。
- 回滚：移除 app 入口 import 或 bridge 文件即可回退视觉接入，不改 API 和数据结构。

## 4. Token 架构

### 4.1 三层模型

1. **Primitive Token**：物理色值、字号、间距、圆角、阴影、时长。
2. **Semantic Token**：业务可读变量，例如 `--sos-bg-page`、`--sos-text-secondary`、`--sos-border-default`、`--sos-accent`。
3. **Expression Mapping**：通过 `data-sos-site="news | shop | art | library | exam"` 将 Semantic Token 映射到不同业务表达。

业务组件只读取 Semantic Token：

```css
.product-price {
  color: var(--sos-link);
}

.product-card {
  background: var(--sos-bg-surface);
  border-color: var(--sos-border-subtle);
}
```

### 4.2 Semantic Token 使用表

| 语义层     | Token                                    | 业务用途                             | 边界                          |
| ---------- | ---------------------------------------- | ------------------------------------ | ----------------------------- |
| 页面底色   | `--sos-bg-page`                          | 页面根背景、全局区段背景             | 不用于卡片内部或按钮          |
| 承载面     | `--sos-bg-surface`                       | 卡片、表单、弹层、Notice             | 不直接替代 `--sos-bg-page`    |
| 弱承载面   | `--sos-bg-subtle`                        | 筛选条、空状态、状态格、局部衬底     | 不用于强调状态                |
| 主文本     | `--sos-text-primary`                     | 标题、强数字、按钮内文、主要链接文本 | 不用于 disabled 文案          |
| 辅助文本   | `--sos-text-secondary`                   | 摘要、说明、meta、help text          | 不承载主要操作                |
| 边界       | `--sos-border-subtle / default / strong` | 分隔信息关系、承载面边界、焦点补强   | 焦点态不能只靠边框变浅        |
| 主行动     | `--sos-accent`                           | Primary button、当前 tab、关键进度   | 每个 Expression Mode 只有一个 |
| 品牌信号   | `--sos-signal`                           | 少量 Badge、编号、重要线索           | 不用于价格、错误或大面积 CTA  |
| 跨站状态   | `--sos-danger / --sos-success`           | 错误、成功、危险操作、完成反馈       | 不随站点气质改变语义          |
| 几何与深度 | `--sos-card-radius / --sos-card-shadow`  | 卡片、浮层、媒体承载面               | 不在业务里新增临时半径或阴影  |

### 4.3 Expression Mapping 边界

Expression Mode 可以映射主行动色、表面色、圆角、媒体圆角、卡片阴影和阅读字体。它不改变组件 anatomy、不改变状态意义、不改变业务信息结构。

| Mode      | 主行动表达   | 卡片圆角 | 深度             | 验收重点                         |
| --------- | ------------ | -------- | ---------------- | -------------------------------- |
| `news`    | 墨色主行动   | `8px`    | 默认平面         | 列表、长文和后台审核能快速扫读   |
| `shop`    | 行动蓝 CTA   | `18px`   | 柔和交易卡片阴影 | 商品图 1:1，价格、库存、进度常驻 |
| `art`     | 画廊青主行动 | `24px`   | 轻磨砂悬浮       | 作品占视觉主位，界面只承载       |
| `library` | 书脊琥珀     | `8px`    | 纸张阴影         | 阅读栈、书封比例和目录连续       |
| `exam`    | 批改红       | `12px`   | 试卷纸张阴影     | 题目、选项、倒计时和批阅稳定     |

需要站点气质时改 Expression Mapping，不在组件里写 `[data-sos-site='shop'] .product-card { color: #3478f6; }` 这类业务覆盖。

不要在业务组件中直接写物理值：

```css
.product-price {
  color: #3478f6;
}

.product-card {
  box-shadow: 0 12px 33px rgba(0, 0, 0, 0.14);
}
```

### 4.4 上下文注入

页面或局部区域通过 `data-sos-site` 注入表达模式：

```html
<main class="sos-scope" data-sos-site="shop">
  <button class="sos-button sos-button--primary">加入购物车</button>
</main>
```

密度使用独立属性，不参与主题语义：

```html
<section data-sos-site="exam" data-sos-density="compact">...</section>
```

`data-sos-density` 只允许影响控件高度、局部间距和列表密度，不允许改变颜色、状态、信息结构和组件层级。

## 5. 核心色彩

| Token              | 色值      | 用途                                        |
| ------------------ | --------- | ------------------------------------------- |
| `--sos-yellow-500` | `#FFC83D` | Signal Yellow，品牌记忆、重要标签、全局提示 |
| `--sos-sky-500`    | `#4B9FE8` | 天空蓝，导航、轻快链接、默认焦点            |
| `--sos-blue-500`   | `#3478F6` | 行动蓝，商城主行动与交易信息                |
| `--sos-teal-500`   | `#159A90` | 画廊青，美术部主行动与成功语义              |
| `--sos-amber-600`  | `#9D5D16` | 书脊棕，图书馆、档案、历史内容              |
| `--sos-red-600`    | `#C8171E` | 批改红，考试、危险、强提醒                  |
| `--sos-ink-950`    | `#171A22` | 标题、深色表面、团报主行动                  |
| `--sos-warm-50`    | `#FFFAF2` | 默认社团背景                                |
| `--sos-paper-100`  | `#F4ECDC` | 书籍、便签、试卷纸张                        |
| `--sos-mint-500`   | `#61C8A9` | 成功、完成、积极状态                        |

使用规则：

- 一个 Expression Mode 只有一个主行动色。
- Signal Yellow 不用于大段正文、错误信息或商城价格。
- Danger 与 Success 是跨站语义，不随主题重新定义意义。
- 焦点环必须比背景和控件边界更清晰，不能只改变阴影。
- 状态不得只依赖颜色，同时使用文字、图标、形状或位置。

## 6. 排版

字体栈：

- UI / 正文：`Inter, system-ui, -apple-system, PingFang SC, Microsoft YaHei, sans-serif`
- 阅读：`Noto Serif SC, Songti SC, STSong, Georgia, serif`
- 代码：`SFMono-Regular, Consolas, Liberation Mono, Menlo, monospace`

层级：

| 层级      | 尺寸      | 字重      | 行高        | 用途                      |
| --------- | --------- | --------- | ----------- | ------------------------- |
| `hero`    | `48-96px` | `850`     | `0.98-1.08` | 宣言式 Hero，每页最多一次 |
| `display` | `36-56px` | `800`     | `1.08`      | 频道标题、专题标题        |
| `section` | `28-36px` | `800`     | `1.20`      | 模块标题                  |
| `title`   | `18-22px` | `700-800` | `1.35`      | 卡片标题                  |
| `body`    | `16-17px` | `400`     | `1.65`      | 默认正文                  |
| `reading` | `17px`    | `400-500` | `1.90`      | 图书馆与长文阅读          |
| `caption` | `12-14px` | `600`     | `1.50`      | 日期、库存、帮助、元数据  |

排版约束：

- 正文不能依靠 `font-weight: 700+` 维持层级。
- 新闻摘要、图书正文使用阅读行高；商城和筛选界面可更紧凑。
- 纯大写只用于 2-8 个字符的英文标签。
- 价格、分数、进度应启用等宽数字：`font-variant-numeric: tabular-nums`。
- 规范页和新代码不使用负字距，不使用随视口连续缩放的字号。

## 7. 间距、网格与容器

采用 4px 网格。合法间距：

`4 / 8 / 12 / 16 / 20 / 24 / 32 / 40 / 48 / 64 / 80 / 96px`

推荐使用：

| 场景              | 间距          |
| ----------------- | ------------- |
| 控件内部          | `8-16px`      |
| 卡片内部          | `16-24px`     |
| 卡片之间          | `16-32px`     |
| 页面模块          | `48-96px`     |
| 移动端左右 gutter | `10-16px`     |
| 桌面内容容器      | 最大 `1248px` |
| 画廊与大网格      | 最大 `1472px` |
| 长文阅读          | 最大 `736px`  |

形状梯度固定为：

`4 / 8 / 12 / 18 / 24 / full`

Expression Mode 可以把 `--sos-card-radius` 映射到上述任一值，但不能新增 13px、17px 等临时半径。

### 7.1 Layout Primitives

v0.2 新增框架无关的布局原语。它们是 UI 库之前的地基，用于稳定间距、对齐、媒体比例和卡片承载面。

| Primitive          | 用途                     | 规则                                                     |
| ------------------ | ------------------------ | -------------------------------------------------------- |
| `.sos-stack`       | 纵向信息组               | 通过 `--sos-stack-gap` 控制间距，不在子元素上追加 margin |
| `.sos-inline`      | 行内操作组               | 允许换行，不拉伸 Badge、按钮和短标签                     |
| `.sos-cluster`     | 两端或多组对齐           | 用于卡片 footer、标题栏、统计行                          |
| `.sos-grid`        | 自适应卡片网格           | 通过 `--sos-grid-min` 和 `--sos-grid-gap` 控制列宽与间距 |
| `.sos-split`       | 双栏布局                 | 只用于稳定双栏，不用于强行制造 hero split                |
| `.sos-surface`     | 有边界的承载面           | 统一背景、边框、圆角和 elevation                         |
| `.sos-media-frame` | 媒体比例容器             | 使用 `data-ratio="1:1 / 4:3 / 3:4 / 2:3"` 声明比例       |
| `.sos-state-row`   | 数值、进度、库存等状态行 | 数字使用等宽；状态不能只靠颜色                           |

使用示例：

```html
<article class="sos-card">
  <div class="sos-media-frame" data-ratio="1:1">
    <img src="/product.png" alt="商品图" />
  </div>
  <div class="sos-card__body sos-stack">
    <div class="sos-cluster">
      <h3>朝比奈实玖瑠 fufu</h3>
      <strong>¥147</strong>
    </div>
    <p>达到目标后进入统一排产。</p>
  </div>
</article>
```

禁止在页面里临时写固定高度来“修正”卡片比例。卡片形态应由 `MediaFrame` 比例、卡片 anatomy 和容器宽度共同决定。

## 8. 媒体几何

| 模式      | 媒体几何                          |
| --------- | --------------------------------- |
| `news`    | 自由横图或无图，边角 4px          |
| `shop`    | 商品主图 1:1，卡片内媒体圆角 14px |
| `art`     | 4:3、3:4 或原图比例，作品优先     |
| `library` | 书封 2:3                          |
| `exam`    | 纸张卡，内容区不裁切              |

图片、书封、文章标题和试卷内容是第一视觉对象。角色图、贴纸和装饰不能替代业务内容。

## 9. Elevation

系统只有三个正式层级：

| 层级    | 用途                          |
| ------- | ----------------------------- |
| `Flat`  | 页面、新闻正文、列表、分隔区  |
| `Card`  | 商品、作品、浮层菜单、书封    |
| `Float` | Dialog、Popover、拖拽中的卡片 |

额外表达性效果：

- `--sos-shadow-paper`：只用于书封与试卷纸张。

禁止为每个组件临时发明不同阴影值。

## 10. Motion

| Token  | 时长    | 用途                         |
| ------ | ------- | ---------------------------- |
| `fast` | `120ms` | 按压、图标、颜色反馈         |
| `base` | `200ms` | hover、focus、Popover        |
| `slow` | `420ms` | 页面进入、大图切换、进度变化 |

规则：

- Button `active` 最大位移 1px。
- Card hover 最大上浮 4-6px。
- 关键数据、计时、考试选择不得延迟显示。
- `prefers-reduced-motion: reduce` 下移除位移、旋转、视差与平滑滚动。

## 11. Expression Modes

### 11.1 `news`，春日团报

- 背景：中性灰白。
- 主行动：墨色。
- Signal Yellow：只作左边线、重要标记、专题编号。
- 圆角：4-8px。
- Elevation：默认无阴影。
- 长文可使用衬线字体。
- 禁止：玻璃、彩色大背景、倾斜正文。

### 11.2 `shop`，春日商城

- 背景：蓝灰浅底。
- 主行动：行动蓝。
- Signal Yellow：新品、限时、团长推荐。
- 卡片：18px，商品图优先。
- 交易信息必须高可读、数字等宽。
- 预售进度为正式共享组件。
- 禁止：把品牌黄用于所有价格和购买按钮。

### 11.3 `art`，春日美术部

- 背景：青绿色氛围或作品衍生背景。
- 主行动：画廊青。
- 卡片：24px，高质量媒体优先。
- 允许：一层磨砂、一层柔和悬浮、轻微透视。
- 禁止：每层容器都玻璃化、模糊正文、背景图抢作品。

### 11.4 `library`，长门有希的书架

- 背景：米纸色。
- 主行动：书脊棕。
- 书封：2:3，轻微书脊与纸张阴影。
- 正文：衬线字体、最大 736px、1.9 行高。
- 禁止：重木纹、颗粒超过 5% 对比度、卡片大幅悬浮。

### 11.5 `exam`，春日试卷中心

- 背景：木桌舞台。
- 表面：高对比纸张。
- 主行动：批改红。
- 辅助角色色：学生蓝。
- 允许：纸张层级、批注、分数标记。
- 试题、选项、倒计时和分数区域不得倾斜。
- 移动端把舞台效果降级，优先答题效率。

## 12. 内容与数据表达

设计系统约束的不只是颜色和组件，也包括页面承载的信息结构。标题、状态、价格、日期、进度和空状态必须来自真实业务字段，并在移动端、加载态和错误态保持可读。

### 12.1 内容原则

| 原则             | 规则                                                                 |
| ---------------- | -------------------------------------------------------------------- |
| 真实对象先于装饰 | 标题、日期、价格、进度、作者、状态来自业务字段，不用虚造站点或口号。 |
| 一条信息一个职责 | 标题识别对象，摘要解释差异，状态说明阶段，操作只指向下一步。         |
| 关键数据常驻     | 价格、库存、倒计时、审核状态、答题进度不能只在 hover 或浮层出现。    |
| 语气具体克制     | 错误、空状态和成功反馈说清原因与下一步，不写泛泛口号。               |

### 12.2 数据格式

| 类型 | 示例                | 规则                                                         |
| ---- | ------------------- | ------------------------------------------------------------ |
| 日期 | `2026-06-23`        | 列表、审核、订单默认使用稳定日期；相对时间只用于动态流。     |
| 价格 | `¥ 147`             | 金额使用等宽数字；品牌黄不作为价格色，折扣和总价必须可比较。 |
| 进度 | `126/200 · 63%`     | 预售、答题、迁移进度同时保留当前值、目标值或百分比。         |
| 库存 | `现货 12 / 预售中`  | 库存状态用文字常驻，不只显示色点或图标。                     |
| 作者 | `显示名 / 匿名投稿` | 投稿、审核和系统内容要区分来源；匿名不是空字段。             |

### 12.3 状态文案

| 状态        | 规则                                                                 |
| ----------- | -------------------------------------------------------------------- |
| Loading     | 保留原布局骨架和上下文标题；不要用大面积转场替代关键数据位置。       |
| Empty       | 说明为空原因，例如筛选无结果、权限不足或尚未创建，并提供真实下一步。 |
| Error       | 说明失败原因、当前内容是否已保留，以及重试、返回或联系支持的动作。   |
| Unavailable | 下架、维护、暂停售卖和审核中必须有不同文案，不能统一写“暂无”。       |

### 12.4 五个站点的信息主语

| Site      | 必备字段                           | 规则                                                  |
| --------- | ---------------------------------- | ----------------------------------------------------- |
| `news`    | 标题、摘要、来源、日期、专题       | 阅读流先证明信息来源和发布时间，再考虑专题视觉。      |
| `shop`    | 商品名、价格、库存、进度、订单状态 | 交易信息必须可比较、可追踪，不能被装饰和 hover 隐藏。 |
| `art`     | 作品、作者、版权状态、审核状态     | 作品是主语，界面只补充作者、筛选、审核和操作。        |
| `library` | 书名、作者、目录、阅读位置         | 长文和书目需要连续阅读线索，不能只展示封面。          |
| `exam`    | 题目、选项、倒计时、分数、恢复状态 | 考试信息不可延迟出现，移动端优先答题效率。            |

## 13. 基础组件

### 13.0 UI 库准入边界

`packages/ui` 已开始做 Vue MVP，但只封装稳定基础组件，不封装业务卡片。每个进入 UI 库的组件必须持续满足：

- anatomy 稳定：DOM 结构、slot、可访问名称和 class contract 清楚。
- variants 有限：不能把页面特例做成组件 variant。
- states 完整：default、hover、focus-visible、disabled、loading、empty、error 至少有定义。
- responsive 明确：320 / 390 / 768 / 1280px 下不溢出、不遮挡、不依赖 hover。
- 视觉证据可复查：规范预览页或 story 能展示状态矩阵。

第一批已进入 UI MVP：`Button`、`Badge`、`Field`、`Notice`、`Progress`、`Card`、`EmptyState`、`HeaderBrand`、`Stack / Inline / Cluster / Grid / Split / Surface / MediaFrame`。  
暂不进入 UI 库：`NewsArticleCard`、`ShopProductCard`、`ArtworkCard`、`LibraryBookCard`、`ExamPaperCard`。它们先作为 recipe 验证。

### 13.0.1 组件成熟度分级

UI 库不是所有重复 UI 的收纳箱。v0.2 使用四级成熟度管理抽象边界：

| Level | 名称                   | 落点              | 准入规则                                                                  |
| ----- | ---------------------- | ----------------- | ------------------------------------------------------------------------- |
| L0    | Token / Class Contract | CSS 包            | 颜色、间距、圆角、布局原语和基础 class；业务迁移可以先 CSS-first 接入。   |
| L1    | Primitive Wrapper      | `@haruhi/ui`      | 跨站重复、语义稳定、状态明确；wrapper 只输出 class、props、slot 和 aria。 |
| L2    | Composition Recipe     | 规范页 + 业务 app | 新闻卡、商品卡、作品卡、书封卡、试卷卡先用真实数据验证，不进入 UI 包。    |
| L3    | Product Component      | 未来评估          | 至少三个页面共享同一信息结构、状态机和数据契约后，才评估升级。            |

当前判断：

| 对象                               | 分级                 | 原因                                                                      |
| ---------------------------------- | -------------------- | ------------------------------------------------------------------------- |
| `SosButton / SosBadge / SosField`  | L1 · UI Wrapper      | 跨站重复、语义稳定、状态明确，可以由 Vue wrapper 输出统一 class。         |
| `SosCard / SosMediaFrame`          | L1 · Anatomy Wrapper | 只封装边界、媒体比例和 body/footer 槽位，不决定新闻、商品或作品信息结构。 |
| `SosStack / Inline / Grid / Split` | L1 · Layout Wrapper  | 允许 gap、min、ratio 等布局参数；不暴露颜色和材质。                       |
| `ShopProductCard / ArtworkCard`    | L2 · Recipe          | 需要真实数据、媒体比例和流程状态继续验证，暂不进入共享 UI 包。            |
| `CheckoutRail / ExamQuestion`      | L3 · Candidate       | 只有当多页面共享同一数据契约和状态机时，才进入产品组件评估。              |

### 13.0.2 API 设计守则

- Props 只暴露稳定语义：`variant`、`size`、`tone`、`ratio`、`gap`、`selected`、`loading`。
- Slot 对应 anatomy 槽位；不能用任意 slot 绕过结构约束。
- 组件不接受 `color`、`shadow`、`radius` 等视觉 props；这些由 Semantic Token 决定。
- 状态 props 必须同步 `aria`、`disabled`、`aria-busy`、`aria-invalid` 等可访问性证据。
- 新增组件或 variant 前，必须先更新规范页、状态矩阵和响应式截图证据。

| 组件       | Anatomy                              | Variants                                      | States                                                | 规则                                            |
| ---------- | ------------------------------------ | --------------------------------------------- | ----------------------------------------------------- | ----------------------------------------------- |
| Button     | `button / a.sos-button`              | `primary / secondary / ghost / danger`        | `hover / active / focus-visible / disabled / loading` | 只承载明确命令；图标按钮以后单独封装            |
| Badge      | `span.sos-badge`                     | `default / accent / solid / outline / signal` | `default / selected / disabled by parent`             | 只表达状态、分类或短标签                        |
| Field      | `label -> control -> help/error`     | `input / textarea / select`                   | `focus / disabled / error / help / required`          | Label 不被 placeholder 替代                     |
| Card       | `card -> media/body/footer`          | `flat / raised / interactive / recipe`        | `hover / focus-within / selected / loading / empty`   | 内部节奏由 anatomy 管理，不在页面临时改 padding |
| Notice     | `icon -> content -> optional action` | `info / warning / success / danger`           | `dismissible / action / compact`                      | 用于系统提示，不用于普通营销文案                |
| Progress   | `meta -> track -> fill`              | `default / success / danger`                  | `0 / in-progress / complete / error`                  | 数值常驻显示，不能只依赖颜色或 hover            |
| EmptyState | `icon -> title -> copy -> action`    | `default`                                     | `empty / no result / permission missing`              | 说明原因，并给出下一步动作                      |

每个进入 UI MVP 的组件在规范页中必须有组件用法卡，至少包含：

- Anatomy：root、slot、状态属性和关键子元素。
- Use：适用场景和组件职责。
- Do：推荐写法、文案和状态证据。
- Don’t：禁止把组件挪作其他语义，禁止页面特例变成 variant。
- Example：CSS-first HTML 和 Vue wrapper 代码至少给出一种。

### 13.1 Button

Variants：`primary / secondary / ghost / danger`

States：`rest / hover / pressed / focus-visible / loading / disabled`

规则：

- 默认高度 44px；关键 CTA 48px。
- 标签使用动词开头。
- Loading 保持按钮宽度。
- Disabled 不能成为解释错误的唯一方式。
- 同一视口只允许一个最高行动层级 CTA。
- 不用于普通状态标签、长句提示或卡片标题。

### 13.2 Badge

Variants：`neutral / accent / solid / outline / signal`

Badge 只承载短状态或类别，不承载句子。Signal Badge 一屏不超过 3 个。

Badge 不处理点击行为。需要点击、提交或导航时使用 Button、Link 或 Tab。

### 13.3 Field / Input / Select / Textarea

稳定 anatomy：`Label -> Control -> Help / Error`

规则：

- Label 永不只用 placeholder 替代。
- Focus 为清晰边界加外环。
- Error 同时显示图标、文字和颜色。
- 移动端输入类型与键盘类型匹配。
- 一个 Field 只绑定一个主要输入意图，复合表单用 Stack 分组。

### 13.4 Tabs / FilterBar

- 支持键盘方向键。
- 移动端横向滚动，不强行缩小字号。
- 选中态至少有两种线索：背景、下划线、字重或图标。

### 13.5 Notice / Toast / Dialog

- Notice：常驻上下文信息。
- Toast：动作结果，自动消失前可暂停。
- Dialog：需要决策或高风险确认。
- 不用 Toast 承载需要复制、比较或长期阅读的信息。
- Notice 必须保留标题和正文，不能只显示一个彩色图标。

### 13.6 Progress

- `determinate`：预售、下载、答题完成度。
- `indeterminate`：无法估算的加载。
- 必须提供文字数值或可读描述。
- 色彩含义不得随站点变化。
- 错误态必须说明失败位置、原因或下一步。

### 13.7 EmptyState

- 空状态不是装饰区块，必须解释为什么为空。
- 至多给一个主行动和一个次行动，避免把空状态做成营销页。
- `no result` 优先给清除筛选、返回全部或换关键词。
- `permission missing` 必须说明需要登录、验证或权限申请，不能只显示锁图标。

### 13.8 Card

- Card 是内容容器，不是业务组件边界。
- 必须让标题、关键状态和主信息常驻。
- 图片、作品、书封和试卷纸张比例交给 `MediaFrame` 或业务 recipe。
- 不允许通过页面局部 CSS 临时改 `sos-card__body` padding 来修单张卡片。

## 14. 业务 Compositions

| Composition       | Anatomy                                                                | 关键要求                                         |
| ----------------- | ---------------------------------------------------------------------- | ------------------------------------------------ |
| `NewsArticleCard` | `Type -> Title -> Summary -> Tags -> Date`                             | 支持无图、横图、专题、置顶；默认无阴影           |
| `ShopProductCard` | `Media -> Title + Price -> Description -> Presale -> Category + Stock` | 商品图必须保留；预售信息不能只在 hover 出现      |
| `ArtworkCard`     | `Artwork -> Overlay Title / Tags -> Artist / Action`                   | Overlay 需有足够 Scrim；详情不能被玻璃层永久遮挡 |
| `LibraryBookCard` | `Cover -> Title -> Author`                                             | 书名与作者必须是独立文本，不能只存在于图片内     |
| `ExamPaperCard`   | `Status -> Title -> Meta -> Progress / Score -> Action`                | 视觉可模拟纸张，交互必须稳定、可聚焦、可恢复     |

## 15. 页面 Patterns

页面模式不是新组件库，而是把基础组件组织成真实流程的结构协议。它们可以在不同站点自然变声，但信息顺序、状态证据和断点行为必须一致。

| Pattern       | Anatomy                                  | 验收要求                                                         |
| ------------- | ---------------------------------------- | ---------------------------------------------------------------- |
| AppShell      | `BrandLockup / Nav / Account / Footer`   | 全站 Header 使用同一 logo + 标题组合，站点差异只改变文字和语义色 |
| ChannelHeader | `Title / Description / Primary / Meta`   | 频道名是首屏信号，说明当前页面任务，不写泛营销口号               |
| FilterBar     | `Search / Category / Sort / ResultCount` | 筛选和结果数量同区，移动端可换行或折叠，但顺序不反转             |
| ContentGrid   | `Grid / RecipeCard / Empty / Loading`    | 列数由 `--sos-grid-min` 驱动，不压缩标题、价格或按钮             |
| DetailLayout  | `Main / Rail / RelatedAction`            | 双栏只在空间足够时出现；窄屏 Rail 下移                           |
| StickyAction  | `State / PrimaryAction / SafeArea`       | 移动端可 sticky，但不能遮挡内容或替代页面状态说明                |
| SystemState   | `Notice / EmptyState / Progress / Error` | 跨站统一状态体验，只让 tone 和上下文文案变化                     |

业务 app 迁移页面模式时，优先用 `Stack / Inline / Cluster / Grid / Split / Surface / MediaFrame` 组合，不为单页新建布局组件。

## 16. 响应式

| 范围         | 关键策略                                                                        |
| ------------ | ------------------------------------------------------------------------------- |
| `< 640px`    | 单列；移动导航；44-48px 控件；主操作可 Sticky Bottom；减少背景纹理和 hover 依赖 |
| `640-960px`  | 2 列卡片；筛选折叠；保留频道导航；详情 Rail 可下移                              |
| `960-1280px` | 3-4 列；完整过滤；详情双栏                                                      |
| `> 1280px`   | 内容容器封顶；画廊可 4-5 列；不无限拉宽正文                                     |

卡片网格按内容自然降列，不允许把卡片压缩到标题、价格或按钮无法阅读。

### 16.1 状态矩阵

所有进入 UI 库的组件必须覆盖以下状态。业务 recipe 至少覆盖受影响状态。

规范页或 Story 必须展示真实 DOM 状态，不能只在表格里写状态名称。为了截图和视觉回归，可在文档预览中使用 `data-state="hover"`、`data-state="focus"` 强制展示伪类状态；业务代码不得依赖这些属性完成交互逻辑。

| 状态           | 验收要求                                   |
| -------------- | ------------------------------------------ |
| Default        | 信息完整、操作可用、无悬浮依赖             |
| Hover          | 只增强可点击感；不能出现新关键信息         |
| Focus-visible  | 键盘焦点必须比边框清晰，不能只靠阴影       |
| Disabled       | 降低可操作性但保留 label，必要时说明原因   |
| Loading        | 锁定重复提交，保留当前上下文和进度反馈     |
| Empty          | 说明为空原因，并给出下一步动作             |
| Error          | 同时使用文字、位置和颜色；不能只把边框改红 |
| Reduced Motion | 位移、旋转和视差降级；信息时序不变         |
| Forced Colors  | 边框、文字、焦点和可点击区域仍可辨认       |

当前 UI MVP 的状态证据范围：

- `Button`：default、hover、focus、loading、disabled。
- `Badge`：default、accent、selected、signal、disabled。
- `Field`：default、focus、error、disabled。
- `Notice`：info、success、warning、danger。
- `Progress`：active、complete、error、zero。
- `Card`：default、hover、selected、loading。
- `EmptyState`：no data、no result、permission missing。

### 16.2 断点 gates

| Gate     | 规则                                               |
| -------- | -------------------------------------------------- |
| `320px`  | 无横向溢出；核心按钮和输入不小于 44px              |
| `390px`  | 移动主路径可完成；卡片不依赖 hover                 |
| `768px`  | 工具组换行后仍保持顺序；详情 Rail 可下移           |
| `1024px` | 双栏布局可用；侧栏 sticky 不遮挡内容               |
| `1280px` | 常规内容锁定 1248px；不无限拉宽正文                |
| `1440px` | 只增加留白、网格列数或媒体展示面积，不放大基础字号 |

## 17. 可访问性

可访问性是上线门槛，不是发布前最后修饰。涉及 UI MVP、页面模式或业务 recipe 的 PR 必须提供可复查证据。

| Gate           | 验收要求                                                       |
| -------------- | -------------------------------------------------------------- |
| Text Contrast  | 普通文本达到 WCAG AA；弱文本不承担唯一状态证据                 |
| Keyboard       | 全流程可键盘操作；Tab 顺序与视觉顺序一致；`focus-visible` 明确 |
| Touch Target   | 可点击元素最小 44px；核心 CTA 48px；相邻目标有足够间距         |
| State Evidence | 状态不只靠颜色，必须有文字、图标、位置或形状证据               |
| Dialog / Toast | Dialog 管理焦点、关闭与背景 inert；Toast 使用适当 `aria-live`  |
| Media          | 图片提供有意义的 alt；装饰图使用空 alt                         |
| Reduced Motion | 位移、旋转和视差降级；信息时序和操作反馈不改变                 |
| Forced Colors  | 边框、文字、焦点和可点击区域仍可辨认                           |
| Zoom           | 200% 缩放不遮挡正文、表单、按钮和关键状态                      |
| Locale         | 中文页面设置正确 `lang="zh-CN"`                                |

## 18. 迁移策略

### Phase 0：Inventory

统计旧变量、组件、页面模式和状态。建立视觉回归基线。

### Phase 1：Token Bridge

引入设计系统 token，再按需加载兼容桥，把旧变量指向新 Semantic Token。第一阶段不改变视觉。

```css
[data-sos-site='shop'] {
  --primary-color: var(--sos-accent);
  --primary-dark: var(--sos-accent-hover);
  --text-main: var(--sos-text-primary);
  --text-secondary: var(--sos-text-secondary);
  --bg-body: var(--sos-bg-page);
  --bg-white: var(--sos-bg-surface);
  --border-color: var(--sos-border-default);
}
```

### Phase 2：Shared Primitives

按 `Button -> Badge -> Input -> Tabs -> Notice -> Progress` 的顺序迁移。优先处理状态复杂、跨站重复的组件。

### Phase 3：Business Compositions

每个站点迁移一张代表性业务卡片，以真实数据验证表达模式，而不是只在孤立组件里看效果。

### Phase 4：Page Patterns

统一 Header、Container、FilterBar、DetailLayout、StickyAction 和系统状态。首页编排保持业务独立。

### Phase 5：QA and Removal

完成视觉回归、键盘、读屏、响应式和 Reduced Motion 测试后，删除旧变量和重复 CSS。

### 18.1 PR 证据包

涉及设计系统接入、UI MVP、页面模式或业务 recipe 的 PR 必须提供可复查证据，而不是只写“已适配设计系统”。

| 证据组      | 内容                                                                     | 载体                |
| ----------- | ------------------------------------------------------------------------ | ------------------- |
| Scope       | app、路由、组件、旧变量 bridge、未迁移项和回滚入口                       | PR 描述 + diff 链接 |
| Visual      | 390 / 768 / 1280px 前后截图，检查首屏主操作、关键数据和信息层级          | Playwright 截图     |
| Interaction | 键盘路径、hover、focus-visible、loading、disabled、empty、error 可见证据 | 状态矩阵截图        |
| A11y        | 触控目标、读屏名称、aria-live、Reduced Motion、Forced Colors、200% Zoom  | 检查记录 + 截图     |
| CSS Debt    | 无 raw hex、临时阴影、临时圆角；bridge 变量有 owner 和删除计划           | `rg` 输出 + 说明    |
| Rollback    | 如何移除入口 import 或 bridge 文件回滚视觉接入，不影响 API 和数据结构    | 回滚步骤            |

### 18.2 合并门槛

| Level  | 判断     | 规则                                                                        |
| ------ | -------- | --------------------------------------------------------------------------- |
| Pass   | 可以合并 | 核心路径可完成；无横向溢出；截图、状态、a11y 和 token 证据齐全。            |
| Review | 设计复核 | 存在轻微视觉差异，但不影响核心任务、状态证据、响应式和回滚边界。            |
| Block  | 不得上线 | 核心路径断裂、移动端依赖 hover、状态只靠颜色、关键数据消失或新增 CSS 债务。 |

### 18.3 自动化检查

设计系统相关 PR 至少运行本地基线命令：

```sh
pnpm check:design-system
```

它会执行 Prettier、规范页 ESLint、规范页 build 和 `@haruhi/ui` typecheck。

涉及规范页布局、断点、内容展示或组件状态的 PR，需要先启动本地预览，再运行浏览器断点检查：

```sh
pnpm dev:design-system -- --host 127.0.0.1 --port 5206
pnpm check:design-system:browser
```

浏览器检查默认访问 `http://127.0.0.1:5206/design-system/`，并验证关键 section 在 `390 / 768 / 1280 / 1440px` 下没有横向溢出。

## 19. Do / Don’t

Do：

- 用一个强品牌时刻建立记忆，其他区域保持克制。
- 让图片、书封、文章标题和试卷内容成为第一层。
- 通过 Semantic Token 切换模式。
- 保留现有站点真正独特的内容展示方式。
- 在真实页面和真实数据中验收组件。
- 先解决状态、触控与阅读问题，再增加装饰。

Don’t：

- 不要把所有网站改成黄色暖白卡片。
- 不要在业务代码硬编码颜色、阴影、圆角和间距。
- 不要用角色图替代品牌系统。
- 不要用玻璃拟态覆盖所有面板。
- 不要让移动端依赖 hover。
- 不要移动正文、表单或考试选项来制造动感。
- 不要同时出现多个同等级主按钮。
- 不要把主题切换当成可访问性的替代方案。

## 20. 上线验收

上线验收不使用“看起来接了设计系统”的抽象目标，而要求能复查的证据：

- [ ] PR 说明列出接入的 app、页面、旧变量 bridge 范围和明确未迁移项。
- [ ] 涉及的真实业务路径可完成：news 发布或阅读、shop 下单、art 上传或审核、library 阅读、exam 答题，按变更范围至少覆盖一个核心路径。
- [ ] 被改页面保存 390 / 768 / 1280px 截图，对比接入前后信息层级、首屏主操作和关键数据位置。
- [ ] 新增样式只使用设计系统 token；确需保留的 bridge 变量有 owner 和删除计划。
- [ ] hover、focus-visible、loading、disabled、empty、error 在受影响组件中可见且有明确文案。
- [ ] 键盘可到达主要操作，焦点不丢失；状态变化不能只靠颜色表达。
- [ ] 320px 无横向溢出；核心按钮和输入控件不小于 44px；购买、提交、继续答题等动作不依赖 hover。
- [ ] 回滚边界清楚：能通过移除 app 入口 import 或 bridge 文件回滚视觉接入，不影响后端 API 和数据结构。
