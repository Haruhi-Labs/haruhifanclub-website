<script setup>
import { onMounted, onBeforeUnmount, ref, watch } from 'vue'
import {
  SosAvatar,
  SosBadge,
  SosBreadcrumb,
  SosButton,
  SosCard,
  SosCheckbox,
  SosChip,
  SosCopy,
  SosDivider,
  SosDropdown,
  SosEmptyState,
  SosEyebrow,
  SosField,
  SosGrid,
  SosInput,
  SosModal,
  SosNotice,
  SosPagination,
  SosProgress,
  SosSelect,
  SosSkeleton,
  SosSpinner,
  SosSwitch,
  SosTable,
  SosTabs,
  SosTextarea,
  SosTitle,
  SosToastRegion,
  SosTooltip,
  useToast,
} from '@haruhi/ui'
import {
  SosArticleCard,
  SosProductCard,
  SosArtworkCard,
  SosBookCard,
  SosExamCard,
} from '@haruhi/ui/recipes'

const logoUrl = `${import.meta.env.BASE_URL}haruhi-logo-192.png`

// 演示用 SVG 占位图
const ph = (text, bg, fg) =>
  `data:image/svg+xml,` +
  encodeURIComponent(
    `<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 400 300'><rect width='400' height='300' fill='${bg || '#e8eef6'}'/><circle cx='320' cy='70' r='46' fill='${fg || '#9aa1ad'}' opacity='.28'/><text x='200' y='168' font-family='sans-serif' font-size='30' font-weight='700' fill='${fg || '#6b7480'}' text-anchor='middle'>${text}</text></svg>`
  )

/* ---------------- 全局演示状态 ---------------- */
const site = ref('shop')
const theme = ref('light')
const density = ref('comfortable')

const SITES = [
  { value: 'base', label: '基础 base' },
  { value: 'news', label: '春日团报 news' },
  { value: 'shop', label: '春日商城 shop' },
  { value: 'art', label: '美术部 art' },
  { value: 'library', label: '小说书库 library' },
  { value: 'exam', label: '考试平台 exam' },
]
const DENSITIES = [
  { value: 'comfortable', label: '舒适' },
  { value: 'compact', label: '紧凑' },
  { value: 'spacious', label: '宽松' },
]

watch(
  theme,
  (value) => {
    if (typeof document !== 'undefined') {
      document.documentElement.dataset.sosTheme = value === 'dark' ? 'dark' : ''
    }
  },
  { immediate: true }
)

const siteAttr = (value) => (value === 'base' ? undefined : value)
const densityAttr = (value) => (value === 'comfortable' ? undefined : value)

/* ---------------- 交互演示状态 ---------------- */
const tab = ref('all')
const tabItems = [
  { value: 'all', label: '全部' },
  { value: 'active', label: '进行中' },
  { value: 'done', label: '已完成' },
]
const utab = ref('overview')
const utabItems = [
  { value: 'overview', label: '概览' },
  { value: 'detail', label: '明细' },
  { value: 'review', label: '评价' },
]
const page = ref(2)
const chips = ref({ new: true, hot: false, limited: false })
const name = ref('凉宫 春日')
const phone = ref('123')
const agree = ref(true)
const notify = ref(true)
const ship = ref('express')
const modalOpen = ref(false)
const toast = useToast()

function confirmCart() {
  modalOpen.value = false
  toast.success('已加入购物车')
}

