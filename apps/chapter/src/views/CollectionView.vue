<script setup>
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { SosButton, SosEmptyState, SosInput, SosPage, SosPageHeader, SosSelect } from '@haruhi/ui'
import { api } from '@/api'
import ActivityTimeline from '@/components/ActivityTimeline.vue'
import CollectionFilterDrawer from '@/components/CollectionFilterDrawer.vue'
import EventCard from '@/components/EventCard.vue'

const route = useRoute()
const router = useRouter()
const items = ref([])
const branches = ref([])
const loading = ref(false)
const error = ref('')
const total = ref(0)
const pageSize = 24
const query = ref('')
const branch = ref('')
const city = ref('')
const eventType = ref('')
const format = ref('')
const period = ref('upcoming')
const year = ref('')
const filterOpen = ref(false)
const currentYear = new Date().getFullYear()
const kind = computed(() => route.meta.collection || 'timeline')
const page = computed(() => Math.max(1, Number(routeString('page')) || 1))
const pageCount = computed(() => Math.max(1, Math.ceil(total.value / pageSize)))
const title = computed(() => (kind.value === 'events' ? '地方活动' : '活动时间线'))
const hasFilters = computed(() =>
  Boolean(
    query.value.trim() ||
    branch.value ||
    city.value.trim() ||
    year.value ||
    (kind.value === 'events' &&
      (eventType.value.trim() || format.value || period.value !== 'upcoming'))
  )
)
const activeFilterCount = computed(
  () =>
    [
      query.value.trim(),
      branch.value,
      city.value.trim(),
      year.value,
      kind.value === 'events' ? eventType.value.trim() : '',
      kind.value === 'events' ? format.value : '',
      kind.value === 'events' && period.value !== 'upcoming' ? period.value : '',
    ].filter(Boolean).length
)
const branchOptions = computed(() => [
  { label: '全部支部', value: '' },
  ...branches.value.map((entry) => ({ label: entry.branch.name, value: entry.branch.slug })),
])
const yearOptions = [
  { label: '全部年份', value: '' },
  ...Array.from({ length: currentYear - 2006 + 3 }, (_, index) => {
    const value = String(currentYear + 2 - index)
    return { label: `${value} 年`, value }
  }),
]
const formatOptions = [
  { label: '全部形式', value: '' },
  { label: '线下活动', value: 'in_person' },
  { label: '线上活动', value: 'online' },
  { label: '线上线下混合', value: 'hybrid' },
]
const periodOptions = [
  { label: '即将举行', value: 'upcoming' },
  { label: '往期活动', value: 'past' },
  { label: '全部活动', value: 'all' },
]

function routeString(name) {
  const value = route.query[name]
  return Array.isArray(value) ? String(value[0] || '') : String(value || '')
}

function syncFiltersFromRoute() {
  query.value = routeString('q')
  branch.value = routeString('branch')
  city.value = routeString('city')
  eventType.value = routeString('type')
  format.value = routeString('format')
  period.value = routeString('period') || 'upcoming'
  const routeYear = routeString('year')
  year.value = /^\d{4}$/.test(routeYear) ? routeYear : ''
}

function appendYearRange(value) {
  if (!/^\d{4}$/.test(year.value)) return
  value.set('from', `${year.value}-01-01T00:00:00`)
  value.set('to', `${year.value}-12-31T23:59:59`)
}

function apiParams() {
  const value = new URLSearchParams()
  if (query.value.trim()) value.set('q', query.value.trim())
  if (branch.value) value.set('branch', branch.value)
  if (city.value.trim()) value.set('city', city.value.trim())
  appendYearRange(value)
  value.set('page', String(page.value))
  value.set('pageSize', String(pageSize))
  if (kind.value === 'events') {
    if (eventType.value.trim()) value.set('eventType', eventType.value.trim())
    if (format.value) value.set('format', format.value)
    if (period.value) value.set('period', period.value)
  }
  return value.toString()
}

function filterQuery(targetPage = 1) {
  const value = {}
  if (query.value.trim()) value.q = query.value.trim()
  if (branch.value) value.branch = branch.value
  if (city.value.trim()) value.city = city.value.trim()
  if (/^\d{4}$/.test(year.value)) value.year = year.value
  if (targetPage > 1) value.page = String(targetPage)
  if (kind.value === 'events') {
    if (eventType.value.trim()) value.type = eventType.value.trim()
    if (format.value) value.format = format.value
    if (period.value) value.period = period.value
  }
  return value
}

async function load() {
  loading.value = true
  error.value = ''
  try {
    const suffix = apiParams()
    const result = await api.get(`/${kind.value}${suffix ? `?${suffix}` : ''}`)
    items.value = result.items || []
    total.value = Number(result.total ?? items.value.length)
  } catch (reason) {
    items.value = []
    total.value = 0
    error.value = reason?.message || '暂时无法载入内容，请稍后重试。'
  } finally {
    loading.value = false
  }
}

