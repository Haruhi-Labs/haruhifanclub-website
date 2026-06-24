# @haruhi/ui

SOS / Parallel Design System 的 Vue 组件库。

这个包只封装已经稳定的 CSS class contract，不重新定义视觉样式。样式来源仍是 `@haruhi/design-system`；业务 app 可以直接使用 CSS，也可以在 Vue 页面中使用本包 wrapper。

## 导出

涵盖布局、排印、控件、表单、数据陈列、反馈、导航、浮层与一个 toast composable：

```js
import {
  // 布局
  SosPage,
  SosPageHeader,
  SosToolbar,
  SosStack,
  SosInline,
  SosCluster,
  SosGrid,
  SosSplit,
  SosSurface,
  SosMediaFrame,
  SosDivider,
  // 排印
  SosEyebrow,
  SosTitle,
  SosCopy,
  // 控件 / 表单
  SosButton,
  SosBadge,
  SosChip,
  SosTabs,
  SosField,
  SosInput,
  SosTextarea,
  SosSelect,
  SosCheckbox,
  SosSwitch,
  // 数据陈列
  SosCard,
  SosAvatar,
  SosTable,
  SosTooltip,
  SosSkeleton,
  SosSpinner,
  // 反馈
  SosNotice,
  SosProgress,
  SosEmptyState,
  SosToastRegion,
  // 导航
  SosAppbar,
  SosNavLink,
  SosBreadcrumb,
  SosPagination,
  SosHeaderBrand,
  // 浮层
  SosModal,
  SosDropdown,
  // composable
  useToast,
} from '@haruhi/ui'
```

`@haruhi/ui` 会自动引入 `@haruhi/design-system/components.css`。如果页面已经在入口处引入该 CSS，也可以继续保留，重复导入由构建器去重。

使用 `SosPage` 时可以由 wrapper 输出 scope 和站点模式；如果页面暂不使用 `SosPage`，根节点仍需要明确表达模式：

```vue
<template>
  <SosPage site="shop">
    <RouterView />
  </SosPage>
</template>
```

## 当前组件

**布局与结构**

| 组件                                    | 用途                            |
| --------------------------------------- | ------------------------------- |
| `SosPage` / `SosPageHeader`             | 页面根容器、scope、标题与动作区 |
| `SosStack` / `SosInline` / `SosCluster` | 纵向节奏 / 同行操作 / 两端对齐  |
| `SosGrid` / `SosSplit`                  | 自适应卡片网格 / 稳定双栏       |
| `SosSurface` / `SosMediaFrame`          | 有边界承载面 / 固定媒体比例     |
| `SosToolbar` / `SosDivider`             | 工具组布局 / 分隔线（可带标签） |

**排印与控件**

| 组件                                  | 用途                                        |
| ------------------------------------- | ------------------------------------------- |
| `SosEyebrow` / `SosTitle` / `SosCopy` | 眉标 / 标题（含 hero）/ 正文                |
| `SosButton`                           | primary / secondary / ghost / danger / link |
| `SosBadge` / `SosChip`                | 状态短标签 / 可切换可删除筛选标记           |
| `SosTabs`                             | 分段 pill 与下划线两种 variant              |

**表单**

| 组件                                     | 用途                             |
| ---------------------------------------- | -------------------------------- |
| `SosField`                               | Label / Control / Help anatomy   |
| `SosInput` / `SosTextarea` / `SosSelect` | 文本、多行、下拉，均支持 v-model |
| `SosCheckbox` / `SosSwitch`              | 勾选 / 单选 / 开关               |

**数据陈列与反馈**

| 组件                                        | 用途                                       |
| ------------------------------------------- | ------------------------------------------ |
| `SosCard`                                   | 基础卡片 anatomy，不封装业务卡片           |
| `SosAvatar` / `SosTable`                    | 头像（含 group）/ 列定义表格               |
| `SosTooltip` / `SosSkeleton` / `SosSpinner` | 提示 / 骨架屏 / 加载指示                   |
| `SosNotice` / `SosProgress`                 | info/success/warning/danger / 常驻数值进度 |
| `SosEmptyState` / `SosToastRegion`          | 空状态 / Toast 区（配 `useToast`）         |

