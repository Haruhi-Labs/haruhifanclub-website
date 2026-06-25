<template>
  <section class="container-card art-board announcement-board">
    <header class="board-header announcement-header">
      <p class="eyebrow">Club Board</p>
      <h1>公告</h1>
      <p>活动安排与维护通知都在这里汇总，当前使用本地 mock 公告数据。</p>
    </header>

    <div class="announcement-layout">
      <section class="notice-browser panel" aria-label="公告分类与列表">
        <nav class="notice-category-rail" aria-label="公告分类">
          <button
            v-for="category in noticeCategories"
            :key="category.id"
            class="notice-category-tab"
            :class="{ active: activeCategory === category.id }"
            type="button"
            @click="setCategory(category.id)"
          >
            <span>{{ category.label }}</span>
            <b>{{ category.count }}</b>
          </button>
        </nav>

        <div class="notice-list-panel">
          <div class="notice-list-heading">
            <span>{{ currentCategory?.label }}</span>
            <small>新到旧</small>
          </div>

          <button
            v-for="notice in filteredNotices"
            :key="notice.id"
            class="notice-list-item"
            :class="{ active: selectedNotice?.id === notice.id }"
            type="button"
            @click="selectNotice(notice.id)"
          >
            <time :datetime="notice.date">
              <span>{{ notice.month }}</span>
              <b>{{ notice.day }}</b>
            </time>
            <span class="notice-list-copy">
              <strong>{{ notice.title }}</strong>
              <small>{{ notice.summary }}</small>
            </span>
          </button>
        </div>
      </section>

      <article v-if="selectedNotice" class="notice-detail panel">
        <div class="notice-detail-meta">
          <span>{{ selectedNotice.type }}</span>
          <time :datetime="selectedNotice.date">{{ selectedNotice.displayDate }}</time>
        </div>
        <h2>{{ selectedNotice.title }}</h2>
        <p>{{ selectedNotice.body }}</p>

        <div class="notice-detail-tags" aria-label="公告标签">
          <span v-for="tag in selectedNotice.tags" :key="tag">#{{ tag }}</span>
        </div>
      </article>
    </div>
  </section>
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
.announcement-board {
  display: grid;
  gap: 28px;
}

.announcement-header {
  max-width: 760px;
}

.announcement-layout {
  display: grid;
  grid-template-columns: minmax(0, 1.05fr) minmax(320px, 0.95fr);
  gap: 22px;
  align-items: stretch;
}

.notice-browser {
  display: grid;
  grid-template-columns: minmax(108px, 0.36fr) minmax(0, 1fr);
  gap: 16px;
  min-height: 430px;
  padding: 16px;
}

.notice-category-rail {
  display: grid;
  grid-auto-rows: minmax(118px, 1fr);
  gap: 12px;
}

.notice-category-tab {
  position: relative;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  min-width: 0;
  padding: 16px 14px;
  overflow: hidden;
  color: var(--muted);
  text-align: left;
  background:
    linear-gradient(145deg, rgba(255, 255, 255, 0.86), rgba(255, 244, 247, 0.68)),
    radial-gradient(circle at 18% 15%, rgba(245, 51, 93, 0.16), transparent 36%);
  border: 1px solid rgba(245, 51, 93, 0.16);
  border-radius: 18px;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.82);
  cursor: pointer;
  transition:
    transform 0.2s ease,
    border-color 0.2s ease,
    color 0.2s ease,
    box-shadow 0.2s ease,
    background 0.2s ease;
}

.notice-category-tab::after {
  position: absolute;
  inset: auto -24px -30px auto;
  width: 76px;
  height: 76px;
  content: '';
  background: radial-gradient(circle, rgba(89, 168, 255, 0.2), transparent 68%);
  border-radius: 999px;
}

.notice-category-tab:hover,
.notice-category-tab.active {
  color: var(--text);
  border-color: rgba(245, 51, 93, 0.34);
  box-shadow:
    0 16px 34px rgba(245, 51, 93, 0.14),
    inset 0 1px 0 rgba(255, 255, 255, 0.9);
  transform: translateY(-2px);
}

.notice-category-tab.active {
  background:
    linear-gradient(145deg, rgba(255, 255, 255, 0.96), rgba(255, 236, 241, 0.88)),
    radial-gradient(circle at 20% 16%, rgba(245, 51, 93, 0.28), transparent 42%);
}

.notice-category-tab span {
  position: relative;
  z-index: 1;
  font-size: 1rem;
  font-weight: 800;
}

.notice-category-tab b {
  position: relative;
  z-index: 1;
  font-size: 2.2rem;
  line-height: 1;
  color: var(--accent);
}

.notice-list-panel {
  display: flex;
  flex-direction: column;
  min-width: 0;
  padding: 4px;
}

.notice-list-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 6px 6px 14px;
  color: var(--text);
  font-weight: 800;
}

.notice-list-heading small {
  color: var(--muted);
  font-size: 0.8rem;
  font-weight: 700;
}

.notice-list-item {
  display: grid;
  grid-template-columns: 58px minmax(0, 1fr);
  gap: 14px;
  align-items: center;
  width: 100%;
  min-width: 0;
  padding: 14px;
  color: var(--text);
  text-align: left;
  background: rgba(255, 255, 255, 0.58);
  border: 1px solid transparent;
  border-radius: 18px;
  cursor: pointer;
  transition:
    transform 0.2s ease,
    border-color 0.2s ease,
    background 0.2s ease,
    box-shadow 0.2s ease;
}

