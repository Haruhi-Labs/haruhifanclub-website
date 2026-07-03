<script setup>
import { ref, computed, onMounted } from 'vue'
import {
  SosButton,
  SosBadge,
  SosChip,
  SosEyebrow,
  SosTitle,
  SosNotice,
  SosSkeleton,
  SosEmptyState,
  SosModal,
  SosPagination,
} from '@haruhi/ui'
import { useUserHub } from './useUserHub.js'
import { useConsoleContext } from './console-context.js'

const ctx = useConsoleContext()
const hub = useUserHub(ctx.apiBase)

// 各来源：拉取 + 取列表 + 归一化为统一结构 + 删除。新增来源只需在此登记一项。
const SOURCES = [
  {
    key: 'art',
    label: '画廊',
    fetch: () => hub.art.comments({ page: 1, pageSize: 60 }),
    pick: (r) => r.data || [],
    normalize: (c) => ({
      id: c.id,
      source: 'art',
      body: c.body,
      context: c.artwork_title,
      createdAt: c.created_at,
      status: c.status,
      likes: c.like_total || 0,
    }),
    del: (id) => hub.art.deleteComment(id),
  },
  {
    key: 'fiction',
    label: '同人文',
    fetch: () => hub.fiction.comments({ page: 1, pageSize: 60 }),
    pick: (r) => r.comments || [],
    normalize: (c) => ({
      id: c.id,
      source: 'fiction',
      body: c.body,
      context: c.storyTitle,
      createdAt: c.createdAt,
      status: 'visible',
      likes: 0,
    }),
    del: (id) => hub.fiction.deleteComment(id),
  },
]
const labelOf = (key) => SOURCES.find((s) => s.key === key)?.label || key

const all = ref([]) // 归一化并合并后的全部评论
const loading = ref(true)
const error = ref('')
const okMsg = ref('')
const active = ref('all') // 当前来源筛选（胶囊）

const counts = computed(() => {
  const c = { all: all.value.length }
  for (const s of SOURCES) c[s.key] = all.value.filter((x) => x.source === s.key).length
  return c
})
// 有评论的来源；仅当≥2 个来源有数据时才显示分类胶囊（单一来源无需分类）
const availableSources = computed(() => SOURCES.filter((s) => counts.value[s.key] > 0))
const showPills = computed(() => availableSources.value.length >= 2)

const filtered = computed(() =>
  active.value === 'all' ? all.value : all.value.filter((x) => x.source === active.value)
)

// 合并后客户端分页
const page = ref(1)
const pageSize = 20
const totalPages = computed(() => Math.max(1, Math.ceil(filtered.value.length / pageSize)))
const pageItems = computed(() =>
  filtered.value.slice((page.value - 1) * pageSize, page.value * pageSize)
)
function go(p) {
  page.value = Math.min(Math.max(1, p), totalPages.value)
}
function setActive(k) {
  if (active.value === k) return
  active.value = k
  page.value = 1
}

async function load() {
  loading.value = true
  error.value = ''
  try {
    // 各来源独立拉取，单个失败不影响其它（如某模块无评论端点）
    const results = await Promise.allSettled(SOURCES.map((s) => s.fetch()))
    const merged = []
    results.forEach((res, i) => {
      if (res.status === 'fulfilled') {
        for (const raw of SOURCES[i].pick(res.value)) merged.push(SOURCES[i].normalize(raw))
      }
    })
    merged.sort((a, b) => new Date(b.createdAt) - new Date(a.createdAt))
    all.value = merged
    // 当前筛选来源已无数据则回到「全部」
    if (active.value !== 'all' && !counts.value[active.value]) active.value = 'all'
    if (page.value > totalPages.value) page.value = totalPages.value
  } catch (e) {
    error.value = e?.message || '加载失败'
  } finally {
    loading.value = false
  }
}
onMounted(load)

const removing = ref(null)
async function confirmRemove() {
  const c = removing.value
  const src = SOURCES.find((s) => s.key === c.source)
  try {
    await src.del(c.id)
    okMsg.value = '评论已删除'
    removing.value = null
    await load()
  } catch (e) {
    error.value = e?.message || '操作失败'
    removing.value = null
  }
}
</script>

<template>
  <div class="sos-stack huc-page">
    <header class="sos-stack sos-stack--tight">
      <SosEyebrow>我的内容</SosEyebrow>
      <SosTitle as="h1" size="xl">我的评论</SosTitle>
      <p class="sos-copy">你在应援团各站作品下发表的评论，可在此查看与删除。</p>
    </header>

    <SosNotice v-if="error" tone="danger">{{ error }}</SosNotice>
    <SosNotice v-if="okMsg" tone="success">{{ okMsg }}</SosNotice>

    <!-- 来源分类胶囊 -->
    <div v-if="showPills" class="huc__chips">
      <SosChip :pressed="active === 'all'" @toggle="setActive('all')">
        全部 <span class="huc__chip-n">{{ counts.all }}</span>
      </SosChip>
      <SosChip
        v-for="s in availableSources"
        :key="s.key"
        :pressed="active === s.key"
        @toggle="setActive(s.key)"
      >
        {{ s.label }} <span class="huc__chip-n">{{ counts[s.key] }}</span>
      </SosChip>
    </div>

    <div v-if="loading" class="huc__rows">
      <SosSkeleton v-for="i in 4" :key="i" variant="block" style="height: 3.5rem" />
    </div>
    <SosEmptyState
      v-else-if="!filtered.length"
      title="还没有评论"
      copy="去各站看看作品，留下你的第一条评论吧。"
    />
    <div v-else class="huc__rows">
      <div v-for="c in pageItems" :key="`${c.source}-${c.id}`" class="huc__row">
        <div class="huc__row-main">
          <div class="huc__row-title">{{ c.body }}</div>
          <div class="huc__row-meta">
            <SosBadge variant="default">{{ labelOf(c.source) }}</SosBadge>
            <span v-if="c.context"> · {{ c.context }}</span>
            <SosBadge v-if="c.status === 'hidden'" variant="default">已隐藏</SosBadge>
            <span v-if="c.createdAt"> · {{ String(c.createdAt).slice(0, 10) }}</span>
            <span v-if="c.likes"> · ♥ {{ c.likes }}</span>
          </div>
        </div>
        <div class="huc__row-actions">
          <SosButton v-if="c.status !== 'hidden'" size="sm" variant="ghost" @click="removing = c">
            删除
          </SosButton>
        </div>
      </div>
    </div>

    <SosPagination
      v-if="!loading && totalPages > 1"
      :model-value="page"
      :page-count="totalPages"
      @update:model-value="go"
    />

    <SosModal
      :open="!!removing"
      title="删除评论"
      @update:open="
        (v) => {
          if (!v) removing = null
        }
      "
    >
      <p class="sos-copy">确定删除这条评论吗？删除后会将其从作品下隐藏。</p>
      <template #footer>
        <SosButton variant="ghost" @click="removing = null">取消</SosButton>
        <SosButton variant="danger" @click="confirmRemove">确认删除</SosButton>
      </template>
    </SosModal>
  </div>
</template>

<style scoped>
.huc__chips {
  display: flex;
  flex-wrap: wrap;
  gap: var(--sos-space-2);
}
.huc__chip-n {
  font-variant-numeric: var(--sos-numeric-tabular);
  opacity: 0.65;
}
</style>
