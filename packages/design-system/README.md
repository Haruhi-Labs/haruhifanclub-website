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

1. 在目标 app 的入口样式中引入 `tokens.css`。
2. 给页面根节点补 `class="sos-scope"` 和对应 `data-sos-site`。
3. 如果旧 CSS 变量较多，短期加载 `bridges.css`，把旧变量指向新语义 token。
4. 先使用 `Stack / Inline / Surface / MediaFrame` 统一布局和媒体比例。
5. 按 Button、Badge、Input、Tabs、Notice、Progress 的顺序迁移基础组件。
6. 再迁移业务卡片和页面骨架。

不要在未盘点旧变量前扩大 bridge。bridge 是过渡层，不是新变量命名空间。

## 和 @haruhi/ui 的关系

`@haruhi/ui` 已启用 MVP，适合在 Vue app 中复用稳定基础件：Button、Badge、Field、Notice、Progress、Card、HeaderBrand、Stack、Inline、Surface、MediaFrame。

业务 app 不必须立刻引入 `@haruhi/ui`。迁移优先级仍是：

1. 先接入本包 token 和 class contract。
2. 再把重复出现的基础控件替换为 `@haruhi/ui` wrapper。
3. 最后才评估业务卡片是否可以抽成共享 recipe 或组件。
