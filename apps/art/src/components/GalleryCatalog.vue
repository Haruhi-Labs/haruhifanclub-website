<template>
  <section id="gallery-catalog" ref="catalogRoot" class="gallery-catalog">
    <header class="catalog-header">
      <div>
        <span>COLLECTION / 04</span>
        <h2>浏览全部作品</h2>
      </div>
      <span v-if="!store.loading" class="result-count">{{ store.total }} 件作品</span>
    </header>

    <div class="catalog-toolbar">
      <nav class="category-tabs" aria-label="作品分类">
        <button
          v-for="item in categories"
          :key="item.key"
          type="button"
          :class="{ active: currentCategory === item.key }"
          :aria-current="currentCategory === item.key ? 'page' : undefined"
          data-sfx="click"
          @click="selectCategory(item.key)"
        >
          {{ item.label }}
        </button>
      </nav>
      <AdvancedFilterBar
        :content="activeContent"
        :source-mode="activeSource"
        :source-locked="currentCategory === 'personal'"
        :show-time-range="currentCategory === 'popular'"
        :time-range="activeTimeRange"
        @update:content="value => updateAdvancedFilters({ content: value })"
        @update:source-mode="value => updateAdvancedFilters({ source: value })"
        @update:time-range="value => updateAdvancedFilters({ range: value })"
      />
    </div>

    <div v-if="store.error" class="error-box">{{ store.error }}</div>
    <ArtworkShelf
      v-else
      :items="store.list"
      :loading="store.loading"
      :tracking-source="catalogTrackingSource"
      @open="emit('open', $event)"
    />

    <GalleryPagination
      :page="store.page"
      :page-count="pageCount"
      :loading="store.loading"
      @go-page="goPage"
    />
  </section>
</template>

<script setup>
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useGalleryStore } from '../stores/galleryStore.js'
import AdvancedFilterBar from './AdvancedFilterBar.vue'
import ArtworkShelf from './ArtworkShelf.vue'
import GalleryPagination from './GalleryPagination.vue'

const categories = [
  { key: 'popular', label: '人气' },
  { key: 'latest', label: '最新' },
  { key: 'personal', label: '个人' }
]

const route = useRoute()
const router = useRouter()
const store = useGalleryStore()
const catalogRoot = ref(null)
const emit = defineEmits(['open'])

const currentCategory = computed(() => {
  const requested = String(route.query.category || 'popular')
  return categories.some(item => item.key === requested) ? requested : 'popular'
})
const activeContent = computed(() => {
  const requested = String(route.query.content || 'mix')
  return requested === 'haruhi' || requested === 'other' ? requested : 'mix'
})
const activeSource = computed(() => {
  if (currentCategory.value === 'personal') return 'personal'
  const requested = String(route.query.source || 'all')
  return requested === 'network' || requested === 'personal' ? requested : 'all'
})
const activeTimeRange = computed(() => {
  const requested = String(route.query.range || 'history')
  return requested === 'week' || requested === 'year' ? requested : 'history'
})
const pageCount = computed(() => Math.max(1, Math.ceil((store.total || 0) / store.limit)))
const catalogTrackingSource = computed(() => (
  currentCategory.value === 'popular'
    ? `catalog-popular-${activeTimeRange.value}`
    : `catalog-${currentCategory.value}`
))

function catalogLocation(patch = {}) {
  const category = patch.category ?? currentCategory.value
  const content = patch.content ?? activeContent.value
  const source = patch.source ?? activeSource.value
  const range = patch.range ?? activeTimeRange.value
  return {
    name: 'gallery',
    query: {
      category,
      content: content !== 'mix' ? content : undefined,
      source: category !== 'personal' && source !== 'all' ? source : undefined,
      range: category === 'popular' && range !== 'history' ? range : undefined,
      page: patch.page > 1 ? patch.page : undefined,
    },
    hash: '#gallery-catalog',
  }
}

function selectCategory(category) {
  router.push(catalogLocation({
    category,
    source: currentCategory.value === 'personal' ? 'all' : activeSource.value,
    page: 1,
  }))
}

