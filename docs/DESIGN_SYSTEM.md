# SOS / Parallel Design System

> **一套脊柱，五个平行世界。**
> 凉宫春日应援团的统一设计系统：团报、商城、美术部、书库、考场共享同一套结构、状态、可访问性与组件契约，各自保留自己的气质。

系统**统一**的是：结构、节奏、组件解剖、状态、焦点与无障碍、响应式。
系统**允许变化**的是：每个站点的配色、局部强调、媒体处理、阅读字体与材质。
`data-sos-site` 是「换气质」而不是「整站换肤」。

## 交互式规范页是唯一权威

```sh
pnpm dev:design-system     # → http://localhost:5206/design-system/
```

规范页用真实组件搭建（dogfood `@haruhi/ui`），顶栏可**实时切换表达模式 / 密度 / 明暗**，覆盖每个组件、状态、五个平行世界与组合范式。**组件的真实外观以规范页为准**；本文是参考与原则，不重复粘贴像素值。

---

## 1. 架构

### 1.1 包与分层

| 包                      | 层  | 职责                                                                                                                              |
| ----------------------- | --- | --------------------------------------------------------------------------------------------------------------------------------- |
| `@haruhi/design-system` | L0  | CSS-first 契约：`tokens.css`（token）、`components.css`（基础组件 class）、`recipes.css`（业务卡片）、`bridges.css`（旧站变量桥） |
| `@haruhi/ui`            | L1  | Vue 封装（~41 组件 + `useToast`）。只输出 L0 的 class，不重定义视觉                                                               |
| `@haruhi/ui/recipes`    | L2  | 业务卡片组件：`SosArticleCard` / `SosProductCard` / `SosArtworkCard` / `SosBookCard` / `SosExamCard`                              |
| `@haruhi/auth-ui`       | —   | 全站共享账号 UI（登录/注册/找回/资料/设置/账户菜单），构建于上面两层之上                                                          |

入口：

```js
import '@haruhi/design-system/tokens.css' // 仅 token（浅层接入用）
import '@haruhi/design-system/components.css' // token + 基础组件（已 @import tokens + recipes）
import '@haruhi/design-system/bridges.css' // 旧站变量桥，仅渐进迁移时加载
import { SosButton, SosCard /* … */ } from '@haruhi/ui'
import { SosProductCard } from '@haruhi/ui/recipes'
```

### 1.2 三层 token 模型

```
Primitive（物理色板，不在业务里直接用）
  → Semantic（语义变量，业务只消费这层）
    → Expression（站点表达，[data-sos-site] 只覆盖语义层）
```

业务代码**只写语义变量**（`var(--sos-link)`），不写物理色值，也不直接读 primitive。

---

## 2. 设计调性与品牌

基础调性：**明亮、可靠、有社团感，但不幼稚**。像一个认真运行的社团工作台——真实内容清楚、关键动作可靠、状态反馈明确，保留少量 Haruhi Fan Club 的识别线索。不是泛用蓝白后台，也不是二次元贴纸包。

稳定识别线索：

- **Signal Yellow（`#fbbf24`）**：少量、稳定、可记忆的品牌信号——用于 news 的专题/置顶线索、eyebrow 的小色条、重点标记。**不要**扩成主按钮、价格或整页主题。
- **Confident Rhythm**：强 display 标题、稳定 4px 网格、足够留白。
- **Tactile Feedback**：明确但克制的按压、上浮与进度反馈。
- **Content Structure**：内容、数据、操作关系清楚，装饰不改变阅读路径。
- **真实内容优先**：商品图、作品、文章、题目、书目是第一视觉对象；背景/纹理/玻璃/阴影只服务内容层级。

**品牌标志**：不重绘 logo，使用现有 `haruhi-logo-192.png`。Header 用「logo + 标题文字」锁头，logo `40–48px`（紧凑 `40px`），标题行高 `1.2–1.45`，颜色用当前模式的 `--sos-text-primary` / `--sos-link`，不临时发明色值。

