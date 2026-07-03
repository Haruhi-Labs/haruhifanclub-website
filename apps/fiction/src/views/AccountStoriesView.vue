<script setup>
// 个人控制台 ·「我的同人文」：只读列出用户的同人文作品（已发布 / 草稿 / 已下架），
// 深度管理（章节、发布、编辑正文）跳转到 fiction 自有的「创作中心」，避免与之重复。
import { ref, computed, onMounted } from 'vue'
import { RouterLink } from 'vue-router'
import { SosEyebrow, SosTitle, SosBadge, SosButton, SosSkeleton, SosEmptyState } from '@haruhi/ui'
import CoverImage from '@/components/CoverImage.vue'
import { myStories } from '@/api'
import { compact, wordLabel, fmtDate } from '@/lib/format'

const loading = ref(true)
const stories = ref([])
const filter = ref('all')

const FILTERS = [
  { key: 'all', label: '全部' },
  { key: 'published', label: '已发布' },
  { key: 'draft', label: '草稿' },
  { key: 'hidden', label: '已下架' },
]

// 与创作中心一致的状态判定：下架=hidden；有已发布章节即「已发布」；否则草稿
function statusOf(s) {
  if (s.status === 'hidden') return 'hidden'
  if (s.chapterCount > 0) return 'published'
  return 'draft'
}
function statusBadge(s) {
  return {
    published: { v: 'success', t: '已发布' },
    draft: { v: 'outline', t: '草稿' },
    hidden: { v: 'danger', t: '已下架' },
  }[statusOf(s)]
}

const counts = computed(() => {
  const c = { all: stories.value.length, published: 0, draft: 0, hidden: 0 }
  for (const s of stories.value) c[statusOf(s)]++
  return c
})
const visible = computed(() =>
  filter.value === 'all' ? stories.value : stories.value.filter((s) => statusOf(s) === filter.value)
)

async function load() {
  loading.value = true
  try {
    const r = await myStories()
    stories.value = r.stories
  } catch {
    stories.value = []
  } finally {
    loading.value = false
  }
}
onMounted(load)
</script>

<template>
  <div class="sos-stack huc-page">
    <header class="acs__head">
      <div class="sos-stack sos-stack--tight">
        <SosEyebrow>我的内容</SosEyebrow>
        <SosTitle as="h1" size="xl">我的同人文</SosTitle>
        <p class="sos-copy">你在同人文库发布的作品都在这里。章节与发布管理请前往创作中心。</p>
      </div>
      <RouterLink to="/write" class="sos-button sos-button--secondary sos-button--sm"
        >去创作中心 →</RouterLink
      >
    </header>

    <div class="huc__toolbar">
      <SosButton
        v-for="f in FILTERS"
        :key="f.key"
        size="sm"
        :variant="filter === f.key ? 'primary' : 'ghost'"
        @click="filter = f.key"
      >
        {{ f.label }} <span class="acs__count">{{ counts[f.key] }}</span>
      </SosButton>
    </div>

    <div v-if="loading" class="acs__list">
      <SosSkeleton v-for="i in 3" :key="i" variant="block" style="height: 92px" />
    </div>

    <SosEmptyState
      v-else-if="!visible.length"
      :title="filter === 'all' ? '还没有作品' : '该分类下暂无作品'"
      copy="在创作中心写下你的第一部 SOS 团故事，发布后就会出现在这里。"
    >
      <template #actions>
        <RouterLink to="/write" class="sos-button sos-button--primary">去创作</RouterLink>
      </template>
    </SosEmptyState>

    <ul v-else class="acs__list">
      <li v-for="s in visible" :key="s.id" class="acs__item">
        <RouterLink
          v-if="statusOf(s) === 'published'"
          :to="`/story/${s.id}`"
          class="acs__cover"
          title="查看作品"
        >
          <CoverImage :path="s.coverPath" :title="s.title" :category="s.category" />
        </RouterLink>
        <div v-else class="acs__cover acs__cover--static">
          <CoverImage :path="s.coverPath" :title="s.title" :category="s.category" />
        </div>

        <div class="acs__body">
          <div class="acs__top">
            <h3 class="acs__title">{{ s.title || '未命名作品' }}</h3>
            <SosBadge :variant="statusBadge(s).v">{{ statusBadge(s).t }}</SosBadge>
          </div>
          <p class="acs__summary">{{ s.summary || '暂无简介' }}</p>
          <div class="acs__meta">
            <span>{{ s.chapterTotal }} 章（{{ s.chapterCount }} 已发布）</span>
            <span>·</span>
            <span>{{ wordLabel(s.wordCount) }}</span>
            <span>·</span>
            <span>{{ compact(s.viewCount) }} 阅读</span>
            <span>·</span>
            <span>更新于 {{ fmtDate(s.updatedAt) }}</span>
          </div>
        </div>

        <RouterLink :to="`/write/${s.id}`" class="acs__manage">管理 ›</RouterLink>
      </li>
    </ul>
  </div>
</template>

<style scoped>
/* .huc-page 是 grid，网格子项默认 min-width:auto 不收缩；显式置 0 才能约束到列宽，
   否则长标题/元信息会把行撑出视口（移动端「管理」被挤到屏幕外） */
.huc-page > * {
  min-width: 0;
}
.acs__head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--sos-space-4);
  flex-wrap: wrap;
}
.acs__count {
  font-variant-numeric: var(--sos-numeric-tabular);
  opacity: 0.7;
}
.acs__list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-3);
}
.acs__item {
  display: flex;
  gap: var(--sos-space-4);
  align-items: center;
  padding: var(--sos-space-3);
  background: var(--sos-bg-surface);
  border: 1px solid var(--sos-border-subtle);
  border-radius: var(--sos-radius-lg);
  transition:
    border-color 0.15s ease,
    box-shadow 0.15s ease;
}
.acs__item:hover {
  border-color: var(--sos-border-strong);
  box-shadow: var(--sos-shadow-card);
}
.acs__cover {
  width: 58px;
  flex: none;
  border-radius: var(--sos-media-radius, 10px);
  overflow: hidden;
  text-decoration: none;
}
.acs__body {
  flex: 1;
  min-width: 0;
}
.acs__top {
  display: flex;
  align-items: center;
  gap: var(--sos-space-3);
}
.acs__title {
  margin: 0;
  min-width: 0; /* 允许在 flex 行内收缩，长标题才会省略号截断，不挤走右侧「管理」 */
  font-family: var(--sos-display-family, var(--sos-font-display));
  font-size: var(--sos-text-md);
  font-weight: 700;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.acs__summary {
  margin: 4px 0;
  color: var(--sos-text-tertiary);
  font-size: var(--sos-text-sm);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.acs__meta {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  font-size: var(--sos-text-xs);
  color: var(--sos-text-tertiary);
}
.acs__manage {
  flex: none;
  color: var(--sos-accent);
  font-size: var(--sos-text-sm);
  font-weight: 600;
  text-decoration: none;
  white-space: nowrap;
}
.acs__manage:hover {
  text-decoration: underline;
}
@media (max-width: 560px) {
  .acs__manage {
    align-self: flex-start;
  }
  .acs__summary {
    display: none;
  }
}
</style>