function updateAdvancedFilters(patch) {
  router.push(catalogLocation({ ...patch, page: 1 }))
}

function updatePageSize(reload = true) {
  const nextLimit = window.innerWidth <= 768 ? 12 : 16
  if (store.limit === nextLimit) return
  store.limit = nextLimit
  if (reload) loadCategory()
}

async function loadCategory() {
  const category = currentCategory.value
  store.setFilters({
    content: activeContent.value,
    sourceMode: activeSource.value,
    sortMode: category === 'popular' ? 'popular' : 'time',
    timeRange: activeTimeRange.value,
    q: '',
    searchField: 'all',
    page: Math.max(1, Number(route.query.page || 1))
  })
  await store.load()
}

function goPage(page) {
  router.push(catalogLocation({ page }))
  catalogRoot.value?.scrollIntoView({ behavior: 'smooth', block: 'start' })
}

let resizeTimer = 0
function onResize() {
  window.clearTimeout(resizeTimer)
  resizeTimer = window.setTimeout(() => updatePageSize(true), 150)
}

watch(
  [
    () => route.query.category,
    () => route.query.content,
    () => route.query.source,
    () => route.query.range,
    () => route.query.page,
  ],
  loadCategory
)

onMounted(() => {
  updatePageSize(false)
  loadCategory()
  window.addEventListener('resize', onResize)
})

onUnmounted(() => {
  window.removeEventListener('resize', onResize)
  window.clearTimeout(resizeTimer)
})
</script>

<style scoped>
.gallery-catalog {
  width: min(1450px, calc(100% - 80px));
  min-height: 70vh;
  margin: 0 auto;
  padding: 104px 0 88px;
  scroll-margin-top: 72px;
}

.catalog-header {
  display: flex;
  align-items: end;
  justify-content: space-between;
  gap: 20px;
  margin-bottom: 18px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--sos-border-default);
}

.catalog-header span:first-child {
  display: block;
  margin-bottom: 6px;
  color: var(--sos-text-tertiary);
  font-size: 11px;
  font-weight: 800;
}

.catalog-header h2 {
  margin: 0;
  color: var(--sos-text-primary);
  font-size: 25px;
  font-weight: 900;
}

.catalog-toolbar {
  display: flex;
  align-items: center;
  gap: 30px;
  flex-wrap: wrap;
  margin-bottom: 22px;
}

.category-tabs {
  display: flex;
  align-items: center;
  gap: 22px;
}

.category-tabs button {
  position: relative;
  padding: 7px 1px;
  color: var(--sos-text-secondary);
  font-size: 14px;
  font-weight: 750;
  cursor: pointer;
  background: transparent;
  border: 0;
}

.category-tabs button::after {
  content: '';
  position: absolute;
  right: 0;
  bottom: 0;
  left: 0;
  height: 3px;
  background: var(--sos-accent);
  border-radius: 999px;
  opacity: 0;
  transform: scaleX(0.5);
  transition: opacity 0.18s ease, transform 0.18s ease;
}

.category-tabs button:hover,
.category-tabs button.active { color: var(--sos-text-primary); }
.category-tabs button.active::after { opacity: 1; transform: scaleX(1); }
.result-count { color: var(--sos-text-tertiary); font-size: 13px; }

.error-box {
  padding: 12px;
  color: #a31621;
  background: #fff0f1;
  border: 1px solid #ffc8cd;
  border-radius: 8px;
}

@media (max-width: 768px) {
  .gallery-catalog {
    width: calc(100% - 28px);
    padding-top: 68px;
  }

  .catalog-header { margin-bottom: 12px; padding-bottom: 11px; }
  .catalog-header h2 { font-size: 20px; }
  .catalog-header span:first-child { margin-bottom: 3px; font-size: 9px; }
  .catalog-toolbar {
    display: flex;
    gap: 10px;
    justify-content: space-between;
    margin-bottom: 16px;
  }
  .catalog-toolbar :deep(.advanced-filter-bar__controls) {
    flex: 1 0 100%;
    order: 3;
  }
  .category-tabs { gap: 20px; }
  .result-count { padding-bottom: 2px; white-space: nowrap; }
}
</style>
