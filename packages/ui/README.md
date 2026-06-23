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

使用 wrapper 前，页面根节点仍需要明确表达模式：

```vue
<template>
  <main class="sos-scope" data-sos-site="shop">
    <RouterView />
  </main>
</template>
```

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

## 内容职责

`@haruhi/ui` 只封装 anatomy、状态和可访问性，不生成业务内容：

- Button 文案由调用方传入，必须指向明确动作。
- Field 的 label、help、error 来自具体表单语义，不能只靠 placeholder。
- EmptyState 必须说明为空原因和下一步，不能只传“暂无数据”。
- Progress 的 `label` 和 `valueLabel` 应保留当前值、目标值或百分比。
- Card wrapper 不决定标题、价格、库存、日期、作者等业务字段。

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

## 变更证据

新增 wrapper、props 或 variant 前，先更新规范页和状态矩阵。PR 至少提供：

- 默认、hover、focus-visible、loading、disabled、error/empty 中受影响状态的截图。
- 390 / 768 / 1280px 响应式截图。
- 键盘路径和可访问名称说明。
- 不新增视觉 props、不重定义 token 的说明。

## 版本规则

- Minor：新增 wrapper、非破坏性 props、文档 section 或 recipe 示例。
- Patch：修复 class 组合、aria、类型声明、文档和响应式问题。
- Breaking：改变 anatomy、slot、variant 语义或移除 props；必须同时给迁移步骤和回滚边界。
