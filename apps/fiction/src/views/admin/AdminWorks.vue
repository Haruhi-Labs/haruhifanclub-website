<script setup>
import { ref, computed, watch, onMounted } from 'vue'
import { RouterLink } from 'vue-router'
import {
  SosEyebrow,
  SosTitle,
  SosChip,
  SosSearch,
  SosBadge,
  SosSwitch,
  SosButton,
  SosSkeleton,
  SosEmptyState,
  SosNotice,
  SosPagination,
  SosModal,
  useToast,
} from '@haruhi/ui'
import CoverImage from '@/components/CoverImage.vue'
import { adminListStories, adminUpdateStory, adminDeleteStory, session } from '@/api'
import { canManage } from '@/lib/admin'
import { categoryLabel, wordLabel, compact, fmtDate } from '@/lib/format'

const toast = useToast()
const user = computed(() => session.state.user)
const mayDelete = computed(() => canManage(user.value))

const FILTERS = [
  { key: '', label: '全部' },
  { key: 'published', label: '已发布' },
  { key: 'draft', label: '草稿' },
  { key: 'hidden', label: '已下架' },
  { key: 'featured', label: '精选' },
]

const status = ref('')
const q = ref('')
const page = ref(1)
const pageSize = 20

const loading = ref(true)
const error = ref('')
const works = ref([])
const total = ref(0)
const totalPages = computed(() => Math.max(1, Math.ceil(total.value / pageSize)))
const busy = ref(new Set()) // 正在操作的作品 id

function statusOf(w) {
  if (w.status === 'hidden') return { v: 'danger', t: '已下架' }
  if (w.chapterCount > 0) return { v: 'success', t: '已发布' }
  return { v: 'outline', t: '草稿' }
}
// 管理员可直接编辑：独立署名作品，或自己创建的作品（成员作品仅可审核，不代改正文）
const editable = (w) => w.authorUserId == null || w.authorUserId === user.value?.id