---

## 3. Tokens

完整清单见 `tokens.css`；下面是结构与约定。

### 3.1 语义颜色（业务只用这些）

| Token                                                              | 用途                                   |
| ------------------------------------------------------------------ | -------------------------------------- |
| `--sos-bg-page` / `-surface` / `-subtle` / `-muted` / `-strong`    | 页面 / 承载面 / 弱背景 / 更弱 / 强背景 |
| `--sos-text-primary` / `-secondary` / `-tertiary` / `-disabled`    | 主 / 次 / 弱 / 禁用文本                |
| `--sos-border-subtle` / `-default` / `-strong`                     | 三档边界                               |
| `--sos-accent` / `-hover` / `-soft` / `-contrast` / `-2`           | 当前模式强调色 + 辅色                  |
| `--sos-link` / `-hover`，`--sos-focus`，`--sos-signal`             | 链接 / 焦点 / 品牌信号                 |
| `--sos-danger` / `-success` / `-warning` / `-info`（各带 `-soft`） | 跨站点一致的状态色                     |

### 3.2 排版

- **流体字阶**：标题用 `clamp()` 随视口平滑缩放（`--sos-text-xl … -hero`），小字固定（`-2xs … -lg`）。无断点跳变。
- **字重**：`--sos-weight-regular(400) … -black(900)`。
- **字距**：`--sos-tracking-tighter … -wider`（标题收紧、eyebrow 大写放宽）。
- **行高**：`--sos-leading-tight … -reading`。
- **字体族**：`--sos-font-sans`（界面）/ `--sos-font-reading`（衬线长文）/ `--sos-font-mono`（数据等宽）。display 与 reading 可被站点覆盖（library/exam 用衬线标题）。

### 3.3 间距 · 圆角 · 层次 · 动效 · 层级

- **间距**：4px 基准网格 `--sos-space-0 … -32`。不写非网格间距。
- **圆角**：`--sos-radius-xs(4) / sm(8) / md(12) / lg(18) / xl(24) / 2xl(32) / full`。
- **层次**：双层柔光 `--sos-shadow-hairline / -xs / -sm / -card / -float / -overlay`（环境光 + 主光，克制不发黑）。
- **动效**：`--sos-duration-fast/base/slow` + `--sos-ease-out/-standard/-spring`；`prefers-reduced-motion` 自动降级。
- **焦点环**：统一用 `--sos-ring`（`focus-visible`），不在业务里自定义 outline。
- **浮层层级**：`--sos-z-sticky(1100) < -dropdown(1200) < -overlay(1300) < -modal(1400) < -popover(1500) < -toast(1600) < -tooltip(1700)`。

---

## 4. 五个平行世界（Expression Modes）

每个模式只覆盖语义层（配色 / 表面 / 圆角 / 字体 / 肌理），组件解剖不变。**各站气质提炼自「接入设计系统之前的原始站点」的优点，再统一进同一套脊柱**——不是把五个站染成同一种黄，也不是照搬原站的艳度。

| Mode      | 站点           | 气质（提炼自原站）                                                       | 强调 / 信号              | 禁止                       |
| --------- | -------------- | ------------------------------------------------------------------------ | ------------------------ | -------------------------- |
| `news`    | 春日团报       | 纯净白纸编辑部、毛笔报头；锐利小圆角、无卡片阴影、正文衬线               | 墨色 accent / 黄色信号   | 彩色大背景、玻璃化正文     |
| `shop`    | 春日商城       | 晴空蓝（沿用原站 `#3498db` 一脉）、圆润亲和、柔软投影；价格/库存优先     | 行动蓝 / 黄色信号        | 整站染蓝、隐藏价格库存     |
| `art`     | 美术部         | 梦幻渐变氛围（克制的粉/青柔光）、玻璃感内高光面板、大圆角；作品主位      | 青绿 + 粉色辅 / 黄色信号 | 渐变过艳、背景抢作品       |
| `library` | 长门有希的书架 | 温润奶油纸、衬线标题、极简留白；书封与连续阅读线索                       | 琥珀 / 黄色信号          | 全站纸纹铺底影响阅读       |
| `exam`    | 春日试卷中心   | 暖木米白试卷、答题横格（卡片上）、阅卷红 + 学生藏蓝 + 金色信号；衬线标题 | 阅卷红 / 金色信号        | 木桌铺满页面、动效干扰答题 |

