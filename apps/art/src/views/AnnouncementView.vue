<template>
  <div class="ann-scope">
    <!-- 页头 -->
    <header class="ann-hero">
      <div class="ann-hero__copy">
        <span class="ann-hero__eyebrow">Club Board · 社团公告栏</span>
        <h1 class="ann-hero__title">公告栏</h1>
        <p class="ann-hero__lede">活动安排与维护通知都汇总在此，按分类查看。</p>
      </div>
      <div class="ann-hero__stamp" aria-hidden="true">
        <span class="ann-hero__stamp-ring"></span>
        <span class="ann-hero__stamp-text">SOS</span>
      </div>
    </header>

    <div class="ann-layout">
      <!-- 列表列 -->
      <section class="ann-browse" aria-label="公告分类与列表">
        <div class="ann-tabs" role="tablist" aria-label="公告分类">
          <button
            v-for="category in noticeCategories"
            :key="category.id"
            class="ann-tab"
            :class="[category.id, { 'is-active': activeCategory === category.id }]"
            type="button"
            role="tab"
            :aria-selected="activeCategory === category.id"
            @click="setCategory(category.id)"
          >
            <span class="ann-tab__dot" aria-hidden="true"></span>
            <span class="ann-tab__label">{{ category.label }}</span>
            <b class="ann-tab__count">{{ category.count }}</b>
          </button>
        </div>

        <div class="ann-list">
          <div class="ann-list__head">
            <span>{{ currentCategory?.label }}</span>
            <small>新 → 旧</small>
          </div>
          <button
            v-for="notice in filteredNotices"
            :key="notice.id"
            class="ann-item"
            :class="{ 'is-active': selectedNotice?.id === notice.id }"
            type="button"
            @click="selectNotice(notice.id)"
          >
            <time class="ann-item__date" :datetime="notice.date">
              <span>{{ notice.month }}月</span>
              <b>{{ notice.day }}</b>
            </time>
            <span class="ann-item__copy">
              <span class="ann-item__type" :class="notice.category">{{ notice.type }}</span>
              <strong>{{ notice.title }}</strong>
              <small>{{ notice.summary }}</small>
            </span>
            <span class="ann-item__chevron" aria-hidden="true">›</span>
          </button>
        </div>
      </section>

      <!-- 详情列 -->
      <article v-if="selectedNotice" class="ann-detail" :class="selectedNotice.category">
        <div class="ann-detail__meta">
          <span class="ann-detail__type" :class="selectedNotice.category">{{ selectedNotice.type }}</span>
          <time :datetime="selectedNotice.date">{{ selectedNotice.displayDate }}</time>
        </div>
        <h2 class="ann-detail__title">{{ selectedNotice.title }}</h2>
        <p class="ann-detail__lede">{{ selectedNotice.summary }}</p>
        <div class="ann-detail__rule" aria-hidden="true"></div>
        <p class="ann-detail__body">{{ selectedNotice.body }}</p>
        <div class="ann-detail__tags" aria-label="公告标签">
          <span v-for="tag in selectedNotice.tags" :key="tag">#{{ tag }}</span>
        </div>
      </article>
    </div>
  </div>
</template>

<script setup>
import { computed, ref } from 'vue'

const activeCategory = ref('activity')
const selectedNoticeId = ref(null)

