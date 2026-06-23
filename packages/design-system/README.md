# @haruhi/design-system

SOS / Parallel Design System 的 CSS-first 共享包。

这个包只提供框架无关样式契约，不提供 Vue 或 React 组件。它和 `@haruhi/ui` 分开：本包负责 token、基础 class contract 和旧变量 bridge；`@haruhi/ui` 只做 Vue wrapper，输出同一套 class，不重新定义视觉。

## 导出

```js
import '@haruhi/design-system/tokens.css'
import '@haruhi/design-system/components.css'
import '@haruhi/design-system/bridges.css'
```

| 入口             | 用途                                                              |
| ---------------- | ----------------------------------------------------------------- |
| `tokens.css`     | Primitive、Semantic、Expression token。适合先接入页面上下文。     |
| `components.css` | Layout primitives、基础组件 class contract，已包含 `tokens.css`。 |
| `bridges.css`    | 旧站点变量兼容桥，只在渐进迁移时加载。                            |

## 使用约定

页面或局部区域通过 `data-sos-site` 选择表达模式：

```html
<section class="sos-scope" data-sos-site="shop">
  <button class="sos-button sos-button--primary">加入购物车</button>
</section>
```

业务代码优先使用语义变量，不直接写物理色值：

```css
.price {
  color: var(--sos-link);
}
```

常用语义变量：

| Token                                     | 用途                            |
| ----------------------------------------- | ------------------------------- |
| `--sos-bg-page`                           | 页面根背景                      |
| `--sos-bg-surface`                        | 卡片、表单、弹层、Notice 承载面 |
| `--sos-bg-subtle`                         | 筛选条、空状态、状态格          |
| `--sos-text-primary`                      | 标题、强数字、主要操作文本      |
| `--sos-text-secondary`                    | 摘要、说明、meta、help text     |
| `--sos-border-subtle/default/strong`      | 分隔和承载面边界                |
| `--sos-accent`                            | 当前表达模式的主行动            |
| `--sos-signal`                            | 少量品牌线索和重点标签          |
| `--sos-danger` / `--sos-success`          | 跨站点一致的错误和成功状态      |
| `--sos-card-radius` / `--sos-card-shadow` | 卡片几何与深度                  |

需要站点气质时调整 Expression Mapping，不在业务组件里新增 hex、临时阴影、13px 圆角或非 4px 网格间距。

## 内容与数据

设计系统接入不只替换样式，也要保留真实信息结构：

- 日期优先使用稳定日期，例如 `2026-06-23`；相对时间只用于动态流。
- 价格、库存、进度、倒计时和审核状态必须常驻，不依赖 hover 或图片说明。
- 进度同时保留当前值、目标值或百分比，例如 `126/200 · 63%`。
- Empty、Error、Unavailable 必须说明原因和下一步，不能统一写“暂无”。
- 业务页面不得用虚造站点、口号或示意字段代替真实数据结构。

## v0.2 Primitives

`components.css` 提供一组框架无关布局原语，供业务页面和未来 Vue wrapper 复用：

| Class              | 用途                     |
| ------------------ | ------------------------ |
| `.sos-stack`       | 纵向内容节奏             |
| `.sos-inline`      | 可换行同行操作           |
| `.sos-cluster`     | 两端或多组对齐           |
| `.sos-grid`        | 自适应卡片网格           |
| `.sos-split`       | 稳定双栏布局             |
| `.sos-surface`     | 有边界的承载面           |
| `.sos-media-frame` | 媒体比例容器             |
| `.sos-state-row`   | 数值、进度、库存等状态行 |

媒体比例通过属性声明：

```html
<div class="sos-media-frame" data-ratio="1:1">
  <img src="/product.png" alt="商品图" />
</div>
```

不要在业务页面用固定高度临时修卡片比例。优先用 `sos-media-frame`、`sos-card__body` 和 `sos-card__footer` 稳定 anatomy。

## 迁移顺序

目标 app 的表达模式：

| App               | Mode             | 入口样式                         |
| ----------------- | ---------------- | -------------------------------- |
| `@haruhi/news`    | `news`           | `apps/news/src/style.css`        |
| `@haruhi/shop`    | `shop`           | `apps/shop/src/assets/shop.css`  |
| `@haruhi/art`     | `art`            | `apps/art/src/style.css`         |
| `@haruhi/novel`   | `library`        | `apps/novel/src/assets/base.css` |
| `@haruhi/exam`    | `exam`           | `apps/exam/src/style.css`        |
| `@haruhi/console` | `base + compact` | `apps/console/src/style.css`     |

1. 在目标 app 的入口样式中引入 `tokens.css`。
2. 给页面根节点补 `class="sos-scope"` 和对应 `data-sos-site`。
3. 如果旧 CSS 变量较多，短期加载 `bridges.css`，把旧变量指向新语义 token。
4. 先使用 `Stack / Inline / Surface / MediaFrame` 统一布局和媒体比例。
5. 按 Button、Badge、Input、Tabs、Notice、Progress 的顺序迁移基础组件。
6. 再迁移业务卡片和页面骨架。

最小入口片段：

```js
import '@haruhi/design-system/tokens.css'
import '@haruhi/design-system/components.css'
// 仅渐进迁移旧变量时加载
import '@haruhi/design-system/bridges.css'
```

业务页面根节点：

```html
<main class="sos-scope" data-sos-site="shop">...</main>
```

不要在未盘点旧变量前扩大 bridge。bridge 是过渡层，不是新变量命名空间。

## PR 证据

涉及本包 token、component class 或 bridge 的变更，需要在 PR 中说明：

- 影响的 app、路由、旧变量和未迁移项。
- 390 / 768 / 1280px 截图，证明无横向溢出且关键数据没有丢失。
- 新增样式没有 raw hex、临时阴影、临时圆角或非 4px 网格间距。
- bridge 变量的 owner、使用范围和删除计划。
- 移除入口 import 或 bridge 文件后的回滚边界。

## 和 @haruhi/ui 的关系

`@haruhi/ui` 已启用 MVP，适合在 Vue app 中复用稳定基础件：Button、Badge、Field、Notice、Progress、Card、EmptyState、HeaderBrand、Stack、Inline、Cluster、Grid、Split、Surface、MediaFrame。

本包处在 L0：Token / Class Contract。`@haruhi/ui` 处在 L1：Primitive Wrapper。新闻卡、商品卡、作品卡、书封卡、试卷卡等业务组合仍处在 L2 recipe 阶段，不应直接进入本包或 UI 包。

业务 app 不必须立刻引入 `@haruhi/ui`。迁移优先级仍是：

1. 先接入本包 token 和 class contract。
2. 再把重复出现的基础控件替换为 `@haruhi/ui` wrapper。
3. 最后才评估业务卡片是否可以抽成共享 recipe 或组件。
