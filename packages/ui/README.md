# @haruhi/ui

SOS / Parallel Design System 的 Vue 基础组件 MVP。

这个包只封装已经稳定的 CSS class contract，不重新定义视觉样式。样式来源仍是 `@haruhi/design-system`；业务 app 可以直接使用 CSS，也可以在 Vue 页面中使用本包 wrapper。

## 导出

```js
import {
  SosBadge,
  SosButton,
  SosCard,
  SosCluster,
  SosEmptyState,
  SosField,
  SosGrid,
  SosHeaderBrand,
  SosInline,
  SosMediaFrame,
  SosNotice,
  SosProgress,
  SosSplit,
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
| `SosEmptyState`  | 空状态说明和下一步动作                                  |
| `SosHeaderBrand` | Header 中 logo + 标题文字组合                           |
| `SosStack`       | 纵向内容节奏                                            |
| `SosInline`      | 可换行同行操作                                          |
| `SosCluster`     | 两端或多组对齐                                          |
| `SosGrid`        | 自适应卡片网格                                          |
| `SosSplit`       | 稳定双栏布局                                            |
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
- `SosEmptyState` 是系统状态模式，不是营销卡片或落地页模块。
- 不新增独立主题变量；表达模式仍由外层 `data-sos-site` 和 `@haruhi/design-system` token 控制。
- 不把页面特例做成组件 variant。新增 variant 前必须先更新规范页和状态矩阵。
- 不替代 `@haruhi/auth-ui`。鉴权 UI 仍由 `packages/auth-ui` 维护，后续可逐步消费本包基础件。

## 准入分级

| Level | 名称                   | 落点                    | 说明                                                   |
| ----- | ---------------------- | ----------------------- | ------------------------------------------------------ |
| L0    | Token / Class Contract | `@haruhi/design-system` | 颜色、间距、圆角、布局原语和基础 class。               |
| L1    | Primitive Wrapper      | `@haruhi/ui`            | 本包只接收跨站重复、语义稳定、状态明确的基础 wrapper。 |
| L2    | Composition Recipe     | 规范页 + 业务 app       | 业务卡片和页面组合先用真实数据验证，不进入本包。       |
| L3    | Product Component      | 未来评估                | 多页面共享同一信息结构、状态机和数据契约后再评估。     |

API 规则：

- Props 只暴露稳定语义：`variant`、`size`、`tone`、`ratio`、`gap`、`selected`、`loading`。
- 不提供 `color`、`shadow`、`radius` 等视觉 props；表达由外层 token 和 `data-sos-site` 决定。
- Slot 对应稳定 anatomy 槽位，不用任意 slot 绕过结构。
- 状态 props 必须同步可访问性证据，例如 `aria-busy`、`aria-invalid`、`disabled`。