/* ---------------- Foundations 数据 ---------------- */
const semanticColors = [
  ['页面', '--sos-bg-page'],
  ['承载面', '--sos-bg-surface'],
  ['弱背景', '--sos-bg-subtle'],
  ['强背景', '--sos-bg-strong'],
  ['主文本', '--sos-text-primary'],
  ['次文本', '--sos-text-secondary'],
  ['强调', '--sos-accent'],
  ['强调浅', '--sos-accent-soft'],
  ['链接', '--sos-link'],
  ['信号', '--sos-signal'],
  ['危险', '--sos-danger'],
  ['成功', '--sos-success'],
  ['警告', '--sos-warning'],
  ['信息', '--sos-info'],
]
const typeScale = [
  ['hero', '--sos-text-hero'],
  ['4xl', '--sos-text-4xl'],
  ['3xl', '--sos-text-3xl'],
  ['2xl', '--sos-text-2xl'],
  ['xl', '--sos-text-xl'],
  ['lg', '--sos-text-lg'],
  ['md', '--sos-text-md'],
  ['sm', '--sos-text-sm'],
  ['xs', '--sos-text-xs'],
]
const spaces = [1, 2, 3, 4, 5, 6, 8, 10, 12, 16]
const radii = [
  ['xs', '--sos-radius-xs'],
  ['sm', '--sos-radius-sm'],
  ['md', '--sos-radius-md'],
  ['lg', '--sos-radius-lg'],
  ['xl', '--sos-radius-xl'],
  ['2xl', '--sos-radius-2xl'],
]
const elevations = [
  ['hairline', '--sos-shadow-hairline'],
  ['sm', '--sos-shadow-sm'],
  ['card', '--sos-shadow-card'],
  ['float', '--sos-shadow-float'],
  ['overlay', '--sos-shadow-overlay'],
]

const orderColumns = [
  { key: 'id', label: '订单号' },
  { key: 'item', label: '商品' },
  { key: 'qty', label: '数量', numeric: true },
  { key: 'total', label: '金额', numeric: true },
  { key: 'status', label: '状态' },
]
const orderRows = [
  { id: '#10231', item: 'SOS 团应援手幅', qty: 2, total: '¥96', status: '已发货' },
  { id: '#10232', item: '朝比奈实玖瑠 徽章', qty: 5, total: '¥75', status: '待付款' },
  { id: '#10233', item: '凉宫春日 复刻海报', qty: 1, total: '¥58', status: '已取消' },
]
const statusTone = (status) =>
  status === '已发货' ? 'success' : status === '已取消' ? 'danger' : 'default'

const crumbs = [
  { label: '首页', href: '#' },
  { label: '周边分类', href: '#' },
  { label: '亚克力立牌' },
]

const products = [
  { name: '长门有希 亚克力立牌', price: '¥128', tag: '新品' },
  { name: '朝比奈实玖瑠 徽章套组', price: '¥45', tag: '热卖' },
  { name: 'SOS 团 复刻应援手幅', price: '¥68', tag: '限量' },
]

const cardShowcase = [
  { site: 'news', name: '团报 · 阅读卡' },
  { site: 'shop', name: '商城 · 商品卡' },
  { site: 'art', name: '美术部 · 作品卡' },
  { site: 'library', name: '书库 · 书封卡' },
  { site: 'exam', name: '考场 · 试卷卡' },
]

/* ---------------- Sidebar 导航 + 滚动高亮 ---------------- */
const nav = [
  { group: '入门', items: [{ id: 'overview', label: '总览' }] },
  {
    group: '基础',
    items: [
      { id: 'color', label: '颜色' },
      { id: 'type', label: '字体排印' },
      { id: 'space', label: '间距 · 圆角 · 层次' },
    ],
  },
  {
    group: '组件',
    items: [
      { id: 'buttons', label: '按钮 · 徽章' },
      { id: 'forms', label: '表单' },
      { id: 'data', label: '卡片 · 数据' },
      { id: 'feedback', label: '反馈' },
      { id: 'overlay', label: '浮层' },
      { id: 'nav', label: '导航' },
      { id: 'cards', label: '业务卡片 recipe' },
    ],
  },
  {
    group: '表达',
    items: [
      { id: 'expressions', label: '五个平行世界' },
      { id: 'patterns', label: '组合范式' },
    ],
  },
]
const active = ref('overview')
let observer
onMounted(() => {
  const targets = nav.flatMap((g) => g.items).map((i) => document.getElementById(i.id))
  observer = new IntersectionObserver(
    (entries) => {
      const visible = entries
        .filter((e) => e.isIntersecting)
        .sort((a, b) => b.intersectionRatio - a.intersectionRatio)[0]
      if (visible) active.value = visible.target.id
    },
    { rootMargin: '-20% 0px -70% 0px', threshold: [0.1, 0.5] }
  )
  targets.forEach((t) => t && observer.observe(t))
})
onBeforeUnmount(() => observer?.disconnect())
</script>