> 历史教训：`exam` 原站曾混用 B 站粉 `#FB7299` 与小红书红 `#FF2442` 两套不搭的红——设计系统的价值之一就是**把它统一成一套「阅卷红 + 藏蓝 + 金」语言**，这正是「抽共性 + 改掉原设计不合理处」。

**明暗与密度**：`data-sos-theme="dark"` 只重映射语义层；`data-sos-density="compact|spacious"` 只调控件节奏。建议把 `data-sos-theme` 挂在 `documentElement` 上，让 Teleport 到 body 的弹层/toast 也继承。

---

## 5. 组件

### 5.1 成熟度分级（抽象边界）

| Level | 名称                   | 落点                                 | 准入                                                             |
| ----- | ---------------------- | ------------------------------------ | ---------------------------------------------------------------- |
| L0    | Token / Class Contract | `@haruhi/design-system`              | 颜色、间距、圆角、布局原语、基础 class                           |
| L1    | Primitive Wrapper      | `@haruhi/ui`                         | 跨站重复、语义稳定、状态明确；只输出 class / props / slot / aria |
| L2    | Composition Recipe     | `recipes.css` + `@haruhi/ui/recipes` | 五类业务卡片；只定义 anatomy / 母题 / 状态，业务字段由调用方传入 |
| L3    | Product Component      | 未来评估                             | 多页面共享同一信息结构、状态机、数据契约后再评估                 |

### 5.2 @haruhi/ui 组件清单（~41）

- **布局**：Page · PageHeader · Toolbar · Stack · Inline · Cluster · Grid · Split · Surface · MediaFrame · Divider
- **排印**：Eyebrow · Title · Copy
- **控件**：Button · Badge · Chip · Tabs（pill / underline）
- **表单**：Field · Input · Textarea · Select · Checkbox · Switch（均 v-model）
- **数据陈列**：Card · Avatar · Table · Tooltip · Skeleton · Spinner
- **反馈**：Notice · Progress · EmptyState · ToastRegion（配 `useToast`）
- **导航**：Appbar · NavLink · Breadcrumb · Pagination · HeaderBrand
- **页脚**：`.sos-footer`（统一页脚规范：品牌锁头 + 标语 + 社交 ｜ 自适应多列链接组 + 底部条 + 回到顶部）
- **浮层**：Modal（Teleport + 焦点/滚动锁）· Dropdown（点击外关闭）

> **统一页头 / 页脚**：`.sos-appbar`（logo 锁头 + 导航 + 账号收进右侧 actions）与 `.sos-footer` 是各站共享的 chrome 规范，全部走 token，挂 `data-sos-site` 即各站主题；社交/分类品牌色经 `--brand` 仅作点缀。品牌字体/字重为表达层旋钮（`--sos-brand-font`/`-weight`，默认随各站 display 字体 + bold）。
> 应用情况：novel / exam / art 用统一 Appbar；shop 与 news 保留各自功能型页头（招牌），console 保留深色侧栏；**页脚六站统一**（console 用精简底部条）。

### 5.3 API 设计守则

- Props 只暴露稳定语义：`variant` / `size` / `tone` / `ratio` / `gap` / `selected` / `loading`；页面级 wrapper 用 `site` / `density` / `contained` / `gap`。
- **不接受** `color` / `shadow` / `radius` 等视觉 props——表达由 token + `data-sos-site` 决定。
- Slot 对应稳定 anatomy 槽位，不用任意 slot 绕过结构。
- 状态 props 必须同步无障碍证据：`aria-busy` / `aria-invalid` / `disabled` / `aria-current`。
- **按钮变体用双类选择器**（`.sos-button.sos-button--primary`）以胜过 app 内嵌的 normalize 式 `[type=submit]{background-color:transparent}` 元素级 reset——否则同特异性 + 后加载会把按钮背景压没。

