<script setup>
import { ref, reactive, watch, onMounted } from 'vue'
import { useRoute, useRouter, RouterLink } from 'vue-router'
import { SosSearch, SosPagination, SosSelect, SosSkeleton, SosEmptyState } from '@haruhi/ui'
import StoryCard from '@/components/StoryCard.vue'
import { listStories, getTags } from '@/api'
import { CATEGORIES, RATINGS } from '@/lib/format'

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

const f = reactive({ category: '', sort: 'latest', completed: '', rating: '', tag: '', q: '', page: 1 })

function readQuery() {
  f.category = route.query.category || ''
  f.sort = route.query.sort || 'latest'
  f.completed = route.query.completed || ''
  f.rating = route.query.rating || ''
  f.tag = route.query.tag || ''
  f.q = route.query.q || ''
  f.page = Number(route.query.page) || 1
  searchInput.value = f.q
}

async function fetchList() {
  loading.value = true
  try {
    const params = { sort: f.sort, page: f.page, pageSize: 24 }
    if (f.category) params.category = f.category
    if (f.completed) params.completed = f.completed
    if (f.rating) params.rating = f.rating
    if (f.tag) params.tag = f.tag
    if (f.q) params.q = f.q
    const r = await listStories(params)
    stories.value = r.stories
    pagination.value = r.pagination
  } catch {
    stories.value = []
  } finally {
    loading.value = false
  }
}

function apply(patch, keepPage = false) {
  Object.assign(f, patch)
  if (!keepPage) f.page = 1
  const q = {}
  if (f.category) q.category = f.category
  if (f.sort !== 'latest') q.sort = f.sort
  if (f.completed) q.completed = f.completed
  if (f.rating) q.rating = f.rating
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
    <aside class="lib__side">
      <div class="lib__block">
        <h3 class="lib__block-title">分类</h3>
        <ul class="lib__cats">
          <li>
            <button :class="{ on: !f.category }" @click="apply({ category: '' })">全部作品</button>
          </li>
          <li v-for="c in CATEGORIES" :key="c.slug">
            <button :class="{ on: f.category === c.slug }" @click="apply({ category: c.slug })">
              {{ c.label }}
            </button>
          </li>
        </ul>
      </div>

      <div class="lib__block">
        <h3 class="lib__block-title">连载状态</h3>
        <div class="lib__chips">
          <button
            v-for="c in COMPLETED"
            :key="c.key"
            class="lib__chip"
            :class="{ on: f.completed === c.key }"
            @click="apply({ completed: c.key })"
          >
            {{ c.label }}
          </button>
        </div>
      </div>

      <div class="lib__block">
        <h3 class="lib__block-title">分级</h3>
        <SosSelect
          :model-value="f.rating"
          @update:model-value="apply({ rating: $event })"
        >
          <option value="">全部分级</option>
          <option v-for="r in RATINGS" :key="r.slug" :value="r.slug">{{ r.label }}</option>
        </SosSelect>
      </div>

      <div v-if="tags.length" class="lib__block">
        <h3 class="lib__block-title">热门标签</h3>
        <div class="lib__tagcloud">
          <button
            v-for="t in tags"
            :key="t.name"
            class="lib__tag"
            :class="{ on: f.tag === t.name }"
            @click="apply({ tag: f.tag === t.name ? '' : t.name })"
          >
            {{ t.name }}
          </button>
        </div>
      </div>
    </aside>

    <main class="lib__main">
      <header class="lib__head">
        <div>
          <h1 class="lib__title">书库</h1>
          <p class="lib__count">共 {{ pagination.total }} 部作品</p>
        </div>
        <SosSearch
          v-model="searchInput"
          placeholder="搜索书名 / 作者"
          submit
          @search="apply({ q: searchInput })"
          @clear="apply({ q: '' })"
        />
      </header>

      <div class="lib__toolbar">
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
        <button v-if="f.tag" class="lib__activetag" @click="apply({ tag: '' })">
          标签：{{ f.tag }} ✕
        </button>
      </div>

      <div v-if="loading" class="fiction-grid">
        <div v-for="i in 12" :key="i">
          <SosSkeleton variant="block" style="aspect-ratio: 3/4" />
          <SosSkeleton variant="text" style="margin-top: 8px" />
        </div>
      </div>

      <template v-else-if="stories.length">
        <div class="fiction-grid">
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
  display: flex;
  flex-direction: column;
  gap: var(--sos-space-6);
}
.lib__block-title {
  font-size: var(--sos-text-sm);
  font-weight: 700;
  color: var(--sos-text-secondary);
  margin: 0 0 var(--sos-space-3);
  letter-spacing: var(--sos-tracking-wide);
}
.lib__cats {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.lib__cats button {
  width: 100%;
  text-align: left;
  border: none;
  background: transparent;
  cursor: pointer;
  padding: 7px 12px;
  border-radius: var(--sos-radius-sm);
  color: var(--sos-text-secondary);
  font-size: var(--sos-text-sm);
}
.lib__cats button:hover {
  background: var(--sos-bg-subtle);
}
.lib__cats button.on {
  background: var(--sos-accent-soft);
  color: var(--sos-accent);
  font-weight: 600;
}
.lib__chips,
.lib__tagcloud {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.lib__chip,
.lib__tag {
  border: 1px solid var(--sos-border-default);
  background: var(--sos-bg-surface);
  cursor: pointer;
  padding: 4px 12px;
  border-radius: var(--sos-radius-full);
  font-size: var(--sos-text-xs);
  color: var(--sos-text-secondary);
}
.lib__chip.on,
.lib__tag.on {
  border-color: var(--sos-accent);
  background: var(--sos-accent-soft);
  color: var(--sos-accent);
}
.lib__head {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: var(--sos-space-4);
  flex-wrap: wrap;
  margin-bottom: var(--sos-space-4);
}
.lib__title {
  font-family: var(--sos-display-family, var(--sos-font-display));
  font-size: var(--sos-text-2xl);
  margin: 0;
}
.lib__count {
  color: var(--sos-text-secondary);
  font-size: var(--sos-text-sm);
  margin: 4px 0 0;
}
.lib__head :deep(.sos-search) {
  min-width: 240px;
}
.lib__toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--sos-space-3);
  flex-wrap: wrap;
  padding-bottom: var(--sos-space-5);
  border-bottom: 1px solid var(--sos-border-subtle);
  margin-bottom: var(--sos-space-6);
}
.lib__sorts {
  display: flex;
  gap: 4px;
}
.lib__sort {
  border: none;
  background: transparent;
  cursor: pointer;
  padding: 6px 14px;
  border-radius: var(--sos-radius-full);
  font-size: var(--sos-text-sm);
  color: var(--sos-text-secondary);
}
.lib__sort.on {
  background: var(--sos-accent);
  color: var(--sos-accent-contrast);
  font-weight: 600;
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
  .lib__side {
    position: static;
    flex-direction: row;
    flex-wrap: wrap;
    gap: var(--sos-space-4);
  }
  .lib__block {
    flex: 1 1 45%;
  }
}
</style>