<template>
  <div class="ds sos-scope">
    <!-- Sidebar -->
    <aside class="ds-sidebar">
      <a class="sos-brand-lockup" href="#overview">
        <span class="sos-brand-lockup__mark"><img :src="logoUrl" alt="SOS" /></span>
        <span class="sos-brand-lockup__text">
          <strong>SOS / Parallel</strong>
          <small>Design System v0.3</small>
        </span>
      </a>
      <nav class="ds-nav">
        <template v-for="group in nav" :key="group.group">
          <p class="ds-nav__group">{{ group.group }}</p>
          <a
            v-for="item in group.items"
            :key="item.id"
            class="ds-nav__link"
            :href="`#${item.id}`"
            :aria-current="active === item.id ? 'true' : undefined"
          >
            {{ item.label }}
          </a>
        </template>
      </nav>
    </aside>

    <!-- Main -->
    <div class="ds-main">
      <div class="ds-topbar">
        <SosEyebrow>实时预览 · LIVE</SosEyebrow>
        <div class="ds-controls">
          <label class="ds-control">
            表达模式
            <SosSelect v-model="site" :options="SITES" />
          </label>
          <label class="ds-control">
            密度
            <SosSelect v-model="density" :options="DENSITIES" />
          </label>
          <SosButton
            variant="secondary"
            size="sm"
            @click="theme = theme === 'dark' ? 'light' : 'dark'"
          >
            {{ theme === 'dark' ? '☀ 亮色' : '☾ 暗色' }}
          </SosButton>
        </div>
      </div>

      <main
        class="ds-content sos-scope"
        :data-sos-site="siteAttr(site)"
        :data-sos-density="densityAttr(density)"
      >
        <!-- Overview -->
        <section id="overview" class="ds-section">
          <div class="ds-section__head">
            <SosEyebrow>ONE SPINE · FIVE WORLDS</SosEyebrow>
            <SosTitle as="h1" size="hero">一套语言，五个平行世界</SosTitle>
            <SosCopy variant="lead">
              同一套组件解剖与无障碍契约，通过语义 token
              在春日团报、商城、美术部、书库与考试平台之间表达出各自气质。
              用上方控件实时切换表达模式、密度与明暗，所有演示都会同步响应。
            </SosCopy>
          </div>
          <SosGrid min="13rem">
            <SosCard>
              <div class="sos-card__body sos-stack sos-stack--tight">
                <SosTitle as="h3" style="font-size: var(--sos-text-2xl)">31</SosTitle>
                <SosCopy variant="small">个组件 class 契约 + Vue 封装</SosCopy>
              </div>
            </SosCard>
            <SosCard>
              <div class="sos-card__body sos-stack sos-stack--tight">
                <SosTitle as="h3" style="font-size: var(--sos-text-2xl)">5</SosTitle>
                <SosCopy variant="small">个站点表达模式，一条共享脊柱</SosCopy>
              </div>
            </SosCard>
            <SosCard>
              <div class="sos-card__body sos-stack sos-stack--tight">
                <SosTitle as="h3" style="font-size: var(--sos-text-2xl)">3 层</SosTitle>
                <SosCopy variant="small">Primitive → Semantic → Expression token</SosCopy>
              </div>
            </SosCard>
            <SosCard>
              <div class="sos-card__body sos-stack sos-stack--tight">
                <SosTitle as="h3" style="font-size: var(--sos-text-2xl)">AA</SosTitle>
                <SosCopy variant="small">对比度、焦点环、reduced-motion 内建</SosCopy>
              </div>
            </SosCard>
          </SosGrid>
        </section>

        <!-- Color -->
        <section id="color" class="ds-section">
          <div class="ds-section__head">
            <SosEyebrow>FOUNDATIONS</SosEyebrow>
            <SosTitle as="h2" size="xl">语义颜色</SosTitle>
            <SosCopy>
              业务只消费语义 token；切换上方表达模式即可看到同一组语义在各世界的取值变化。
            </SosCopy>
          </div>
          <div class="ds-swatches">
            <div v-for="[label, token] in semanticColors" :key="token" class="ds-swatch">
              <div class="ds-swatch__chip" :style="{ background: `var(${token})` }" />
              <div class="ds-swatch__meta">
                <span class="ds-swatch__name">{{ label }}</span>
                <span class="ds-swatch__token">{{ token }}</span>
              </div>
            </div>
          </div>
        </section>

        <!-- Typography -->
        <section id="type" class="ds-section">
          <div class="ds-section__head">
            <SosEyebrow>FOUNDATIONS</SosEyebrow>
            <SosTitle as="h2" size="xl">字体排印</SosTitle>
            <SosCopy>流体字阶（clamp）让标题随视口平滑缩放；字重、字距、行高成体系。</SosCopy>
          </div>
          <div class="ds-stage ds-stage--surface ds-typescale">
            <div v-for="[tag, token] in typeScale" :key="token" class="ds-typescale__row">
              <span class="ds-typescale__tag">{{ tag }}</span>
              <span
                :style="{
                  fontSize: `var(${token})`,
                  fontWeight: 'var(--sos-weight-heavy)',
                  letterSpacing: 'var(--sos-tracking-tight)',
                  lineHeight: 'var(--sos-leading-tight)',
                }"
              >
                团长涼宮春日 SOS
              </span>
            </div>
          </div>
        </section>

        <!-- Space / radius / elevation -->
        <section id="space" class="ds-section">
          <div class="ds-section__head">
            <SosEyebrow>FOUNDATIONS</SosEyebrow>
            <SosTitle as="h2" size="xl">间距 · 圆角 · 层次</SosTitle>
            <SosCopy>4px 基准网格，双层柔光阴影，圆角随表达模式收放。</SosCopy>
          </div>
          <div class="ds-cols">
            <div class="ds-demo">
              <div class="ds-demo__label">间距 <small>4px grid</small></div>
              <div class="ds-stage ds-stage--surface ds-spaces">
                <div v-for="n in spaces" :key="n" class="ds-space-row">
                  <span style="width: 3rem">space-{{ n }}</span>
                  <span class="ds-space-bar" :style="{ width: `var(--sos-space-${n})` }" />
                </div>
              </div>
            </div>
            <div class="ds-demo">
              <div class="ds-demo__label">圆角</div>
              <div class="ds-stage ds-stage--surface ds-row">
                <div
                  v-for="[tag, token] in radii"
                  :key="token"
                  style="
                    display: grid;
                    place-items: center;
                    width: 4rem;
                    height: 4rem;
                    background: var(--sos-accent-soft);
                    color: var(--sos-link);
                    font-size: var(--sos-text-2xs);
                    font-weight: var(--sos-weight-bold);
                  "
                  :style="{ borderRadius: `var(${token})` }"
                >
                  {{ tag }}
                </div>
              </div>
            </div>
          </div>
          <div class="ds-demo">
            <div class="ds-demo__label">层次 <small>elevation</small></div>
            <div class="ds-stage ds-elevations">
              <div
                v-for="[tag, token] in elevations"
                :key="token"
                class="ds-elevation"
                :style="{ boxShadow: `var(${token})` }"
              >
                {{ tag }}
              </div>
            </div>
          </div>
        </section>

        <!-- Buttons / badges -->
        <section id="buttons" class="ds-section">
          <div class="ds-section__head">
            <SosEyebrow>COMPONENTS</SosEyebrow>
            <SosTitle as="h2" size="xl">按钮 · 徽章 · 标签</SosTitle>
          </div>
          <div class="ds-demo">
            <div class="ds-demo__label">按钮 <small>variant · size · state</small></div>
            <div class="ds-stage ds-stage--surface sos-stack">
              <div class="ds-row">
                <SosButton variant="primary">主要操作</SosButton>
                <SosButton variant="secondary">次要操作</SosButton>
                <SosButton variant="ghost">幽灵</SosButton>
                <SosButton variant="danger">删除</SosButton>
                <SosButton variant="link">了解更多 →</SosButton>
              </div>
              <div class="ds-row">
                <SosButton size="sm">小</SosButton>
                <SosButton size="md">中</SosButton>
                <SosButton size="lg">大</SosButton>
                <SosButton loading>加载中</SosButton>
                <SosButton disabled>禁用</SosButton>
              </div>
            </div>
          </div>
          <div class="ds-demo">
            <div class="ds-demo__label">徽章 · 标签</div>
            <div class="ds-stage ds-stage--surface ds-row">
              <SosBadge>默认</SosBadge>
              <SosBadge variant="accent">分类</SosBadge>
              <SosBadge variant="solid">实心</SosBadge>
              <SosBadge variant="signal">置顶</SosBadge>
              <SosBadge variant="success">已上架</SosBadge>
              <SosBadge variant="danger">缺货</SosBadge>
              <SosBadge variant="info">公告</SosBadge>
              <span class="sos-signal-tab">SOS 信号</span>
              <span class="sos-stamp">合格</span>
              <SosChip :pressed="chips.new" @toggle="chips.new = !chips.new">新品</SosChip>
              <SosChip :pressed="chips.hot" @toggle="chips.hot = !chips.hot">热卖</SosChip>
              <SosChip :pressed="chips.limited" @toggle="chips.limited = !chips.limited">
                限量
              </SosChip>
            </div>
          </div>
        </section>

        <!-- Forms -->
        <section id="forms" class="ds-section">
          <div class="ds-section__head">
            <SosEyebrow>COMPONENTS</SosEyebrow>
            <SosTitle as="h2" size="xl">表单</SosTitle>
          </div>
          <div class="ds-stage ds-stage--surface ds-cols">
            <div class="sos-stack">
              <SosField label="收件人" required>
                <SosInput v-model="name" placeholder="请输入姓名" />
              </SosField>
              <SosField label="手机号" error="手机号格式不正确">
                <SosInput v-model="phone" :invalid="true" />
              </SosField>
              <SosField label="配送方式">
                <SosSelect
                  v-model="ship"
                  :options="[
                    { value: 'express', label: '标准快递' },
                    { value: 'next', label: '次日达' },
                    { value: 'pickup', label: '到店自提' },
                  ]"
                />
              </SosField>
            </div>
            <div class="sos-stack">
              <SosField label="留言">
                <SosTextarea placeholder="给团长留句话…" :rows="4" />
              </SosField>
              <div class="sos-stack sos-stack--tight">
                <SosCheckbox v-model="agree">我已阅读并同意服务条款</SosCheckbox>
                <SosSwitch v-model="notify">接收活动通知</SosSwitch>
              </div>
              <div class="ds-row">
                <SosButton variant="primary">提交</SosButton>
                <SosButton variant="ghost">重置</SosButton>
              </div>
            </div>
          </div>
        </section>

        <!-- Data display -->
        <section id="data" class="ds-section">
          <div class="ds-section__head">
            <SosEyebrow>COMPONENTS</SosEyebrow>
            <SosTitle as="h2" size="xl">卡片 · 数据陈列</SosTitle>
          </div>

          <div class="ds-demo">
            <div class="ds-demo__label">商品卡 <small>SosProductCard recipe</small></div>
            <SosGrid min="14rem">
              <SosProductCard
                v-for="p in products"
                :key="p.name"
                :title="p.name"
                :price="p.price"
                :image="ph(p.tag, '#eaf0f8', '#7e94b8')"
                :badge="p.tag"
                desc="官方授权周边，SOS 团限量发售。"
                state="现货 12"
              >
                <template #actions>
                  <SosButton variant="primary" size="sm">加入购物车</SosButton>
                </template>
              </SosProductCard>
            </SosGrid>
          </div>

          <div class="ds-cols">
            <div class="ds-demo">
              <div class="ds-demo__label">表格</div>
              <SosTable :columns="orderColumns" :rows="orderRows" zebra row-key="id">
                <template #cell-status="{ value }">
                  <SosBadge :variant="statusTone(value)">{{ value }}</SosBadge>
                </template>
              </SosTable>
            </div>
            <div class="ds-demo">
              <div class="ds-demo__label">头像 · 骨架 · Tooltip</div>
              <div class="ds-stage ds-stage--surface sos-stack">
                <div class="sos-avatar-group">
                  <SosAvatar name="SO" />
                  <SosAvatar
                    name="団"
                    style="background: var(--sos-accent); color: var(--sos-accent-contrast)"
                  />
                  <SosAvatar name="HK" />
                  <SosAvatar name="+5" />
                </div>
                <SosCard>
                  <div class="sos-card__body sos-stack sos-stack--tight" aria-busy="true">
                    <SosSkeleton variant="title" />
                    <SosSkeleton variant="text" />
                    <SosSkeleton variant="text" width="70%" />
                  </div>
                </SosCard>
                <div class="ds-row">
                  <SosTooltip label="这是一个提示气泡">
                    <SosButton variant="secondary" size="sm">悬停查看 Tooltip</SosButton>
                  </SosTooltip>
                  <SosSpinner />
                  <SosSpinner size="lg" />
                </div>
              </div>
            </div>
          </div>
        </section>

        <!-- Feedback -->
        <section id="feedback" class="ds-section">
          <div class="ds-section__head">
            <SosEyebrow>COMPONENTS</SosEyebrow>
            <SosTitle as="h2" size="xl">反馈</SosTitle>
          </div>
          <div class="ds-cols">
            <div class="sos-stack">
              <SosNotice tone="info" title="温馨提示">校园祭筹备会将于本周日召开。</SosNotice>
              <SosNotice tone="success" title="提交成功">我们已收到你的报名信息。</SosNotice>
              <SosNotice tone="warning" title="名额紧张">该活动剩余名额不足 10 个。</SosNotice>
              <SosNotice tone="danger" title="提交失败">网络异常，请稍后重试。</SosNotice>
            </div>
            <div class="sos-stack">
              <SosProgress :value="126" :max="200" label="众筹进度" value-label="126 / 200 · 63%" />
              <SosProgress
                :value="100"
                :max="100"
                tone="success"
                label="已完成"
                value-label="100%"
              />
              <SosProgress
                :value="18"
                :max="100"
                tone="danger"
                label="库存预警"
                value-label="18%"
              />
              <SosEmptyState
                title="还没有内容"
                copy="符合条件的结果会显示在这里，换个筛选条件试试。"
              />
            </div>
          </div>
        </section>

        <!-- Overlay -->
        <section id="overlay" class="ds-section">
          <div class="ds-section__head">
            <SosEyebrow>COMPONENTS</SosEyebrow>
            <SosTitle as="h2" size="xl">浮层</SosTitle>
            <SosCopy>模态框、下拉菜单与 Toast 都通过 Teleport 渲染，带焦点、滚动锁与动效。</SosCopy>
          </div>
          <div class="ds-stage ds-stage--surface ds-row">
            <SosButton variant="primary" @click="modalOpen = true">打开模态框</SosButton>
            <SosButton
              variant="secondary"
              @click="toast.push('已加入购物车', { title: '操作成功' })"
            >
              触发 Toast
            </SosButton>
            <SosButton variant="secondary" @click="toast.success('保存成功')">成功 Toast</SosButton>
            <SosButton variant="secondary" @click="toast.danger('删除失败')">危险 Toast</SosButton>
            <SosDropdown>
              <template #trigger>
                <SosButton variant="secondary">下拉菜单 ▾</SosButton>
              </template>
              <button class="sos-menu__item">编辑</button>
              <button class="sos-menu__item">复制链接</button>
              <div class="sos-menu__sep" />
              <button class="sos-menu__item sos-menu__item--danger">删除</button>
            </SosDropdown>
          </div>
        </section>

        <!-- Navigation -->
        <section id="nav" class="ds-section">
          <div class="ds-section__head">
            <SosEyebrow>COMPONENTS</SosEyebrow>
            <SosTitle as="h2" size="xl">导航</SosTitle>
          </div>
          <div class="ds-stage ds-stage--surface sos-stack">
            <SosBreadcrumb :items="crumbs" />
            <SosDivider />
            <div class="ds-row" style="justify-content: space-between">
              <SosTabs v-model="tab" :items="tabItems" />
              <SosTabs v-model="utab" :items="utabItems" variant="underline" />
            </div>
            <SosPagination v-model="page" :page-count="9" />
          </div>
        </section>

        <!-- Business card recipes -->
        <section id="cards" class="ds-section">
          <div class="ds-section__head">
            <SosEyebrow>RECIPES</SosEyebrow>
            <SosTitle as="h2" size="xl">业务卡片 recipe</SosTitle>
            <SosCopy>
              共享 <code>.sos-card</code> 解剖 + 五个内容类型特化 recipe，由
              <code>@haruhi/ui/recipes</code> 输出，内容由业务传入。每张卡片置于它的原生表达模式中——
              配色、圆角、字体、肌理与母题随之切换，这就是"平行世界"在卡片层的落地。
            </SosCopy>
          </div>
          <div class="ds-cards">
            <div
              v-for="card in cardShowcase"
              :key="card.site"
              class="ds-card-cell sos-scope"
              :data-sos-site="card.site"
            >
              <div class="ds-card-cell__bar">
                <SosEyebrow>{{ card.site }}</SosEyebrow>
                <span class="ds-card-cell__name">{{ card.name }}</span>
              </div>
              <div class="ds-card-cell__stage">
                <SosArticleCard
                  v-if="card.site === 'news'"
                  label="NEWS"
                  pinned
                  title="北高校园祭筹备进入最终检查"
                  subtitle="活动组完成摊位、排队与志愿者排班复核"
                  excerpt="校园祭摊位复核完成，志愿者排班表已同步到活动组；现场检查与物料确认在本周内收尾。"
                  :tags="['活动', '校园祭', '公告']"
                  author="编辑部"
                  date="2026-06-23"
                />
                <SosProductCard
                  v-else-if="card.site === 'shop'"
                  title="朝比奈实玖瑠 亚克力立牌"
                  desc="达标后统一排产，订单持续累计中。"
                  :price="147"
                  :original-price="168"
                  badge="预售"
                  :image="ph('fufu', '#eaf0f8', '#7e94b8')"
                  :progress="{ value: 126, max: 200, label: '众筹进度' }"
                  state="预售中"
                >
                  <template #actions>
                    <SosButton variant="primary" size="sm">加入购物车</SosButton>
                  </template>
                </SosProductCard>
                <SosArtworkCard
                  v-else-if="card.site === 'art'"
                  category="个人作品"
                  title="夏日的长门"
                  author="@kimidori"
                  :image="ph('ART', '#cdeee7', '#2f8f84')"
                  :tags="['京阿尼', '插画']"
                  :likes="128"
                />
                <SosBookCard
                  v-else-if="card.site === 'library'"
                  title="凉宫春日的忧郁"
                  author="谷川流"
                  color="#dfe9f4"
                  badge="卷一"
                />
                <SosExamCard
                  v-else
                  subject="语文"
                  title="北高第一学期期末模拟卷"
                  score="优秀"
                  :meta="['20 题', '45 分钟', '满分 100']"
                />
              </div>
            </div>
          </div>
        </section>

        <!-- Expressions -->
        <section id="expressions" class="ds-section">
          <div class="ds-section__head">
            <SosEyebrow>EXPRESSION</SosEyebrow>
            <SosTitle as="h2" size="xl">五个平行世界</SosTitle>
            <SosCopy>
              同一张卡片解剖，在五种表达模式下的不同气质——配色、圆角、阴影与字体随语义 token 切换。
            </SosCopy>
          </div>
          <div
            class="ds-swatches"
            style="grid-template-columns: repeat(auto-fill, minmax(15rem, 1fr))"
          >
            <div
              v-for="s in SITES.filter((m) => m.value !== 'base')"
              :key="s.value"
              class="sos-scope"
              :data-sos-site="s.value"
              style="border-radius: var(--sos-radius-lg); overflow: hidden"
            >
              <div style="background: var(--sos-bg-page); padding: var(--sos-space-4)">
                <SosCard interactive>
                  <div class="sos-card__body sos-stack sos-stack--tight">
                    <SosEyebrow>{{ s.value }}</SosEyebrow>
                    <h3 class="sos-title" style="font-size: var(--sos-text-lg)">
                      {{ s.label.split(' ')[0] }}
                    </h3>
                    <SosCopy variant="small">悬停体验各世界的抬升与投影。</SosCopy>
                    <div class="ds-row" style="margin-top: 0.5rem">
                      <SosButton variant="primary" size="sm">行动</SosButton>
                      <SosBadge variant="accent">标记</SosBadge>
                    </div>
                  </div>
                </SosCard>
              </div>
            </div>
          </div>
        </section>

        <!-- Patterns -->
        <section id="patterns" class="ds-section">
          <div class="ds-section__head">
            <SosEyebrow>PATTERNS</SosEyebrow>
            <SosTitle as="h2" size="xl">组合范式</SosTitle>
            <SosCopy>把基础件组合成真实页面切片：一个报名卡。它在任何表达模式下都成立。</SosCopy>
          </div>
          <div style="max-width: 34rem">
            <SosCard>
              <div class="sos-card__body sos-stack">
                <div class="sos-cluster">
                  <div class="sos-stack sos-stack--tight">
                    <SosEyebrow>校园祭 · 报名</SosEyebrow>
                    <h3 class="sos-title" style="font-size: var(--sos-text-xl)">
                      SOS 团摊位志愿者
                    </h3>
                  </div>
                  <SosBadge variant="signal">招募中</SosBadge>
                </div>
                <SosProgress :value="34" :max="40" label="已报名" value-label="34 / 40 人" />
                <SosField label="你的昵称" required>
                  <SosInput placeholder="例如：阿虚" />
                </SosField>
                <SosCheckbox :model-value="true">可参与全程排班</SosCheckbox>
                <SosButton variant="primary" class="sos-button--block">立即报名</SosButton>
              </div>
            </SosCard>
          </div>
        </section>
      </main>
    </div>

    <!-- Overlays -->
    <SosModal v-model:open="modalOpen" title="确认加入购物车">
      <p>将「长门有希 亚克力立牌」加入购物车？库存充足，可随时修改数量。</p>
      <template #footer>
        <SosButton variant="ghost" @click="modalOpen = false">取消</SosButton>
        <SosButton variant="primary" @click="confirmCart">确认</SosButton>
      </template>
    </SosModal>
    <SosToastRegion />
  </div>
</template>