---

## 6. 业务卡片 recipe

`recipes.css` 沉淀「共享卡片解剖 + 五个内容类型特化 recipe」，由 `@haruhi/ui/recipes` 的 prop 驱动组件输出；内容仍由业务传入。

| Recipe              | 站点    | 母题                                        |
| ------------------- | ------- | ------------------------------------------- |
| `.sos-article-card` | news    | 顶部信号细线、墨色描边标签、衬线摘要        |
| `.sos-product-card` | shop    | 方形媒体、角标、预售进度、价格、悬浮动作    |
| `.sos-artwork-card` | art     | 玻璃画框 + 深色信息条（题注/标签/粉色点赞） |
| `.sos-book-card`    | library | 书脊高光、竖排书名、卷册角标                |
| `.sos-exam-card`    | exam    | 答题横格、阅卷红印章、折角、藏蓝分割线      |

共享解剖（`.sos-card__media/kicker/heading/excerpt/tags`、`.sos-price`、`.sos-ribbon`、`.sos-stat`）可跨 recipe 复用。每张卡片消费当前 `data-sos-site` 的表达 token，放进对应站点即天然契合。

> 卡片是核心场景：版式、换行、尺寸比例、内部间距都要随内容类型打磨；不要把一种卡硬套到另一种内容上。

---

## 7. 接入

### 7.1 两种接入深度

| 深度             | 做法                                                                          | 例子                             |
| ---------------- | ----------------------------------------------------------------------------- | -------------------------------- |
| **浅层（基线）** | 只引 `tokens.css` + 根节点 `data-sos-site`；用站点自有组件/布局，仅消费 token | `news`（保留团报自有编辑部设计） |
| **深度**         | 引 `components.css` + `@haruhi/ui` / recipes，用 DS 组件搭页面                | 规范页、`auth-ui`                |
| **桥接**         | 引 `bridges.css`，把旧站变量映射到 token（仅 shop / art）                     | `shop` / `art`                   |

### 7.2 铁律：接入不得同化既有站点的字体

`.sos-scope` 自带一套 DS 基础排版。把它整块罩在「自带成熟排版的旧站」上会**覆盖原字体气质、造成视觉倒退**（如 shop 原 Helvetica + 1px 字距被换成 Inter）。设计系统应只经 `data-sos-site` + bridges 提供**配色/几何**，不强行同化字体。`.sos-scope` 的字体/行高因此可覆盖：

```css
/* 旧站在自己的根上保留原排版，仅吃 DS 的配色/几何 */
.shop-app.sos-scope {
  --sos-scope-font: 'Helvetica Neue', Helvetica, Arial, sans-serif;
  --sos-scope-leading: 1.6;
}
.sos-scope[data-sos-site='news'] {
  --sos-scope-font: var(--font-sans); /* 团报的 Noto Sans SC */
}
```

新建的 DS 原生页（规范页、auth-ui）不设这两个变量，自然采用设计系统字体。
**接旧站只加 `data-sos-site`，别盲目套 `.sos-scope` 全量重置。**

### 7.3 当前各站接入状态

