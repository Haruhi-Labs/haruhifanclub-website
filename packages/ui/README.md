# @haruhi/ui

SOS / Parallel Design System 的 Vue 基础组件 MVP。

这个包只封装已经稳定的 CSS class contract，不重新定义视觉样式。样式来源仍是 `@haruhi/design-system`；业务 app 可以直接使用 CSS，也可以在 Vue 页面中使用本包 wrapper。

## 导出

```js
import {
  SosBadge,
  SosButton,
  SosCard,
  SosField,
  SosHeaderBrand,
  SosInline,
  SosMediaFrame,
  SosNotice,
  SosProgress,
  SosStack,
  SosSurface,
} from '@haruhi/ui'
```

`@haruhi/ui` 会自动引入 `@haruhi/design-system/components.css`。如果页面已经在入口处引入该 CSS，也可以继续保留，重复导入由构建器去重。

## 当前组件

| 组件             | 用途                                                    |
| ---------------- | ------------------------------------------------------- |
| `SosButton`      | 明确命令按钮，支持 primary / secondary / ghost / danger |
| `SosBadge`       | 状态、分类、短标签                                      |
| `SosField`       | Label、Control、Help / Error anatomy                    |
| `SosNotice`      | 系统提示，支持 info / success / warning / danger        |
| `SosProgress`    | 常驻数值的进度反馈                                      |
| `SosCard`        | 基础卡片 anatomy，不封装业务卡片                        |
| `SosHeaderBrand` | Header 中 logo + 标题文字组合                           |
| `SosStack`       | 纵向内容节奏                                            |
| `SosInline`      | 可换行同行操作                                          |
| `SosSurface`     | 有边界的承载面                                          |
| `SosMediaFrame`  | 固定媒体比例容器                                        |

## 使用示例

```vue
<script setup>
import { SosButton, SosField, SosStack } from '@haruhi/ui'
</script>

<template>
  <SosStack gap="loose" data-sos-site="shop">
    <SosField label="商品标题" help="Label 不被 placeholder 替代。">
      <input class="sos-input" value="SOS 团限定徽章" />
    </SosField>
    <SosButton>保存商品</SosButton>
  </SosStack>
</template>
```

## 边界

- 不封装 `NewsArticleCard`、`ShopProductCard`、`ArtworkCard`、`LibraryBookCard`、`ExamPaperCard` 等业务卡片。
- 不新增独立主题变量；表达模式仍由外层 `data-sos-site` 和 `@haruhi/design-system` token 控制。
- 不把页面特例做成组件 variant。新增 variant 前必须先更新规范页和状态矩阵。
- 不替代 `@haruhi/auth-ui`。鉴权 UI 仍由 `packages/auth-ui` 维护，后续可逐步消费本包基础件。
