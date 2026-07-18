<template>
  <section class="search-page">
    <header class="search-page__header">
      <div>
        <button class="back-button" type="button" data-sfx="click" @click="router.push({ name: 'gallery' })">
          <ArrowLeft :size="17" :stroke-width="2.3" aria-hidden="true" />
          <span>返回画廊</span>
        </button>
        <h1>{{ query ? `“${query}”的搜索结果` : '搜索作品' }}</h1>
      </div>
      <span v-if="query && !store.loading" class="result-count">共 {{ store.total }} 件作品</span>
    </header>

    <div v-if="store.error" class="error-box">{{ store.error }}</div>

    <ArtworkShelf
      v-else
      :items="query ? store.list : []"
      :loading="store.loading"
      tracking-source="search"
      @open="openItem"
    />

    <GalleryPagination
      v-if="query"
      :page="store.page"
      :page-count="pageCount"
      :loading="store.loading"
      @go-page="goPage"
    />
  </section>
</template>

<script setup>
import { computed, onMounted, onUnmounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ArrowLeft } from 'lucide-vue-next'
import { useGalleryStore } from '../stores/galleryStore.js'
import ArtworkShelf from '../components/ArtworkShelf.vue'
import GalleryPagination from '../components/GalleryPagination.vue'

const route = useRoute()
const router = useRouter()
const store = useGalleryStore()

const query = computed(() => String(route.query.q || '').trim())
const searchField = computed(() => String(route.query.field || 'all'))
const pageCount = computed(() => Math.max(1, Math.ceil((store.total || 0) / store.limit)))

function updatePageSize(reload = true) {
  const nextLimit = window.innerWidth <= 768 ? 8 : 12
  if (store.limit === nextLimit) return
  store.limit = nextLimit
  if (reload && query.value) loadResults()
}

async function loadResults() {
  if (!query.value) {
    store.list = []
    store.total = 0
    store.hasMore = false
    return
  }

  store.setFilters({
    q: query.value,
    searchField: searchField.value,
    sortMode: 'time',
    page: Math.max(1, Number(route.query.page || 1))
  })
  await store.load()
}

function goPage(page) {
  router.push({
    query: {
      ...route.query,
      artwork: undefined,
      page: page > 1 ? page : undefined
    }
  })
  window.scrollTo({ top: 0, behavior: 'smooth' })
}

function openItem(item) {
  router.push({ name: 'artwork-detail', params: { id: item.id } })
}

let resizeTimer = 0
function onResize() {
  window.clearTimeout(resizeTimer)
  resizeTimer = window.setTimeout(() => updatePageSize(true), 150)
}

watch([() => route.query.q, () => route.query.field, () => route.query.page], loadResults)
onMounted(() => {
  updatePageSize(false)
  loadResults()
  window.addEventListener('resize', onResize)
})

onUnmounted(() => {
  window.removeEventListener('resize', onResize)
  window.clearTimeout(resizeTimer)
})
</script>

<style scoped>
.search-page {
  width: min(1450px, calc(100% - 80px));
  min-height: 65vh;
  margin: 0 auto;
  padding: 10px 0 88px;
}

.search-page__header {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 20px;
  margin-bottom: 24px;
}

.search-page__header h1 {
  margin: 13px 0 0;
  color: var(--sos-text-primary);
  font-size: 27px;
  font-weight: 950;
  line-height: 1.3;
}

.back-button {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 0;
  color: var(--sos-text-secondary);
  font: inherit;
  font-size: 14px;
  font-weight: 750;
  cursor: pointer;
  background: transparent;
  border: 0;
}

.back-button:hover { color: var(--sos-text-primary); }
.result-count { color: var(--sos-text-tertiary); font-size: 13px; }

.error-box {
  padding: 12px;
  color: #a31621;
  background: #fff0f1;
  border: 1px solid #ffc8cd;
  border-radius: 8px;
}

@media (max-width: 768px) {
  .search-page {
    width: calc(100% - 28px);
    padding-top: 4px;
  }

  .search-page__header { align-items: flex-start; margin-bottom: 18px; }
  .search-page__header h1 { font-size: 20px; overflow-wrap: anywhere; }
  .result-count { flex: 0 0 auto; padding-top: 35px; }
}
</style>
