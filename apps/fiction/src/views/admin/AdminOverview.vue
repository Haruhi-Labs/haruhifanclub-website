<script setup>
import { ref, onMounted } from 'vue'
import { RouterLink } from 'vue-router'
import { SosEyebrow, SosTitle, SosSkeleton, SosNotice } from '@haruhi/ui'
import { adminOverview } from '@/api'
import { compact } from '@/lib/format'

const loading = ref(true)
const error = ref('')
const data = ref(null)

const groups = () => {
  const d = data.value
  if (!d) return []
  return [
    {
      title: '作品',
      to: '/admin/works',
      items: [
        { label: '总数', value: d.works.total },
        { label: '已发布', value: d.works.published, tone: 'ok' },
        { label: '草稿', value: d.works.draft },
        { label: '已下架', value: d.works.hidden, tone: 'warn' },
        { label: '精选', value: d.works.featured, tone: 'accent' },
      ],
    },
    {
      title: '评论',
      to: '/admin/comments',
      items: [
        { label: '总数', value: d.comments.total },
        { label: '展示中', value: d.comments.visible, tone: 'ok' },
        { label: '已隐藏', value: d.comments.hidden, tone: 'warn' },
      ],
    },
    {
      title: '对外数据',
      items: [
        { label: '总阅读', value: compact(d.totals.views) },
        { label: '总点赞', value: compact(d.totals.likes) },
        { label: '总收藏', value: compact(d.totals.bookmarks) },
        { label: '已发布章节', value: d.totals.chapters },
      ],
    },
  ]
}

onMounted(async () => {
  try {
    data.value = await adminOverview()
  } catch (e) {
    error.value = e?.message || '加载失败'
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div class="ov">
    <header class="ov__head">
      <SosEyebrow>控制台</SosEyebrow>
      <SosTitle as="h1" size="xl">总览</SosTitle>
      <p class="ov__sub">春日文库的作品、评论与对外数据一览。</p>
    </header>

    <SosNotice v-if="error" tone="danger">{{ error }}</SosNotice>

    <div v-if="loading" class="ov__groups">
      <SosSkeleton v-for="i in 3" :key="i" variant="block" style="height: 132px" />
    </div>

    <div v-else-if="data" class="ov__groups">
      <section v-for="g in groups()" :key="g.title" class="ov__group">
        <div class="ov__group-head">
          <h2 class="ov__group-title">{{ g.title }}</h2>
          <RouterLink v-if="g.to" :to="g.to" class="ov__group-more">管理 →</RouterLink>
        </div>
        <div class="ov__cards">
          <div v-for="it in g.items" :key="it.label" class="ov__card" :data-tone="it.tone || ''">
            <b class="ov__value">{{ it.value }}</b>
            <span class="ov__label">{{ it.label }}</span>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.ov__head {
  margin-bottom: var(--sos-space-7);
}
.ov__sub {
  color: var(--sos-text-secondary);
  margin: var(--sos-space-2) 0 0;
}
.ov__groups {
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-6);
}
.ov__group-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  margin-bottom: var(--sos-space-3);
}
.ov__group-title {
  font-family: var(--sos-display-family, var(--sos-font-display));
  font-size: var(--sos-text-lg);
  margin: 0;
}
.ov__group-more {
  font-size: var(--sos-text-sm);
  color: var(--sos-accent);
  text-decoration: none;
}
.ov__cards {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: var(--sos-space-3);
}
.ov__card {
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-2);
  padding: var(--sos-space-4) var(--sos-space-5);
  background: var(--sos-bg-surface);
  border: 1px solid var(--sos-border-subtle);
  border-radius: var(--sos-radius-lg);
}
.ov__value {
  font-family: var(--sos-font-sans);
  font-variant-numeric: var(--sos-numeric-tabular);
  font-size: var(--sos-text-2xl);
  line-height: 1;
  color: var(--sos-text-primary);
}
.ov__label {
  font-size: var(--sos-text-xs);
  color: var(--sos-text-tertiary);
}
.ov__card[data-tone='ok'] .ov__value {
  color: var(--sos-success, #16a34a);
}
.ov__card[data-tone='warn'] .ov__value {
  color: var(--sos-warning, #d97706);
}
.ov__card[data-tone='accent'] .ov__value {
  color: var(--sos-accent);
}
</style>
