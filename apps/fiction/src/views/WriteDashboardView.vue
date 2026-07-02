<script setup>
import { ref, onMounted } from 'vue'
import { useRouter, RouterLink } from 'vue-router'
import { SosButton, SosBadge, SosSkeleton, useToast } from '@haruhi/ui'
import CoverImage from '@/components/CoverImage.vue'
import { myStories, myStats, createStory } from '@/api'
import { compact, wordLabel, fmtDate } from '@/lib/format'

const router = useRouter()
const toast = useToast()
const loading = ref(true)
const creating = ref(false)
const stories = ref([])
const stats = ref(null)

async function load() {
  loading.value = true
  try {
    const [s, st] = await Promise.all([myStories(), myStats()])
    stories.value = s.stories
    stats.value = st
  } catch {
    stories.value = []
  } finally {
    loading.value = false
  }
}

async function create() {
  creating.value = true
  try {
    const r = await createStory({ title: '未命名作品' })
    router.push(`/write/${r.id}`)
  } catch (e) {
    toast.danger(e.message || '创建失败')
    creating.value = false
  }
}

function statusBadge(s) {
  if (s.status === 'published') return { v: 'success', t: '已发布' }
  if (s.status === 'hidden') return { v: 'danger', t: '已下架' }
  return { v: 'outline', t: '草稿' }
}

onMounted(load)
</script>

<template>
  <div class="fiction-page wd">
    <header class="wd__head">
      <div>
        <h1 class="wd__title">创作中心</h1>
        <p class="wd__sub">管理你的同人作品，续写那个夏天。</p>
      </div>
      <SosButton variant="primary" size="lg" :loading="creating" @click="create">＋ 新建作品</SosButton>
    </header>

    <div v-if="stats" class="wd__stats">
      <div class="wd__stat"><b>{{ stats.works.total }}</b><span>作品</span></div>
      <div class="wd__stat"><b>{{ stats.works.published }}</b><span>已发布</span></div>
      <div class="wd__stat"><b>{{ stats.publishedChapters }}</b><span>章节</span></div>
      <div class="wd__stat"><b>{{ wordLabel(stats.totalWords) }}</b><span>总字数</span></div>
      <div class="wd__stat"><b>{{ compact(stats.totalViews) }}</b><span>总阅读</span></div>
      <div class="wd__stat"><b>{{ compact(stats.totalLikes) }}</b><span>获赞</span></div>
    </div>

    <div v-if="loading" class="wd__list">
      <SosSkeleton v-for="i in 3" :key="i" variant="block" style="height: 116px" />
    </div>

    <div v-else-if="stories.length" class="wd__list">
      <RouterLink v-for="s in stories" :key="s.id" :to="`/write/${s.id}`" class="wd__card">
        <div class="wd__card-cover">
          <CoverImage :path="s.coverPath" :title="s.title" :category="s.category" />
        </div>
        <div class="wd__card-body">
          <div class="wd__card-top">
            <h3>{{ s.title }}</h3>
            <SosBadge :variant="statusBadge(s).v">{{ statusBadge(s).t }}</SosBadge>
          </div>
          <p class="wd__card-summary">{{ s.summary || '暂无简介' }}</p>
          <div class="wd__card-meta">
            <span>{{ s.chapterTotal }} 章（{{ s.chapterCount }} 已发布）</span>
            <span>·</span>
            <span>{{ wordLabel(s.wordCount) }}</span>
            <span>·</span>
            <span>{{ compact(s.viewCount) }} 阅读</span>
            <span>·</span>
            <span>更新于 {{ fmtDate(s.updatedAt) }}</span>
          </div>
        </div>
        <span class="wd__card-arrow">管理 ›</span>
      </RouterLink>
    </div>

    <div v-else class="wd__empty">
      <h2>还没有作品</h2>
      <p>你的第一个 SOS 团故事，从这里开始。</p>
      <SosButton variant="primary" size="lg" :loading="creating" @click="create">写第一部作品</SosButton>
    </div>
  </div>
</template>

<style scoped>
.wd__head {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: var(--sos-space-4);
  flex-wrap: wrap;
  margin-bottom: var(--sos-space-6);
}
.wd__title {
  font-family: var(--sos-display-family, var(--sos-font-display));
  font-size: var(--sos-text-2xl);
  margin: 0;
}
.wd__sub {
  color: var(--sos-text-secondary);
  margin: 6px 0 0;
}
.wd__stats {
  display: grid;
  grid-template-columns: repeat(6, 1fr);
  gap: var(--sos-space-3);
  padding: var(--sos-space-5);
  background: var(--sos-bg-surface);
  border: 1px solid var(--sos-border-subtle);
  border-radius: var(--sos-radius-lg);
  margin-bottom: var(--sos-space-7);
}
.wd__stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}
.wd__stat b {
  font-family: var(--sos-font-sans);
  font-variant-numeric: var(--sos-numeric-tabular);
  font-size: var(--sos-text-xl);
  color: var(--sos-text-primary);
}
.wd__stat span {
  font-size: var(--sos-text-xs);
  color: var(--sos-text-tertiary);
}
.wd__list {
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-4);
}
.wd__card {
  display: flex;
  gap: var(--sos-space-4);
  align-items: center;
  padding: var(--sos-space-4);
  background: var(--sos-bg-surface);
  border: 1px solid var(--sos-border-subtle);
  border-radius: var(--sos-radius-lg);
  text-decoration: none;
  color: inherit;
  transition: box-shadow 0.15s ease, border-color 0.15s ease;
}
.wd__card:hover {
  box-shadow: var(--sos-shadow-card);
  border-color: var(--sos-accent);
}
.wd__card-cover {
  width: 64px;
  flex: none;
}
.wd__card-body {
  flex: 1;
  min-width: 0;
}
.wd__card-top {
  display: flex;
  align-items: center;
  gap: var(--sos-space-3);
}
.wd__card-top h3 {
  font-family: var(--sos-display-family, var(--sos-font-display));
  font-size: var(--sos-text-md);
  margin: 0;
}
.wd__card-summary {
  color: var(--sos-text-tertiary);
  font-size: var(--sos-text-sm);
  margin: 4px 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.wd__card-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  font-size: var(--sos-text-xs);
  color: var(--sos-text-tertiary);
}
.wd__card-arrow {
  color: var(--sos-accent);
  font-size: var(--sos-text-sm);
  flex: none;
}
.wd__empty {
  text-align: center;
  padding: var(--sos-space-12) 0;
}
.wd__empty h2 {
  font-family: var(--sos-display-family, var(--sos-font-display));
  margin: 0 0 var(--sos-space-2);
}
.wd__empty p {
  color: var(--sos-text-secondary);
  margin: 0 0 var(--sos-space-5);
}

@media (max-width: 720px) {
  .wd__stats {
    grid-template-columns: repeat(3, 1fr);
    gap: var(--sos-space-4);
  }
  .wd__card-arrow {
    display: none;
  }
}
</style>