async function load() {
  loading.value = true
  error.value = ''
  try {
    const r = await adminListStories({ status: status.value, q: q.value, page: page.value, pageSize })
    works.value = r.stories
    total.value = r.pagination?.total ?? r.stories.length
  } catch (e) {
    error.value = e?.message || '加载失败'
    works.value = []
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

async function withBusy(id, fn) {
  if (busy.value.has(id)) return
  busy.value = new Set(busy.value).add(id)
  try {
    await fn()
  } catch (e) {
    toast.danger(e?.message || '操作失败')
  } finally {
    const s = new Set(busy.value)
    s.delete(id)
    busy.value = s
  }
}

async function toggleFeatured(w, v) {
  await withBusy(w.id, async () => {
    await adminUpdateStory(w.id, { featured: v })
    w.featured = v
    toast.success(v ? '已设为首页精选' : '已取消精选')
    if (status.value === 'featured' && !v) load() // 精选筛选下取消则需刷新列表
  })
}

async function toggleHidden(w) {
  const toHidden = w.status !== 'hidden'
  await withBusy(w.id, async () => {
    await adminUpdateStory(w.id, { status: toHidden ? 'hidden' : 'draft' })
    toast.success(toHidden ? '已下架' : '已恢复')
    load()
  })
}

// 删除确认
const removing = ref(null)
async function confirmDelete() {
  const w = removing.value
  if (!w) return
  await withBusy(w.id, async () => {
    await adminDeleteStory(w.id)
    toast.success('作品已删除')
    removing.value = null
    // 删除后当前页可能空了，回退一页
    if (works.value.length === 1 && page.value > 1) page.value -= 1
    else load()
  })
  if (removing.value) removing.value = null
}

onMounted(load)
</script>

<template>
  <div class="aw">
    <header class="aw__head">
      <SosEyebrow>控制台</SosEyebrow>
      <SosTitle as="h1" size="xl">作品管理</SosTitle>
      <p class="aw__sub">设置首页精选、上下架、删除；共 {{ total }} 部作品。</p>
    </header>

    <div class="aw__bar">
      <div class="aw__chips">
        <SosChip
          v-for="f in FILTERS"
          :key="f.key"
          :pressed="status === f.key"
          @toggle="setFilter(f.key)"
        >
          {{ f.label }}
        </SosChip>
      </div>
      <SosSearch v-model="q" placeholder="搜索标题 / 作者" class="aw__search" @search="onSearch" />
    </div>

    <SosNotice v-if="error" tone="danger">{{ error }}</SosNotice>

    <div v-if="loading" class="aw__list">
      <SosSkeleton v-for="i in 5" :key="i" variant="block" style="height: 104px" />
    </div>

    <SosEmptyState v-else-if="!works.length" title="没有作品" copy="换个筛选或搜索词试试。" />

    <div v-else class="aw__list">
      <article v-for="w in works" :key="w.id" class="aw__row" :class="{ 'is-busy': busy.has(w.id) }">
        <div class="aw__cover">
          <CoverImage :path="w.coverPath" :title="w.title" :category="w.category" />
        </div>

        <div class="aw__info">
          <div class="aw__titleline">
            <RouterLink :to="`/story/${w.id}`" class="aw__title">{{ w.title }}</RouterLink>
            <SosBadge :variant="statusOf(w).v">{{ statusOf(w).t }}</SosBadge>
            <SosBadge v-if="w.featured" variant="default">精选</SosBadge>
            <SosBadge v-if="w.authorUserId == null" variant="outline">独立署名</SosBadge>
          </div>
          <div class="aw__meta">
            <span>✍ {{ w.authorName || '佚名' }}</span>
            <span>·</span>
            <span>{{ categoryLabel(w.category) }}</span>
            <span>·</span>
            <span>{{ w.chapterCount }} 章</span>
            <span>·</span>
            <span>{{ wordLabel(w.wordCount) }}</span>
            <span>·</span>
            <span>{{ compact(w.viewCount) }} 阅读</span>
            <span>·</span>
            <span>♥ {{ compact(w.likeCount) }}</span>
            <span>·</span>
            <span>更新于 {{ fmtDate(w.updatedAt) }}</span>
          </div>
        </div>

        <div class="aw__actions">
          <label class="aw__feat">
            <SosSwitch
              :model-value="!!w.featured"
              :disabled="busy.has(w.id)"
              @update:model-value="(v) => toggleFeatured(w, v)"
            />
            <span>精选</span>
          </label>
          <SosButton
            size="sm"
            variant="ghost"
            :disabled="busy.has(w.id)"
            @click="toggleHidden(w)"
          >
            {{ w.status === 'hidden' ? '恢复' : '下架' }}
          </SosButton>
          <RouterLink
            v-if="editable(w)"
            :to="`/write/${w.id}`"
            class="sos-button sos-button--secondary sos-button--sm"
          >
            编辑
          </RouterLink>
          <SosButton
            v-if="mayDelete"
            size="sm"
            variant="ghost"
            class="aw__del"
            :disabled="busy.has(w.id)"
            @click="removing = w"
          >
            删除
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

    <SosModal
      :open="!!removing"
      title="删除作品"
      @update:open="(v) => { if (!v) removing = null }"
    >
      <p class="sos-copy">
        确定删除作品「{{ removing?.title }}」吗？此操作将<strong>永久删除</strong>其全部章节、评论、点赞、收藏与阅读进度，且不可恢复。
      </p>
      <template #footer>
        <SosButton variant="ghost" @click="removing = null">取消</SosButton>
        <SosButton variant="danger" :disabled="removing && busy.has(removing.id)" @click="confirmDelete">
          确认删除
        </SosButton>
      </template>
    </SosModal>
  </div>
</template>

<style scoped>
.aw__head {
  margin-bottom: var(--sos-space-5);
}
.aw__sub {
  color: var(--sos-text-secondary);
  margin: var(--sos-space-2) 0 0;
}
.aw__bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sos-space-4);
  flex-wrap: wrap;
  margin-bottom: var(--sos-space-5);
}
.aw__chips {
  display: flex;
  flex-wrap: wrap;
  gap: var(--sos-space-2);
}
.aw__search {
  max-width: 280px;
  flex: 1 1 220px;
}
.aw__list {
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-3);
}
.aw__row {
  display: flex;
  align-items: center;
  gap: var(--sos-space-4);
  padding: var(--sos-space-4);
  background: var(--sos-bg-surface);
  border: 1px solid var(--sos-border-subtle);
  border-radius: var(--sos-radius-lg);
  transition: opacity 0.15s ease;
}
.aw__row.is-busy {
  opacity: 0.55;
  pointer-events: none;
}
.aw__cover {
  width: 56px;
  flex: none;
}
.aw__info {
  flex: 1;
  min-width: 0;
}
.aw__titleline {
  display: flex;
  align-items: center;
  gap: var(--sos-space-2);
  flex-wrap: wrap;
}
.aw__title {
  font-family: var(--sos-display-family, var(--sos-font-display));
  font-size: var(--sos-text-md);
  color: var(--sos-text-primary);
  text-decoration: none;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.aw__title:hover {
  color: var(--sos-accent);
}
.aw__meta {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: var(--sos-space-2);
  font-size: var(--sos-text-xs);
  color: var(--sos-text-tertiary);
}
.aw__actions {
  display: flex;
  align-items: center;
  gap: var(--sos-space-2);
  flex: none;
}
.aw__feat {
  display: inline-flex;
  align-items: center;
  gap: var(--sos-space-2);
  font-size: var(--sos-text-xs);
  color: var(--sos-text-secondary);
  margin-right: var(--sos-space-1);
}
.aw__del {
  color: var(--sos-danger, #dc2626);
}

@media (max-width: 720px) {
  .aw__row {
    flex-wrap: wrap;
  }
  .aw__info {
    flex-basis: calc(100% - 56px - var(--sos-space-4));
  }
  .aw__actions {
    width: 100%;
    flex-wrap: wrap;
    padding-top: var(--sos-space-2);
    border-top: 1px solid var(--sos-border-subtle);
  }
}
</style>