const notices = [
  {
    id: 'activity-20260624',
    category: 'activity',
    date: '2026-06-24',
    month: '06',
    day: '24',
    displayDate: '2026.06.24',
    type: '活动公告',
    title: '夏日应援投稿周开放预告',
    summary: '围绕暑期、社团、宇宙观测主题征集画作。',
    body: '本地开发期间先以 mock 公告展示活动节奏。正式活动规则接入前，投稿页仍保持现有模拟数据流程，方便持续调试画廊体验。',
    tags: ['投稿活动', '暑期主题', '本地模拟']
  },
  {
    id: 'activity-20260620',
    category: 'activity',
    date: '2026-06-20',
    month: '06',
    day: '20',
    displayDate: '2026.06.20',
    type: '活动公告',
    title: '积分兑换功能准备中',
    summary: '兑换页将先用本地项目卡片展示兑换方向。',
    body: '兑换功能目前保持本地 mock 项目，不接真实后端。后续可逐步补充库存、兑换记录、积分校验等流程。',
    tags: ['积分', '兑换', '预告']
  },
  {
    id: 'activity-20260616',
    category: 'activity',
    date: '2026-06-16',
    month: '06',
    day: '16',
    displayDate: '2026.06.16',
    type: '活动公告',
    title: '画廊标签整理计划',
    summary: '整理凉宫、社团、夏日等标签展示层级。',
    body: '为了让作品检索更清楚，后续会基于现有 seed 数据继续调试标签分布与筛选体验，不改变 mock 数据结构。',
    tags: ['标签', '画廊', '筛选']
  },
  {
    id: 'maintenance-20260623',
    category: 'maintenance',
    date: '2026-06-23',
    month: '06',
    day: '23',
    displayDate: '2026.06.23',
    type: '维护公告',
    title: '导航与夜间模式试运行',
    summary: '顶部入口与关灯模式持续观察中。',
    body: '新增页面入口、首页视觉模式和夜间切换仍处于本地调试阶段。若出现样式闪烁，会优先在前端交互层做小步优化。',
    tags: ['开发中', '夜间模式', '导航']
  },
  {
    id: 'maintenance-20260618',
    category: 'maintenance',
    date: '2026-06-18',
    month: '06',
    day: '18',
    displayDate: '2026.06.18',
    type: '维护公告',
    title: 'mock seed 数据稳定性确认',
    summary: '画廊、首页统计和详情弹窗继续使用本地数据。',
    body: '当前开发优先保证 mock seed 数据可用，不依赖真实后端。画廊详情、首页统计和公告展示都应能在离线开发环境中正常打开。',
    tags: ['mock', '本地开发', '稳定性']
  }
]

const categoryLabels = {
  activity: '活动公告',
  maintenance: '维护公告'
}

const noticeCategories = computed(() =>
  Object.entries(categoryLabels).map(([id, label]) => ({
    id,
    label,
    count: notices.filter((notice) => notice.category === id).length
  }))
)

const currentCategory = computed(() =>
  noticeCategories.value.find((category) => category.id === activeCategory.value)
)

const filteredNotices = computed(() =>
  notices
    .filter((notice) => notice.category === activeCategory.value)
    .slice()
    .sort((a, b) => new Date(b.date) - new Date(a.date))
)

const selectedNotice = computed(() => {
  const selected = filteredNotices.value.find((notice) => notice.id === selectedNoticeId.value)
  return selected ?? filteredNotices.value[0]
})

function setCategory(categoryId) {
  activeCategory.value = categoryId
  selectedNoticeId.value = null
}

function selectNotice(noticeId) {
  selectedNoticeId.value = noticeId
}
</script>

