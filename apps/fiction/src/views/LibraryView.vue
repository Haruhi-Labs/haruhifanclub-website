<script setup>
import { ref, reactive, computed, watch, onMounted } from 'vue'
import { useRoute, useRouter, RouterLink } from 'vue-router'
import { SosSearch, SosPagination, SosSkeleton, SosEmptyState, SosButton, SosModal } from '@haruhi/ui'
import StoryCard from '@/components/StoryCard.vue'
import LibraryFilters from '@/components/LibraryFilters.vue'
import { listStories, getTags } from '@/api'
import { CATEGORIES } from '@/lib/format'

const route = useRoute()
const router = useRouter()

const loading = ref(true)
const stories = ref([])
const pagination = ref({ page: 1, totalPages: 1, total: 0 })
const tags = ref([])
const searchInput = ref('')

const SORTS = [
  { key: 'latest', label: '最新' },
  { key: 'updated', label: '最近更新' },
  { key: 'popular', label: '人气' },
  { key: 'views', label: '阅读' },
  { key: 'words', label: '字数' },
]
const COMPLETED = [
  { key: '', label: '全部' },
  { key: '0', label: '连载中' },
  { key: '1', label: '已完结' },
]

const f = reactive({ category: '', sort: 'latest', completed: '', tag: '', q: '', page: 1 })

// 移动端筛选抽屉开关；徽标计数用于「筛选」按钮
const showFilters = ref(false)
const activeFilters = computed(() => [f.category, f.completed, f.tag].filter(Boolean).length)

function resetFilters() {
  searchInput.value = ''
  apply({ category: '', completed: '', tag: '', q: '' })
}

function readQuery() {
  f.category = route.query.category || ''
  f.sort = route.query.sort || 'latest'
  f.completed = route.query.completed || ''
  f.tag = route.query.tag || ''
  f.q = route.query.q || ''
  f.page = Number(route.query.page) || 1
  searchInput.value = f.q
}

let listSeq = 0
async function fetchList() {
  const seq = ++listSeq
  loading.value = true
  try {
    const params = { sort: f.sort, page: f.page, pageSize: 24 }
    if (f.category) params.category = f.category
    if (f.completed) params.completed = f.completed
    if (f.tag) params.tag = f.tag
    if (f.q) params.q = f.q
    const r = await listStories(params)
    if (seq !== listSeq) return // 快速切换筛选时丢弃过期结果，避免乱序覆盖
    stories.value = r.stories
    pagination.value = r.pagination
  } catch {
    if (seq === listSeq) stories.value = []
  } finally {
    if (seq === listSeq) loading.value = false
  }
}

function apply(patch, keepPage = false) {
  Object.assign(f, patch)
  if (!keepPage) f.page = 1
  const q = {}
  if (f.category) q.category = f.category
  if (f.sort !== 'latest') q.sort = f.sort
  if (f.completed) q.completed = f.completed
  if (f.tag) q.tag = f.tag
  if (f.q) q.q = f.q
  if (f.page > 1) q.page = f.page
  router.push({ query: q })
}

watch(
  () => route.query,
  () => {
    readQuery()
    fetchList()
  },
  { immediate: true, deep: true },
)

onMounted(async () => {
  try {
    tags.value = (await getTags(30)).tags
  } catch {
    tags.value = []
  }
})
</script>

<template>
  <div class="fiction-page fiction-page--wide lib">
    <!-- PC 侧栏（≤820px 隐藏，改由「筛选」抽屉承载） -->
    <aside class="lib__side">
      <LibraryFilters
        :filters="f"
        :categories="CATEGORIES"
        :completed-options="COMPLETED"
        :tags="tags"
        @apply="apply"
      />
    </aside>

    <main class="lib__main">
      <div class="lib__bar">
        <!-- 移动端筛选入口（PC 隐藏） -->
        <button class="lib__filter-btn" @click="showFilters = true">
          <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3" />
          </svg>
          筛选
          <span v-if="activeFilters" class="lib__filter-badge">{{ activeFilters }}</span>
        </button>
        <div class="lib__sorts">
          <button
            v-for="s in SORTS"
            :key="s.key"
            class="lib__sort"
            :class="{ on: f.sort === s.key }"
            @click="apply({ sort: s.key })"
          >
            {{ s.label }}
          </button>
        </div>
        <div class="lib__bar-end">
          <span class="lib__count">{{ pagination.total }} 部</span>
          <SosSearch
            class="lib__search"
            v-model="searchInput"
            placeholder="搜索书名 / 作者"
            @search="apply({ q: searchInput })"
            @clear="apply({ q: '' })"
          />
        </div>
      </div>

      <div v-if="f.q || f.tag" class="lib__active">
        <button v-if="f.q" class="lib__activetag" @click="apply({ q: '' })">
          搜索：{{ f.q }} ✕
        </button>
        <button v-if="f.tag" class="lib__activetag" @click="apply({ tag: '' })">
          标签：{{ f.tag }} ✕
        </button>
      </div>

      <div v-if="loading" class="fic-list">
        <div v-for="i in 12" :key="i" class="fic-skel">
          <SosSkeleton variant="title" />
          <SosSkeleton variant="text" />
          <SosSkeleton variant="text" style="width: 60%" />
        </div>
      </div>

      <template v-else-if="stories.length">
        <div class="fic-list">
          <StoryCard v-for="s in stories" :key="s.id" :story="s" />
        </div>
        <div v-if="pagination.totalPages > 1" class="lib__pager">
          <SosPagination
            :model-value="pagination.page"
            :page-count="pagination.totalPages"
            @update:model-value="apply({ page: $event }, true)"
          />
        </div>
      </template>

      <SosEmptyState v-else title="没有找到相关作品" copy="换个筛选条件，或成为第一个在此创作的人。">
        <template #actions>
          <RouterLink to="/write" class="sos-button sos-button--primary">去创作</RouterLink>
        </template>
      </SosEmptyState>
    </main>

    <!-- 移动端筛选抽屉：复用侧栏同一套筛选块 -->
    <SosModal v-model:open="showFilters" title="筛选">
      <LibraryFilters
        :filters="f"
        :categories="CATEGORIES"
        :completed-options="COMPLETED"
        :tags="tags"
        @apply="apply"
      />
      <template #footer>
        <SosButton variant="ghost" @click="resetFilters">重置</SosButton>
        <SosButton variant="primary" @click="showFilters = false">
          查看 {{ pagination.total }} 部作品
        </SosButton>
      </template>
    </SosModal>
  </div>
