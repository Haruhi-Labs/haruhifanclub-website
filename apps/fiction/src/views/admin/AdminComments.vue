<script setup>
import { ref, computed, watch, onMounted } from 'vue'
import { RouterLink } from 'vue-router'
import {
  SosEyebrow,
  SosTitle,
  SosChip,
  SosSearch,
  SosBadge,
  SosButton,
  SosSkeleton,
  SosEmptyState,
  SosNotice,
  SosPagination,
  useToast,
} from '@haruhi/ui'
import { adminListComments, adminUpdateComment } from '@/api'
import { fmtDate } from '@/lib/format'

const toast = useToast()

const FILTERS = [
  { key: '', label: '全部' },
  { key: 'visible', label: '展示中' },
  { key: 'hidden', label: '已隐藏' },
]

const status = ref('')
const q = ref('')
const page = ref(1)
const pageSize = 30

const loading = ref(true)
const error = ref('')
const comments = ref([])
const total = ref(0)
const totalPages = computed(() => Math.max(1, Math.ceil(total.value / pageSize)))
const busy = ref(new Set())

async function load() {
  loading.value = true
  error.value = ''
  try {
    const r = await adminListComments({ status: status.value, q: q.value, page: page.value, pageSize })
    comments.value = r.comments
    total.value = r.pagination?.total ?? r.comments.length
  } catch (e) {
    error.value = e?.message || '加载失败'
    comments.value = []
  } finally {
    loading.value = false
  }
}

function setFilter(k) {
  if (status.value === k) return
  status.value = k
  page.value = 1
  load()
}
function onSearch() {
  page.value = 1
  load()
}
watch(page, load)

async function setStatus(c, next) {
  if (busy.value.has(c.id)) return
  busy.value = new Set(busy.value).add(c.id)
  try {
    await adminUpdateComment(c.id, next)
    c.status = next
    toast.success(next === 'hidden' ? '评论已隐藏' : '评论已恢复')
    if (status.value && status.value !== next) load() // 当前按状态筛选时需刷新
  } catch (e) {
    toast.danger(e?.message || '操作失败')
  } finally {
    const s = new Set(busy.value)
    s.delete(c.id)
    busy.value = s
  }
}

// 章节级评论链到具体章节，作品级评论链到作品页
const storyLink = (c) =>
  c.chapterId ? `/story/${c.storyId}/chapter/${c.chapterId}` : `/story/${c.storyId}`

onMounted(load)
</script>

<template>
  <div class="ac">
    <header class="ac__head">
      <SosEyebrow>控制台</SosEyebrow>
      <SosTitle as="h1" size="xl">评论管理</SosTitle>
      <p class="ac__sub">隐藏违规评论或恢复展示；共 {{ total }} 条。</p>
    </header>

    <div class="ac__bar">
      <div class="ac__chips">
        <SosChip
          v-for="f in FILTERS"
          :key="f.key"
          :pressed="status === f.key"
          @toggle="setFilter(f.key)"
        >
          {{ f.label }}
        </SosChip>
      </div>
      <SosSearch v-model="q" placeholder="搜索内容 / 作者" class="ac__search" @search="onSearch" />
    </div>

    <SosNotice v-if="error" tone="danger">{{ error }}</SosNotice>

    <div v-if="loading" class="ac__list">
      <SosSkeleton v-for="i in 6" :key="i" variant="block" style="height: 76px" />
    </div>

    <SosEmptyState v-else-if="!comments.length" title="没有评论" copy="换个筛选或搜索词试试。" />

    <div v-else class="ac__list">
      <article
        v-for="c in comments"
        :key="c.id"
        class="ac__row"
        :class="{ 'is-hidden': c.status === 'hidden', 'is-busy': busy.has(c.id) }"
      >
        <div class="ac__body">
          <p class="ac__text">{{ c.body }}</p>
          <div class="ac__meta">
            <span class="ac__author">{{ c.authorName || '匿名' }}</span>
            <span>·</span>
            <RouterLink :to="storyLink(c)" class="ac__story">《{{ c.storyTitle }}》</RouterLink>
            <span>·</span>
            <span>{{ fmtDate(c.createdAt) }}</span>
            <SosBadge v-if="c.status === 'hidden'" variant="danger">已隐藏</SosBadge>
          </div>
        </div>
        <div class="ac__actions">
          <SosButton
            v-if="c.status === 'hidden'"
            size="sm"
            variant="ghost"
            :disabled="busy.has(c.id)"
            @click="setStatus(c, 'visible')"
          >
            恢复
          </SosButton>
          <SosButton
            v-else
            size="sm"
            variant="ghost"
            class="ac__hide"
            :disabled="busy.has(c.id)"
            @click="setStatus(c, 'hidden')"
          >
            隐藏
          </SosButton>
        </div>
      </article>
    </div>

    <SosPagination
      v-if="!loading && totalPages > 1"
      :model-value="page"
      :page-count="totalPages"
      @update:model-value="(p) => (page = p)"
    />
  </div>
</template>

<style scoped>
.ac__head {
  margin-bottom: var(--sos-space-5);
}
.ac__sub {
  color: var(--sos-text-secondary);
  margin: var(--sos-space-2) 0 0;
}
.ac__bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sos-space-4);
  flex-wrap: wrap;
  margin-bottom: var(--sos-space-5);
}
.ac__chips {
  display: flex;
  flex-wrap: wrap;
  gap: var(--sos-space-2);
}
.ac__search {
  max-width: 280px;
  flex: 1 1 220px;
}
.ac__list {
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-3);
}
.ac__row {
  display: flex;
  align-items: center;
  gap: var(--sos-space-4);
  padding: var(--sos-space-4);
  background: var(--sos-bg-surface);
  border: 1px solid var(--sos-border-subtle);
  border-radius: var(--sos-radius-lg);
  transition: opacity 0.15s ease;
}
.ac__row.is-hidden {
  background: var(--sos-bg-page);
}
.ac__row.is-busy {
  opacity: 0.55;
  pointer-events: none;
}
.ac__body {
  flex: 1;
  min-width: 0;
}
.ac__text {
  margin: 0;
  color: var(--sos-text-primary);
  font-size: var(--sos-text-sm);
  line-height: 1.6;
  word-break: break-word;
}
.ac__row.is-hidden .ac__text {
  color: var(--sos-text-tertiary);
}
.ac__meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
  margin-top: var(--sos-space-2);
  font-size: var(--sos-text-xs);
  color: var(--sos-text-tertiary);
}
.ac__author {
  color: var(--sos-text-secondary);
  font-weight: 600;
}
.ac__story {
  color: var(--sos-accent);
  text-decoration: none;
}
.ac__actions {
  flex: none;
}
.ac__hide {
  color: var(--sos-danger, #dc2626);
}
</style>