.notice-list-item + .notice-list-item {
  margin-top: 10px;
}

.notice-list-item:hover,
.notice-list-item.active {
  background: rgba(255, 255, 255, 0.86);
  border-color: rgba(89, 168, 255, 0.28);
  box-shadow: 0 16px 30px rgba(16, 24, 40, 0.1);
  transform: translateX(3px);
}

.notice-list-item time {
  display: grid;
  place-items: center;
  aspect-ratio: 1;
  color: #f5335d;
  background: linear-gradient(155deg, rgba(255, 235, 240, 0.95), rgba(234, 246, 255, 0.9));
  border: 1px solid rgba(245, 51, 93, 0.18);
  border-radius: 16px;
}

.notice-list-item time span {
  font-size: 0.76rem;
  font-weight: 800;
}

.notice-list-item time b {
  margin-top: -8px;
  font-size: 1.35rem;
  line-height: 1;
}

.notice-list-copy {
  display: grid;
  min-width: 0;
  gap: 5px;
}

.notice-list-copy strong,
.notice-list-copy small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.notice-list-copy strong {
  font-size: 1rem;
}

.notice-list-copy small {
  color: var(--muted);
  font-size: 0.84rem;
  line-height: 1.5;
}

.notice-detail {
  display: flex;
  flex-direction: column;
  justify-content: center;
  min-height: 430px;
  padding: clamp(22px, 4vw, 38px);
  overflow: hidden;
  background:
    radial-gradient(circle at 88% 10%, rgba(89, 168, 255, 0.18), transparent 32%),
    linear-gradient(145deg, rgba(255, 255, 255, 0.92), rgba(255, 246, 249, 0.78));
}

.notice-detail-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: center;
  margin-bottom: 18px;
  color: var(--muted);
  font-size: 0.86rem;
  font-weight: 800;
}

.notice-detail-meta span {
  color: #f5335d;
}

.notice-detail h2 {
  max-width: 720px;
  margin: 0;
  color: var(--text);
  font-size: clamp(1.8rem, 4vw, 3rem);
  line-height: 1.08;
}

.notice-detail p {
  max-width: 680px;
  margin: 20px 0 0;
  color: var(--muted);
  font-size: 1rem;
  line-height: 1.9;
}

.notice-detail-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  margin-top: 30px;
}

.notice-detail-tags span {
  padding: 7px 12px;
  color: #f5335d;
  font-size: 0.84rem;
  font-weight: 800;
  background: rgba(245, 51, 93, 0.1);
  border: 1px solid rgba(245, 51, 93, 0.18);
  border-radius: 999px;
}

:global(html.art-lights-out) .notice-category-tab {
  color: rgba(220, 232, 255, 0.72);
  background:
    linear-gradient(145deg, rgba(13, 21, 45, 0.9), rgba(24, 20, 52, 0.82)),
    radial-gradient(circle at 18% 15%, rgba(255, 92, 130, 0.18), transparent 40%);
  border-color: rgba(132, 172, 255, 0.18);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08);
}

:global(html.art-lights-out) .notice-category-tab:hover,
:global(html.art-lights-out) .notice-category-tab.active {
  color: #f5f8ff;
  border-color: rgba(111, 206, 255, 0.38);
  box-shadow:
    0 18px 44px rgba(55, 160, 255, 0.14),
    inset 0 1px 0 rgba(255, 255, 255, 0.12);
}

:global(html.art-lights-out) .notice-list-item {
  color: #f5f8ff;
  background: rgba(13, 21, 45, 0.54);
}

:global(html.art-lights-out) .notice-list-item:hover,
:global(html.art-lights-out) .notice-list-item.active {
  background: rgba(22, 33, 66, 0.78);
  border-color: rgba(111, 206, 255, 0.28);
}

:global(html.art-lights-out) .notice-list-item time {
  color: #ff8ca7;
  background: linear-gradient(155deg, rgba(45, 26, 57, 0.86), rgba(15, 44, 73, 0.82));
  border-color: rgba(255, 140, 167, 0.25);
}

:global(html.art-lights-out) .notice-detail {
  background:
    radial-gradient(circle at 88% 10%, rgba(111, 206, 255, 0.18), transparent 34%),
    linear-gradient(145deg, rgba(12, 20, 44, 0.88), rgba(25, 20, 55, 0.8));
}

:global(html.art-lights-out) .notice-detail h2 {
  color: #f7fbff;
}

:global(html.art-lights-out) .notice-detail p,
:global(html.art-lights-out) .notice-list-copy small,
:global(html.art-lights-out) .notice-list-heading small,
:global(html.art-lights-out) .notice-detail-meta {
  color: rgba(220, 232, 255, 0.72);
}

@media (max-width: 980px) {
  .announcement-layout {
    grid-template-columns: 1fr;
  }

  .notice-detail {
    min-height: 320px;
  }
}

@media (max-width: 640px) {
  .notice-browser {
    grid-template-columns: 1fr;
    min-height: auto;
  }

  .notice-category-rail {
    grid-template-columns: repeat(2, minmax(0, 1fr));
    grid-auto-rows: minmax(96px, auto);
  }

  .notice-list-item {
    grid-template-columns: 52px minmax(0, 1fr);
    padding: 12px;
  }
}
</style>