| App       | 模式    | 接入深度                                 |
| --------- | ------- | ---------------------------------------- |
| `shop`    | shop    | 桥接（bridges 全量映射）+ 个别 SosButton |
| `news`    | news    | **浅层 + 全量 token 化**（编辑部核心与各 feature 配色收敛到 token，保留毛笔报头/衬线/编辑部卡片与活动中心等彩色特色；仍不套 DS 组件以免同化） |
| `art`     | art     | 桥接                                     |
| `novel`   | library | **深度**（tokens + components/recipe 类，书架/阅读器/后台重构；正文沿用站点自有衬线排版） |
| `exam`    | exam    | **深度**（tokens + components.css；首页/编辑器/审核/页脚全面 token 化，绿+多重红收敛为阅卷红+藏蓝+金；答题纸木纹/手写/阅卷印章作保护区保留） |
| `console` | base    | **桥接**（tokens + data-sos-theme=dark；深色超管台局部变量桥接到 DS 暗色语义层，行动色用 DS 蓝） |
| `auth-ui` | 随站点  | 深度（DS 原生），由各 app 路由传 `site`  |

---

## 8. 内容与数据表达

设计系统约束的不只是颜色和组件，也包括页面承载的信息结构。

**内容原则**

| 原则             | 规则                                                           |
| ---------------- | -------------------------------------------------------------- |
| 真实对象先于装饰 | 标题、日期、价格、进度、作者、状态来自业务字段，不造站点或口号 |
| 一条信息一个职责 | 标题识别对象，摘要解释差异，状态说明阶段，操作指向下一步       |
| 关键数据常驻     | 价格、库存、倒计时、审核、答题进度不能只在 hover 或浮层出现    |
| 语气具体克制     | 错误、空、成功反馈说清原因与下一步，不写泛口号                 |

**数据格式**

| 类型 | 示例               | 规则                                                 |
| ---- | ------------------ | ---------------------------------------------------- |
| 日期 | `2026-06-23`       | 列表/审核/订单默认稳定日期；相对时间只用于动态流     |
| 价格 | `¥147`             | 等宽数字（`--sos-numeric-tabular`）；折扣/总价可比较 |
| 进度 | `126/200 · 63%`    | 同时保留当前值、目标值或百分比                       |
| 库存 | `现货 12 / 预售中` | 文字常驻，不只显示色点或图标                         |

**状态文案**：Loading 保留骨架与上下文标题；Empty 说明为空原因 + 真实下一步；Error 说明失败原因 + 内容是否保留 + 重试/返回；Unavailable（下架/维护/暂停/审核中）必须有不同文案，不能统一写「暂无」。

---

## 9. 可访问性（上线门槛）

| Gate           | 验收                                                                  |
| -------------- | --------------------------------------------------------------------- |
| Text Contrast  | 普通文本达 WCAG AA；弱文本不承担唯一状态证据                          |
| Keyboard       | 全流程可键盘操作；Tab 顺序与视觉一致；`focus-visible` 用 `--sos-ring` |
| Touch Target   | 可点击元素 ≥44px；核心 CTA 48px；相邻目标有间距                       |
| State Evidence | 状态不只靠颜色，需文字/图标/位置/形状证据                             |
| Dialog / Toast | Dialog 管理焦点、关闭、背景 inert；Toast 用 `aria-live`               |
| Media          | 有意义的 `alt`；装饰图空 `alt`                                        |
| Reduced Motion | 位移/旋转降级，信息时序不变                                           |
| Forced Colors  | 边框/文字/焦点/可点区仍可辨认                                         |
| Zoom           | 200% 缩放不遮挡正文、表单、按钮与状态                                 |
| Locale         | 中文页面 `lang="zh-CN"`                                               |

---

## 10. 变更纪律

- 改 token / 组件 / recipe 后，先在规范页 `pnpm dev:design-system` 自查五个模式 + 明暗 + 密度。
- 本地门禁：`pnpm check:design-system`（prettier + eslint + 规范页 build + `@haruhi/ui` typecheck）。
- 视觉回归（免后端，Playwright 拦截 `/api/**` 注入 fixture）：`pnpm check:news:visual` / `check:shop:visual`，先起对应 `pnpm dev:<app>`。
- 不新增 raw hex、临时阴影、非 4px 间距、临时圆角；表达走 token，不发明视觉 props。
- 业务卡片属 L2，先用真实数据在 recipe / 业务页验证，再考虑升级。