<style scoped>
.ann-scope {
  /* art 青绿为主、维护类用琥珀作区分；玻璃面板 + 设计系统 token */
  --ann-accent: var(--sos-accent, hsl(172, 70%, 42%));
  --ann-accent-strong: color-mix(in srgb, var(--ann-accent) 76%, #0b3a36);
  --ann-amber: hsl(35, 92%, 52%);
  --ann-amber-strong: color-mix(in srgb, var(--ann-amber) 72%, #5a3a08);
  --ann-text: var(--sos-text-primary, #16242b);
  --ann-muted: var(--sos-text-secondary, #5c6b72);
  --ann-glass: color-mix(in srgb, #ffffff 70%, transparent);
  --ann-glass-line: color-mix(in srgb, #ffffff 84%, transparent);
  max-width: 1180px;
  margin: 0 auto;
  padding: 0 var(--sos-space-4, 16px) var(--sos-space-8, 48px);
  color: var(--ann-text);
}

/* ---------- 页头 ---------- */
.ann-hero {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sos-space-4);
  padding: var(--sos-space-4) var(--sos-space-1) var(--sos-space-6);
}
.ann-hero__eyebrow {
  display: inline-block;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.14em;
  color: var(--ann-accent-strong);
  padding: 4px 11px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--ann-accent) 13%, transparent);
}
.ann-hero__title {
  margin: 12px 0 6px;
  font-size: clamp(28px, 4vw, 42px);
  font-weight: 850;
  letter-spacing: -0.02em;
}
.ann-hero__lede {
  margin: 0;
  color: var(--ann-muted);
  font-size: 15px;
}
.ann-hero__stamp {
  position: relative;
  display: grid;
  place-items: center;
  width: 92px;
  height: 92px;
  flex-shrink: 0;
  color: var(--ann-accent-strong);
}
.ann-hero__stamp-ring {
  position: absolute;
  inset: 0;
  border: 2px dashed color-mix(in srgb, var(--ann-accent) 45%, transparent);
  border-radius: 50%;
  animation: ann-spin 26s linear infinite;
}
.ann-hero__stamp-text {
  font-size: 22px;
  font-weight: 900;
  letter-spacing: 0.06em;
}
@keyframes ann-spin { to { transform: rotate(360deg); } }

/* ---------- 两列布局 ---------- */
.ann-layout {
  display: grid;
  grid-template-columns: 1fr;
  gap: var(--sos-space-4);
  align-items: start;
}
@media (min-width: 920px) {
  .ann-layout { grid-template-columns: minmax(0, 1.05fr) minmax(360px, 0.95fr); gap: var(--sos-space-5); }
}

/* ---------- 列表列 ---------- */
.ann-browse {
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-3);
}
.ann-tabs {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--sos-space-3);
}
.ann-tab {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 14px 16px;
  border-radius: 16px;
  border: 1px solid var(--ann-glass-line);
  background: var(--ann-glass);
  -webkit-backdrop-filter: blur(14px);
  backdrop-filter: blur(14px);
  cursor: pointer;
  transition: border-color 0.2s, box-shadow 0.2s, transform 0.2s;
  text-align: left;
}
.ann-tab__dot {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  flex-shrink: 0;
  background: var(--ann-accent);
}
.ann-tab.maintenance .ann-tab__dot { background: var(--ann-amber); }
.ann-tab__label { font-size: 14px; font-weight: 700; flex: 1; }
.ann-tab__count {
  font-size: 20px;
  font-weight: 850;
  line-height: 1;
  color: var(--ann-accent-strong);
}
.ann-tab.maintenance .ann-tab__count { color: var(--ann-amber-strong); }
.ann-tab:hover { transform: translateY(-1px); box-shadow: 0 12px 26px -16px rgba(20, 60, 60, 0.4); }
.ann-tab.is-active {
  border-color: color-mix(in srgb, var(--ann-accent) 55%, transparent);
  box-shadow: 0 14px 30px -18px color-mix(in srgb, var(--ann-accent) 60%, transparent);
}
.ann-tab.maintenance.is-active {
  border-color: color-mix(in srgb, var(--ann-amber) 55%, transparent);
  box-shadow: 0 14px 30px -18px color-mix(in srgb, var(--ann-amber) 55%, transparent);
}

.ann-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: var(--sos-space-4);
  border-radius: 20px;
  border: 1px solid var(--ann-glass-line);
  background: var(--ann-glass);
  -webkit-backdrop-filter: blur(16px);
  backdrop-filter: blur(16px);
  box-shadow: 0 20px 44px -28px rgba(20, 60, 60, 0.4), inset 0 1px 0 rgba(255, 255, 255, 0.7);
}
.ann-list__head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  padding: 2px 4px 8px;
  font-size: 13px;
  font-weight: 800;
}
.ann-list__head small { color: var(--ann-muted); font-weight: 700; }