**导航与浮层**

| 组件                              | 用途                         |
| --------------------------------- | ---------------------------- |
| `SosAppbar` / `SosNavLink`        | 应用顶栏 / 导航链接          |
| `SosBreadcrumb` / `SosPagination` | 面包屑 / 分页（v-model）     |
| `SosHeaderBrand`                  | Header logo + 标题组合       |
| `SosModal` / `SosDropdown`        | 模态框（Teleport）/ 下拉菜单 |

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

## 业务卡片 recipe（@haruhi/ui/recipes）

L2 业务卡片从 `@haruhi/ui/recipes` 子路径按需引入，prop 驱动，内容由业务传入：

```js
import {
  SosArticleCard, // 团报阅读卡：label/title/subtitle/excerpt/tags/date/author/pinned
  SosProductCard, // 商品卡：price/originalPrice/badge/progress/soldOut + #actions 插槽
  SosArtworkCard, // 作品卡：image/author/likes/views + 悬停题注
  SosBookCard, // 书封卡：cover/color/badge + 竖排书名
  SosExamCard, // 试卷卡：subject/title/score/meta
} from '@haruhi/ui/recipes'
```

每个 recipe 都消费当前 `data-sos-site` 的表达 token，因此放进对应站点即天然契合；样式契约见
`@haruhi/design-system/recipes.css`。这是 L2：组件只定义 anatomy 与状态，标题、价格、库存等业务字段仍由调用方传入。

## 主题与表达

- 表达模式由外层 `data-sos-site`（news / shop / art / library / exam）控制，组件解剖不变。
- 明暗主题由 `data-sos-theme="dark"` 切换，只重映射语义 token；建议挂在 `documentElement` 上，
  这样 Teleport 到 body 的 `SosModal` / `SosToastRegion` 也能继承。
- 密度由 `data-sos-density="compact | spacious"` 控制，只改控件节奏。

## 边界

- 不封装 `NewsArticleCard`、`ShopProductCard`、`ArtworkCard`、`LibraryBookCard`、`ExamPaperCard` 等业务卡片。
- `SosEmptyState` 是系统状态模式，不是营销卡片或落地页模块。
- `SosPage`、`SosPageHeader`、`SosToolbar` 用于新界面重构时统一页面骨架，不负责生成业务内容。
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
- PageHeader 的 title、copy、meta 和 actions 必须来自当前页面任务，不能写成泛营销口号。
- Toolbar 只组织筛选、排序、结果数和批量动作，不决定具体筛选语义。

## 准入分级

| Level | 名称                   | 落点                    | 说明                                                 |
| ----- | ---------------------- | ----------------------- | ---------------------------------------------------- |
| L0    | Token / Class Contract | `@haruhi/design-system` | 颜色、间距、圆角、布局原语和基础 class。             |
| L1    | Primitive Wrapper      | `@haruhi/ui`            | 本包接收跨站重复、语义稳定、状态明确的基础 wrapper。 |
| L1.5  | Page Structure Wrapper | `@haruhi/ui`            | Page、PageHeader、Toolbar 等帮助应用按统一骨架重构。 |
| L2    | Composition Recipe     | 规范页 + 业务 app       | 业务卡片和页面组合先用真实数据验证，不进入本包。     |
| L3    | Product Component      | 未来评估                | 多页面共享同一信息结构、状态机和数据契约后再评估。   |

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

本地检查：

```sh
pnpm check:design-system
```

## 版本规则

- Minor：新增 wrapper、非破坏性 props、文档 section 或 recipe 示例。
- Patch：修复 class 组合、aria、类型声明、文档和响应式问题。
- Breaking：改变 anatomy、slot、variant 语义或移除 props；必须同时给迁移步骤和回滚边界。