</template>

<style scoped>
.lib {
  display: grid;
  grid-template-columns: 232px 1fr;
  gap: var(--sos-space-8);
  align-items: start;
}
.lib__side {
  position: sticky;
  top: calc(var(--sos-appbar-height, 64px) + var(--sos-space-4));
}
/* 移动端筛选入口：PC 隐藏，≤820px 显示 */
.lib__filter-btn {
  display: none;
  align-items: center;
  gap: 6px;
  border: 1px solid var(--sos-border-default);
  background: var(--sos-bg-surface);
  color: var(--sos-text-primary);
  cursor: pointer;
  padding: 6px 14px;
  border-radius: var(--sos-radius-full);
  font-size: var(--sos-text-sm);
  font-weight: 600;
}
.lib__filter-btn:hover {
  border-color: var(--sos-accent);
  color: var(--sos-accent);
}
.lib__filter-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  border-radius: var(--sos-radius-full);
  background: var(--sos-accent);
  color: var(--sos-accent-contrast);
  font-size: var(--sos-text-2xs);
  font-weight: 700;
}
/* 精炼工具条：吸顶，排序在左、结果计数 + 搜索在右 */
.lib__bar {
  position: sticky;
  top: var(--sos-appbar-height, 64px);
  z-index: 5;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sos-space-4);
  flex-wrap: wrap;
  padding-block: var(--sos-space-3);
  margin-bottom: var(--sos-space-5);
  background: color-mix(in srgb, var(--sos-bg-page) 86%, transparent);
  backdrop-filter: blur(8px);
  border-bottom: 1px solid var(--sos-border-subtle);
}
.lib__sorts {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}
.lib__sort {
  border: none;
  background: transparent;
  cursor: pointer;
  padding: 6px 14px;
  border-radius: var(--sos-radius-full);
  font-size: var(--sos-text-sm);
  color: var(--sos-text-secondary);
  transition: background 0.15s ease, color 0.15s ease;
}
.lib__sort:hover {
  color: var(--sos-accent);
  background: var(--sos-accent-soft);
}
.lib__sort.on {
  background: var(--sos-accent);
  color: var(--sos-accent-contrast);
  font-weight: 600;
}
.lib__bar-end {
  display: flex;
  align-items: center;
  gap: var(--sos-space-3);
}
.lib__count {
  font-size: var(--sos-text-sm);
  color: var(--sos-text-tertiary);
  white-space: nowrap;
}
.lib__search {
  width: clamp(160px, 22vw, 248px);
}
.lib__active {
  display: flex;
  flex-wrap: wrap;
  gap: var(--sos-space-2);
  margin-bottom: var(--sos-space-5);
}
.lib__activetag {
  border: 1px solid var(--sos-accent);
  background: var(--sos-accent-soft);
  color: var(--sos-accent);
  border-radius: var(--sos-radius-full);
  padding: 4px 12px;
  font-size: var(--sos-text-xs);
  cursor: pointer;
}
.lib__pager {
  display: flex;
  justify-content: center;
  margin-top: var(--sos-space-8);
}

@media (max-width: 820px) {
  .lib {
    grid-template-columns: 1fr;
  }
  /* 移动端隐藏 PC 侧栏，筛选改由「筛选」抽屉承载，主列表占满整屏 */
  .lib__side {
    display: none;
  }
  /* 工具条重排：第一行「筛选 …… 计数 + 搜索」，第二行排序横向滚动，避免挤成一坨 */
  .lib__filter-btn {
    display: inline-flex;
    order: 1;
  }
  .lib__bar-end {
    order: 2;
    margin-left: auto;
  }
  .lib__sorts {
    order: 3;
    flex-basis: 100%;
    flex-wrap: nowrap;
    overflow-x: auto;
    scrollbar-width: none;
  }
  .lib__sorts::-webkit-scrollbar {
    display: none;
  }
  .lib__search {
    flex: 1;
    min-width: 150px;
  }
}
</style>