.ann-item {
  display: grid;
  grid-template-columns: 54px minmax(0, 1fr) auto;
  gap: 14px;
  align-items: center;
  width: 100%;
  padding: 12px;
  border-radius: 14px;
  border: 1px solid transparent;
  background: color-mix(in srgb, #ffffff 40%, transparent);
  cursor: pointer;
  text-align: left;
  transition: border-color 0.2s, background 0.2s, transform 0.2s, box-shadow 0.2s;
}
.ann-item:hover {
  background: color-mix(in srgb, #ffffff 72%, transparent);
  transform: translateX(2px);
}
.ann-item.is-active {
  border-color: color-mix(in srgb, var(--ann-accent) 45%, transparent);
  background: color-mix(in srgb, var(--ann-accent) 8%, #ffffff);
}
.ann-item__date {
  display: grid;
  place-items: center;
  aspect-ratio: 1;
  border-radius: 12px;
  color: var(--ann-accent-strong);
  background: color-mix(in srgb, var(--ann-accent) 12%, #ffffff);
  border: 1px solid color-mix(in srgb, var(--ann-accent) 22%, transparent);
}
.ann-item__date span { font-size: 11px; font-weight: 800; }
.ann-item__date b { font-size: 18px; line-height: 1; margin-top: -2px; }
.ann-item__copy { display: grid; gap: 4px; min-width: 0; }
.ann-item__type {
  justify-self: start;
  font-size: 10.5px;
  font-weight: 800;
  padding: 1px 8px;
  border-radius: 999px;
  color: var(--ann-accent-strong);
  background: color-mix(in srgb, var(--ann-accent) 14%, transparent);
}
.ann-item__type.maintenance {
  color: var(--ann-amber-strong);
  background: color-mix(in srgb, var(--ann-amber) 16%, transparent);
}
.ann-item__copy strong { font-size: 14.5px; font-weight: 750; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.ann-item__copy small { font-size: 12.5px; color: var(--ann-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.ann-item__chevron { color: var(--ann-muted); font-size: 20px; font-weight: 700; }

/* ---------- 详情列 ---------- */
.ann-detail {
  position: relative;
  display: flex;
  flex-direction: column;
  padding: clamp(22px, 3vw, 34px);
  border-radius: 22px;
  border: 1px solid var(--ann-glass-line);
  overflow: hidden;
  background:
    radial-gradient(120% 80% at 88% 0%, color-mix(in srgb, var(--ann-accent) 12%, transparent), transparent 60%),
    var(--ann-glass);
  -webkit-backdrop-filter: blur(18px);
  backdrop-filter: blur(18px);
  box-shadow: 0 24px 50px -28px rgba(20, 60, 60, 0.42), inset 0 1px 0 rgba(255, 255, 255, 0.72);
}
.ann-detail.maintenance {
  background:
    radial-gradient(120% 80% at 88% 0%, color-mix(in srgb, var(--ann-amber) 14%, transparent), transparent 60%),
    var(--ann-glass);
}
.ann-detail__meta {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 13px;
  font-weight: 700;
  color: var(--ann-muted);
  margin-bottom: 14px;
}
.ann-detail__type {
  font-size: 11px;
  font-weight: 800;
  padding: 3px 10px;
  border-radius: 999px;
  color: var(--ann-accent-strong);
  background: color-mix(in srgb, var(--ann-accent) 14%, transparent);
}
.ann-detail__type.maintenance { color: var(--ann-amber-strong); background: color-mix(in srgb, var(--ann-amber) 16%, transparent); }
.ann-detail__title {
  margin: 0;
  font-size: clamp(22px, 3.4vw, 34px);
  font-weight: 850;
  line-height: 1.14;
  letter-spacing: -0.01em;
}
.ann-detail__lede {
  margin: 12px 0 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--ann-text);
}
.ann-detail__rule {
  height: 1px;
  margin: 20px 0;
  background: linear-gradient(90deg, color-mix(in srgb, var(--ann-accent) 40%, transparent), transparent);
}
.ann-detail__body {
  margin: 0;
  font-size: 14.5px;
  line-height: 1.85;
  color: var(--ann-muted);
}
.ann-detail__tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 22px;
}
.ann-detail__tags span {
  font-size: 12.5px;
  font-weight: 700;
  padding: 5px 11px;
  border-radius: 999px;
  color: var(--ann-accent-strong);
  background: color-mix(in srgb, var(--ann-accent) 11%, transparent);
  border: 1px solid color-mix(in srgb, var(--ann-accent) 20%, transparent);
}

/* ---------- 关灯（暗色）适配 ---------- */
/* 整条选择器必须放进 :global(...)，否则 Vue scoped 会丢弃括号外的后代选择器。 */
:global(html.art-lights-out .ann-scope) {
  --ann-text: #f5f8ff;
  --ann-muted: rgba(220, 232, 255, 0.7);
  --ann-glass: rgba(15, 24, 46, 0.62);
  --ann-glass-line: rgba(120, 160, 220, 0.18);
  --ann-accent-strong: color-mix(in srgb, var(--ann-accent) 70%, #d8fff4);
}
:global(html.art-lights-out .ann-item) { background: rgba(13, 21, 45, 0.5); }
:global(html.art-lights-out .ann-item:hover) { background: rgba(22, 33, 66, 0.7); }
:global(html.art-lights-out .ann-item__date) { background: rgba(13, 33, 40, 0.7); }

@media (max-width: 560px) {
  .ann-hero__stamp { display: none; }
  .ann-item { grid-template-columns: 48px minmax(0, 1fr); }
  .ann-item__chevron { display: none; }
}
</style>