async function applyFilters() {
  const nextQuery = filterQuery(1)
  filterOpen.value = false
  const target = router.resolve({ path: route.path, query: nextQuery }).fullPath
  if (target === route.fullPath) await load()
  else await router.push({ path: route.path, query: nextQuery })
}

async function changePage(targetPage) {
  if (targetPage < 1 || targetPage > pageCount.value || targetPage === page.value) return
  await router.push({ path: route.path, query: filterQuery(targetPage) })
}

async function clearFilters() {
  query.value = ''
  branch.value = ''
  city.value = ''
  eventType.value = ''
  format.value = ''
  period.value = 'upcoming'
  year.value = ''
  await applyFilters()
}

function openFilters() {
  syncFiltersFromRoute()
  filterOpen.value = true
}

function cancelFilters() {
  syncFiltersFromRoute()
  filterOpen.value = false
}

function handleFilterOpenUpdate(open) {
  if (open) openFilters()
  else cancelFilters()
}

watch(
  () => [kind.value, route.fullPath],
  async () => {
    filterOpen.value = false
    syncFiltersFromRoute()
    const result = await api.get('/branches')
    branches.value = result.items || []
    await load()
  },
  { immediate: true }
)
</script>

<template>
  <SosPage contained="wide">
    <SosPageHeader
      :eyebrow="kind === 'events' ? 'NATIONAL EVENTS' : 'ACTIVITY TIMELINE'"
      :title="title"
    >
      <template #actions>
        <SosButton
          variant="secondary"
          class="collection-filter-trigger"
          aria-controls="collection-filter-drawer"
          :aria-expanded="filterOpen"
          @click="openFilters"
        >
          搜索与筛选
          <span v-if="activeFilterCount" class="collection-filter-trigger__count">
            {{ activeFilterCount }}
          </span>
        </SosButton>
      </template>
    </SosPageHeader>

    <CollectionFilterDrawer
      :open="filterOpen"
      :title="`${title}搜索与筛选`"
      @update:open="handleFilterOpenUpdate"
    >
      <form class="collection-filter-form" @submit.prevent="applyFilters">
        <div class="collection-filter-fields">
          <label class="filter-field filter-field--search">
            <span>搜索</span>
            <SosInput
              v-model="query"
              autofocus
              :placeholder="kind === 'events' ? '输入活动或地点' : '输入活动名称'"
            />
          </label>
          <label class="filter-field">
            <span>支部</span>
            <SosSelect v-model="branch" :options="branchOptions" />
          </label>
          <label class="filter-field">
            <span>年份</span>
            <SosSelect v-model="year" :options="yearOptions" />
          </label>
          <label v-if="kind === 'events'" class="filter-field">
            <span>时间范围</span>
            <SosSelect v-model="period" :options="periodOptions" />
          </label>
          <label class="filter-field">
            <span>城市或地区</span>
            <SosInput v-model="city" placeholder="例如上海" />
          </label>
          <template v-if="kind === 'events'">
            <label class="filter-field">
              <span>活动类型</span>
              <SosInput v-model="eventType" placeholder="例如观影会" />
            </label>
            <label class="filter-field">
              <span>活动形式</span>
              <SosSelect v-model="format" :options="formatOptions" />
            </label>
          </template>
        </div>
        <div class="filter-actions collection-filter-actions">
          <button class="sos-button sos-button--primary">应用筛选</button>
          <SosButton type="button" variant="ghost" :disabled="!hasFilters" @click="clearFilters">
            清除筛选
          </SosButton>
        </div>
      </form>
    </CollectionFilterDrawer>

    <p v-if="loading" class="chapter-muted" role="status">正在载入……</p>
    <div v-else-if="error" class="filter-empty">
      <SosEmptyState title="暂时无法载入" :copy="error" />
      <SosButton variant="secondary" @click="load">重新载入</SosButton>
    </div>
    <p v-else class="filter-result-count" aria-live="polite">共找到 {{ total }} 条结果</p>
    <div v-if="!loading && !error && !items.length" class="filter-empty">
      <SosEmptyState
        :title="hasFilters ? '没有符合当前条件的结果' : `暂无${title}`"
        :copy="hasFilters ? '可以调整条件，或清除全部筛选后重新查看。' : '可以稍后再来看看。'"
      />
      <SosButton v-if="hasFilters" variant="secondary" @click="clearFilters">清除筛选</SosButton>
    </div>

    <ActivityTimeline
      v-else-if="!loading && !error && kind === 'timeline'"
      :items="items"
      aria-live="polite"
    />

    <div v-else-if="!loading && !error" class="chapter-feed">
      <EventCard v-for="item in items" :key="item.id" :item="item" show-branch />
    </div>
    <nav v-if="!loading && !error && pageCount > 1" class="collection-pagination" aria-label="分页">
      <SosButton variant="secondary" :disabled="page <= 1" @click="changePage(page - 1)">
        上一页
      </SosButton>
      <span>第 {{ page }} / {{ pageCount }} 页</span>
      <SosButton variant="secondary" :disabled="page >= pageCount" @click="changePage(page + 1)">
        下一页
      </SosButton>
    </nav>
  </SosPage>
</template>
