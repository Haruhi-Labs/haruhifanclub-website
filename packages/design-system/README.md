# @haruhi/design-system

SOS / Parallel Design System 的 CSS-first 共享包。

这个包只提供框架无关样式契约，不提供 Vue 或 React 组件。现阶段它和 `packages/ui` 分开：前者负责 token、基础 class contract 和旧变量 bridge；后者仍预留给未来真正稳定的组件封装。

## 导出

```js
import '@haruhi/design-system/tokens.css'
import '@haruhi/design-system/components.css'
import '@haruhi/design-system/bridges.css'
```

| 入口             | 用途                                                          |
| ---------------- | ------------------------------------------------------------- |
| `tokens.css`     | Primitive、Semantic、Expression token。适合先接入页面上下文。 |
| `components.css` | 基础组件 class contract，已包含 `tokens.css`。                |
| `bridges.css`    | 旧站点变量兼容桥，只在渐进迁移时加载。                        |

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

## 迁移顺序

1. 在目标 app 的入口样式中引入 `tokens.css`。
2. 给页面根节点补 `class="sos-scope"` 和对应 `data-sos-site`。
3. 如果旧 CSS 变量较多，短期加载 `bridges.css`，把旧变量指向新语义 token。
4. 按 Button、Badge、Input、Tabs、Notice、Progress 的顺序迁移基础组件。
5. 再迁移业务卡片和页面骨架。

不要在未盘点旧变量前扩大 bridge。bridge 是过渡层，不是新变量命名空间。
