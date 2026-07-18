<template>
  <section id="gallery-catalog" ref="catalogRoot" class="gallery-catalog">
    <header class="catalog-header">
      <div>
        <span>ALL ARTWORKS</span>
        <h1>浏览全部作品</h1>
      </div>
      <span v-if="total && !loadingInitial" class="result-count">
        已显示 {{ items.length }} / {{ total }} 件
      </span>
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

      <div class="catalog-actions">
        <button
          class="catalog-refresh"
          type="button"
          :disabled="busy"
          :aria-label="refreshLabel"
          :title="refreshLabel"
          data-sfx="click"
          @click="refreshFeed"
        >
          <RefreshCw
            :size="17"
            :stroke-width="2.2"
            :class="{ spinning: refreshing }"
            aria-hidden="true"
          />
          <span>刷新</span>
        </button>
        <AdvancedFilterBar
          :content="activeContent"
          :source-mode="activeSource"
          :show-time-range="currentCategory === 'popular'"
          :time-range="activeTimeRange"
          @update:content="value => updateAdvancedFilters({ content: value })"
          @update:source-mode="value => updateAdvancedFilters({ source: value })"
          @update:time-range="value => updateAdvancedFilters({ range: value })"
        />
      </div>
    </div>

    <p class="catalog-context" aria-live="polite">{{ categoryDescription }}</p>

    <div v-if="error && !items.length" class="error-box" role="alert">
      <span>{{ error }}</span>
      <button type="button" @click="loadInitial()">重新加载</button>
    </div>
    <div v-else-if="error" class="catalog-inline-error" role="status">{{ error }}</div>

    <ArtworkMasonryFeed
      :items="items"
      :total="total"
      :loading-initial="loadingInitial"
      :tracking-source="trackingSource"
      @open="emit('open', $event)"
    />

    <div ref="loadSentinel" class="feed-tail" aria-live="polite">
      <div v-if="loadingMore" class="batch-loader" role="status">
        <span class="batch-loader__signal" aria-hidden="true">
          <i></i><i></i><i></i>
        </span>
        <small>正在整理下一批作品</small>
      </div>
      <div v-else-if="loadMoreError" class="feed-tail__retry">
        <span>{{ loadMoreError }}</span>
        <button type="button" @click="loadMore">再试一次</button>
      </div>
      <p v-else-if="items.length && !hasMore" class="feed-tail__end">
        已经看到当前筛选下的全部作品
      </p>
    </div>

    <Transition name="back-to-top">
      <button
        v-if="showBackToTop"
        class="back-to-top"
        type="button"
        aria-label="回到页面顶部"
        title="回到顶部"
        @click="scrollToTop"
      >
        <ArrowUp :size="19" :stroke-width="2.4" aria-hidden="true" />
      </button>
    </Transition>
  </section>
</template>

<script setup>
import {
  computed,
  nextTick,
  onActivated,
  onBeforeUnmount,
  onDeactivated,
  onMounted,
  ref,
  watch,
} from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ArrowUp, RefreshCw } from 'lucide-vue-next'
import { api, thumbUrl } from '../services/api.js'
import { useGalleryStore } from '../stores/galleryStore.js'
import AdvancedFilterBar from './AdvancedFilterBar.vue'
import ArtworkMasonryFeed from './ArtworkMasonryFeed.vue'

const BATCH_SIZE = 24
const BATCH_REVEAL_DELAY_MS = 520
const PRELOAD_MARGIN = 900
const categories = [
  { key: 'recommended', label: '推荐' },
  { key: 'popular', label: '人气' },
  { key: 'latest', label: '最新' },
]

const route = useRoute()
const router = useRouter()
const galleryStore = useGalleryStore()
const catalogRoot = ref(null)
const loadSentinel = ref(null)
const items = ref([])
const total = ref(0)
const page = ref(1)
const feedId = ref('')
const hasMore = ref(true)
const personalized = ref(false)
const loadingInitial = ref(false)
const loadingMore = ref(false)
const refreshing = ref(false)
const error = ref('')
const loadMoreError = ref('')
const showBackToTop = ref(false)
const emit = defineEmits(['open'])

let viewActive = false
let initialized = false
let lastLoadedSignature = ''
let requestVersion = 0
let requestController = null
let sentinelObserver = null
let scrollFrame = 0
const artworkRatioCache = new Map()

const currentCategory = computed(() => {
  const requested = String(route.query.category || 'recommended')
  return categories.some(item => item.key === requested) ? requested : 'recommended'
})
const activeContent = computed(() => {
  const requested = String(route.query.content || 'mix')
  return requested === 'haruhi' || requested === 'other' ? requested : 'mix'
})
const activeSource = computed(() => {
  const requested = String(route.query.source || 'all')
  return requested === 'network' || requested === 'personal' ? requested : 'all'
})
const activeTimeRange = computed(() => {
  const requested = String(route.query.range || 'history')
  return requested === 'week' || requested === 'year' ? requested : 'history'
})
const busy = computed(() => loadingInitial.value || loadingMore.value || refreshing.value)
const refreshLabel = computed(() => (
  currentCategory.value === 'recommended' ? '重新排列推荐作品' : '刷新作品列表'
))
const categoryDescription = computed(() => {
  if (currentCategory.value === 'recommended') {
    return personalized.value
      ? '根据你的浏览偏好持续推荐，每一批都从尚未展示的候选作品中重新挑选。'
      : '综合作品质量、新鲜度与探索性持续推荐，每一批都不会与上方作品重复。'
  }
  if (currentCategory.value === 'popular') {
    const rangeLabel = { week: '近一周', year: '近一年', history: '历史' }[activeTimeRange.value]
    return `按${rangeLabel}人气值从高到低持续浏览。`
  }
  return '按审核发布时间从新到旧持续浏览。'
})
const trackingSource = computed(() => {
  if (currentCategory.value === 'popular') return `gallery-popular-${activeTimeRange.value}`
  return `gallery-${currentCategory.value}`
})
const feedSignature = computed(() => JSON.stringify({
  category: currentCategory.value,
  content: activeContent.value,
  source: activeSource.value,
  range: currentCategory.value === 'popular' ? activeTimeRange.value : 'history',
}))

function routeLocation(patch = {}) {
  const category = patch.category ?? currentCategory.value
  const content = patch.content ?? activeContent.value
  const source = patch.source ?? activeSource.value
  const range = patch.range ?? activeTimeRange.value
  return {
    name: 'gallery',
    query: {
      category: category !== 'recommended' ? category : undefined,
      content: content !== 'mix' ? content : undefined,
      source: source !== 'all' ? source : undefined,
      range: category === 'popular' && range !== 'history' ? range : undefined,
    },
  }
}

function selectCategory(category) {
  if (category === currentCategory.value) return
  scrollToTop(false)
  router.push(routeLocation({ category }))
}

function updateAdvancedFilters(patch) {
  scrollToTop(false)
  router.push(routeLocation(patch))
}

function recommendationParams(reset) {
  return {
    feedId: !reset && feedId.value ? feedId.value : undefined,
    content_type: activeContent.value !== 'mix' ? activeContent.value : undefined,
    source_type: activeSource.value !== 'all' ? activeSource.value : undefined,
  }
}

function listParams(targetPage) {
  const params = {
    status: 'approved',
    page: targetPage,
    pageSize: BATCH_SIZE,
    sort: currentCategory.value === 'popular' ? 'popular' : 'time',
    order: 'desc',
  }
  if (activeContent.value !== 'mix') params.content_type = activeContent.value
  if (activeSource.value !== 'all') params.source_type = activeSource.value
  if (currentCategory.value === 'popular') params.range = activeTimeRange.value
  return params
}

async function requestBatch({ reset, targetPage, signal }) {
  if (currentCategory.value === 'recommended') {
    return api.recommendations(BATCH_SIZE, recommendationParams(reset), { signal })
  }
  return api.artworksList(listParams(targetPage), { signal })
}

function artworkImageSource(item) {
  return item?.image_url || item?.imageUrl || item?.url || ''
}

function hasArtworkRatio(item) {
  return Number(item?.image_width) > 0 && Number(item?.image_height) > 0
}

function measureArtworkRatio(item, signal) {
  if (hasArtworkRatio(item) || typeof Image === 'undefined') return Promise.resolve()
  const source = thumbUrl(artworkImageSource(item), 640)
  if (!source) return Promise.resolve()
  const cached = artworkRatioCache.get(source)
  if (cached) {
    item.image_width = cached.width
    item.image_height = cached.height
    return Promise.resolve()
  }

  return new Promise((resolve) => {
    const image = new Image()
    let settled = false
    const finish = (width = 4, height = 3) => {
      if (settled) return
      settled = true
      window.clearTimeout(timeout)
      signal?.removeEventListener('abort', onAbort)
      const measured = { width: Math.max(1, width), height: Math.max(1, height) }
      artworkRatioCache.set(source, measured)
      item.image_width = measured.width
      item.image_height = measured.height
      resolve()
    }
    const onAbort = () => {
      image.onload = null
      image.onerror = null
      image.removeAttribute('src')
      finish()
    }
    const timeout = window.setTimeout(() => finish(), 8000)
    image.decoding = 'async'
    image.onload = () => finish(image.naturalWidth, image.naturalHeight)
    image.onerror = () => finish()
    signal?.addEventListener('abort', onAbort, { once: true })
    image.src = source
  })
}

async function hydrateArtworkRatios(data, signal) {
  const queue = (data || []).filter(item => !hasArtworkRatio(item))
  let cursor = 0
  const workers = Array.from({ length: Math.min(6, queue.length) }, async () => {
    while (cursor < queue.length && !signal?.aborted) {
      const item = queue[cursor]
      cursor += 1
      await measureArtworkRatio(item, signal)
    }
  })
  await Promise.all(workers)
}

async function requestHydratedBatch(options) {
  const response = await requestBatch(options)
  await hydrateArtworkRatios(response.data, options.signal)
  return response
}

function uniqueBatch(data, replace) {
  const seen = replace ? new Set() : new Set(items.value.map(item => String(item.id)))
  return (data || []).filter((item) => {
    const key = String(item?.id)
    if (!key || seen.has(key)) return false
    seen.add(key)
    galleryStore.rememberArtwork(item)
    return true
  })
}

function applyResponse(response, { replace, targetPage }) {
  const cacheWasReset = currentCategory.value === 'recommended'
    && !replace
    && Boolean(response.cacheReset)
  const shouldReplace = replace || cacheWasReset
  const batch = uniqueBatch(response.data, shouldReplace)
  items.value = shouldReplace ? batch : [...items.value, ...batch]
  total.value = Number(response.total || items.value.length)
  page.value = cacheWasReset ? 1 : targetPage
  if (currentCategory.value === 'recommended') {
    feedId.value = response.feedId || ''
    personalized.value = Boolean(response.personalized)
    hasMore.value = Boolean(response.hasMore)
  } else {
    feedId.value = ''
    personalized.value = false
    hasMore.value = page.value * BATCH_SIZE < total.value
  }
  if (cacheWasReset) scrollToTop(false)
}

function delay(ms) {
  return new Promise(resolve => window.setTimeout(resolve, ms))
}

async function loadInitial({ retainItems = false } = {}) {
  const version = ++requestVersion
  requestController?.abort()
  requestController = new AbortController()
  error.value = ''
  loadMoreError.value = ''
  loadingInitial.value = !retainItems
  refreshing.value = retainItems
  if (!retainItems) {
    items.value = []
    total.value = 0
  }
  page.value = 1
  feedId.value = ''
  hasMore.value = true

  try {
    const response = await requestHydratedBatch({
      reset: true,
      targetPage: 1,
      signal: requestController.signal,
    })
    if (version !== requestVersion) return
    applyResponse(response, { replace: true, targetPage: 1 })
    initialized = true
    lastLoadedSignature = feedSignature.value
    await nextTick()
    maybeFillViewport()
  } catch (loadError) {
    if (loadError?.name === 'AbortError' || version !== requestVersion) return
    error.value = '作品加载失败，请稍后重试'
    console.warn('[Gallery] 首页作品流加载失败：', loadError)
  } finally {
    if (version === requestVersion) {
      loadingInitial.value = false
      refreshing.value = false
      nextTick(maybeFillViewport)
    }
  }
}

async function loadMore() {
  if (!viewActive || busy.value || !hasMore.value || !items.value.length) return
  const version = ++requestVersion
  requestController = new AbortController()
  loadingMore.value = true
  loadMoreError.value = ''
  const targetPage = page.value + 1

  try {
    const [response] = await Promise.all([
      requestHydratedBatch({ reset: false, targetPage, signal: requestController.signal }),
      delay(BATCH_REVEAL_DELAY_MS),
    ])
    if (version !== requestVersion) return
    applyResponse(response, { replace: false, targetPage })
    await nextTick()
    maybeFillViewport()
  } catch (loadError) {
    if (loadError?.name === 'AbortError' || version !== requestVersion) return
    loadMoreError.value = '下一批作品没有加载成功'
    console.warn('[Gallery] 首页作品流续载失败：', loadError)
  } finally {
    if (version === requestVersion) {
      loadingMore.value = false
      nextTick(maybeFillViewport)
    }
  }
}

function refreshFeed() {
  if (busy.value) return
  scrollToTop()
  loadInitial({ retainItems: true })
}

function maybeFillViewport() {
  if (
    !viewActive
    || !loadSentinel.value
    || busy.value
    || !hasMore.value
    || error.value
    || loadMoreError.value
  ) return
  if (loadSentinel.value.getBoundingClientRect().top < window.innerHeight + PRELOAD_MARGIN) {
    loadMore()
  }
}

function setupSentinelObserver() {
  sentinelObserver?.disconnect()
  if (!loadSentinel.value || typeof IntersectionObserver === 'undefined') return
  sentinelObserver = new IntersectionObserver((entries) => {
    if (entries.some(entry => entry.isIntersecting)) loadMore()
  }, { rootMargin: `${PRELOAD_MARGIN}px 0px`, threshold: 0 })
  sentinelObserver.observe(loadSentinel.value)
}

function syncScrollState() {
  scrollFrame = 0
  showBackToTop.value = window.scrollY > 900
}

function onScroll() {
  if (scrollFrame) return
  scrollFrame = window.requestAnimationFrame(syncScrollState)
}

function scrollToTop(smooth = true) {
  const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches
  window.scrollTo({ top: 0, behavior: smooth && !reduceMotion ? 'smooth' : 'auto' })
}

function activateFeed() {
  if (viewActive) return
  viewActive = true
  window.addEventListener('scroll', onScroll, { passive: true })
  syncScrollState()
  nextTick(setupSentinelObserver)
  if (!initialized || lastLoadedSignature !== feedSignature.value) loadInitial()
  else nextTick(maybeFillViewport)
}

function deactivateFeed() {
  if (!viewActive) return
  viewActive = false
  sentinelObserver?.disconnect()
  sentinelObserver = null
  window.removeEventListener('scroll', onScroll)
  if (scrollFrame) window.cancelAnimationFrame(scrollFrame)
  scrollFrame = 0
  requestController?.abort()
}

watch(feedSignature, () => {
  if (!viewActive || route.name !== 'gallery') return
  loadInitial()
})

onMounted(activateFeed)
onActivated(activateFeed)
onDeactivated(deactivateFeed)
onBeforeUnmount(deactivateFeed)
</script>

<style scoped>
.gallery-catalog {
  width: min(1500px, calc(100% - 64px));
  min-height: 82vh;
  margin: 0 auto;
  padding: 12px 0 96px;
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
  letter-spacing: 0.08em;
}

.catalog-header h1 {
  margin: 0;
  color: var(--sos-text-primary);
  font-size: 30px;
  font-weight: 950;
  letter-spacing: -0.025em;
}

.result-count {
  color: var(--sos-text-tertiary);
  font-size: 13px;
  letter-spacing: 0;
}

.catalog-toolbar {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  margin-bottom: 8px;
}

.category-tabs,
.catalog-actions {
  display: flex;
  align-items: center;
}

.category-tabs { gap: 25px; }
.catalog-actions { gap: 13px; }

.category-tabs button {
  position: relative;
  padding: 8px 1px;
  color: var(--sos-text-secondary);
  font: inherit;
  font-size: 15px;
  font-weight: 780;
  cursor: pointer;
  background: transparent;
  border: 0;
  transition: color 160ms ease;
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
  transform: scaleX(0.55);
  transition: opacity 180ms ease, transform 180ms cubic-bezier(0.23, 1, 0.32, 1);
}

.category-tabs button:hover,
.category-tabs button.active { color: var(--sos-text-primary); }
.category-tabs button.active::after { opacity: 1; transform: scaleX(1); }

.category-tabs button:focus-visible,
.catalog-refresh:focus-visible,
.feed-tail__retry button:focus-visible,
.error-box button:focus-visible {
  outline: 2px solid color-mix(in srgb, var(--sos-accent) 45%, transparent);
  outline-offset: 3px;
}

.catalog-refresh {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  min-height: 38px;
  padding: 0 12px;
  color: var(--sos-text-secondary);
  font: inherit;
  font-size: 13px;
  font-weight: 800;
  cursor: pointer;
  background: var(--sos-bg-surface);
  border: 1px solid var(--sos-border-default);
  border-radius: 999px;
  transition:
    transform 150ms cubic-bezier(0.23, 1, 0.32, 1),
    color 160ms ease,
    border-color 160ms ease;
}

.catalog-refresh:hover:not(:disabled) { color: var(--sos-accent); border-color: var(--sos-accent); }
.catalog-refresh:active:not(:disabled) { transform: scale(0.97); }
.catalog-refresh:disabled { cursor: wait; opacity: 0.58; }
.catalog-refresh .spinning { animation: catalog-spin 700ms linear infinite; }

.catalog-context {
  min-height: 20px;
  margin: 0 0 20px;
  color: var(--sos-text-tertiary);
  font-size: 12px;
  line-height: 1.6;
}

.error-box,
.catalog-inline-error {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 18px;
  padding: 12px 14px;
  color: #8e1c27;
  background: #fff0f1;
  border: 1px solid #ffc8cd;
  border-radius: 8px;
}

.error-box button,
.feed-tail__retry button {
  padding: 0;
  color: currentColor;
  font: inherit;
  font-weight: 800;
  cursor: pointer;
  background: transparent;
  border: 0;
}

.catalog-inline-error { justify-content: center; font-size: 13px; }

.feed-tail {
  min-height: 104px;
  display: grid;
  place-items: center;
  padding-top: 20px;
}

.batch-loader {
  display: grid;
  place-items: center;
  gap: 9px;
  color: var(--sos-text-tertiary);
}

.batch-loader__signal {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 12px;
}

.batch-loader__signal i {
  width: 6px;
  height: 6px;
  background: color-mix(in srgb, var(--sos-accent) 72%, white);
  border-radius: 50%;
  animation: batch-breathe 680ms ease-in-out infinite alternate;
}

.batch-loader__signal i:nth-child(2) { animation-delay: 110ms; }
.batch-loader__signal i:nth-child(3) { animation-delay: 220ms; }
.batch-loader small { font-size: 11px; }

.feed-tail__retry {
  display: flex;
  align-items: center;
  gap: 12px;
  color: var(--sos-text-secondary);
  font-size: 12px;
}

.feed-tail__retry button { color: var(--sos-accent); }
.feed-tail__end { margin: 0; color: var(--sos-text-tertiary); font-size: 12px; }

.back-to-top {
  position: fixed;
  right: max(22px, env(safe-area-inset-right));
  bottom: max(76px, calc(env(safe-area-inset-bottom) + 66px));
  z-index: 18;
  display: grid;
  width: 44px;
  height: 44px;
  place-items: center;
  padding: 0;
  color: var(--sos-text-primary);
  cursor: pointer;
  background: color-mix(in srgb, var(--sos-bg-surface) 92%, transparent);
  border: 1px solid color-mix(in srgb, var(--sos-accent) 40%, var(--sos-border-default));
  border-radius: 50%;
  box-shadow: 0 12px 30px -14px rgba(17, 52, 61, 0.55);
  backdrop-filter: blur(10px);
  transition:
    transform 150ms cubic-bezier(0.23, 1, 0.32, 1),
    border-color 160ms ease;
}

.back-to-top:hover { border-color: var(--sos-accent); }
.back-to-top:active { transform: scale(0.95); }
.back-to-top:focus-visible { outline: 3px solid color-mix(in srgb, var(--sos-focus) 42%, transparent); outline-offset: 3px; }

.back-to-top-enter-active { transition: opacity 180ms ease, transform 180ms cubic-bezier(0.23, 1, 0.32, 1); }
.back-to-top-leave-active { transition: opacity 120ms ease, transform 120ms cubic-bezier(0.23, 1, 0.32, 1); }
.back-to-top-enter-from,
.back-to-top-leave-to { opacity: 0; transform: translateY(8px) scale(0.96); }

@keyframes catalog-spin { to { transform: rotate(360deg); } }
@keyframes batch-breathe {
  from { opacity: 0.35; transform: translateY(2px); }
  to { opacity: 1; transform: translateY(-2px); }
}

@media (max-width: 860px) {
  .gallery-catalog { width: calc(100% - 28px); padding-top: 4px; }
  .catalog-header { margin-bottom: 12px; padding-bottom: 11px; }
  .catalog-header h1 { font-size: 22px; }
  .catalog-header span:first-child { margin-bottom: 3px; font-size: 9px; }
  .catalog-toolbar { align-items: flex-start; gap: 12px; margin-bottom: 7px; }
  .category-tabs { gap: 20px; }
  .catalog-actions { gap: 8px; }
  .catalog-actions :deep(.advanced-filter-bar__controls) {
    position: absolute;
    top: calc(100% + 8px);
    right: 0;
    left: 0;
    z-index: 5;
    box-sizing: border-box;
    margin-top: 0;
    padding: 10px;
    background: color-mix(in srgb, var(--sos-bg-surface) 96%, transparent);
    border: 1px solid var(--sos-border-default);
    border-radius: 12px;
    box-shadow: 0 18px 36px -24px rgba(17, 52, 61, 0.62);
  }
  .catalog-refresh { width: 36px; height: 36px; min-height: 36px; justify-content: center; padding: 0; }
  .catalog-refresh span { display: none; }
  .result-count { padding-bottom: 2px; white-space: nowrap; }
  .catalog-context { margin-bottom: 14px; }
  .feed-tail { min-height: 88px; }
  .back-to-top {
    right: max(14px, env(safe-area-inset-right));
    bottom: max(64px, calc(env(safe-area-inset-bottom) + 56px));
    width: 42px;
    height: 42px;
  }
}

@media (max-width: 420px) {
  .catalog-header h1 { font-size: 20px; }
  .result-count { font-size: 11px; }
  .category-tabs { gap: 16px; }
  .category-tabs button { font-size: 14px; }
}

@media (prefers-reduced-motion: reduce) {
  .category-tabs button::after,
  .catalog-refresh,
  .back-to-top,
  .back-to-top-enter-active,
  .back-to-top-leave-active { transition: none; }
  .catalog-refresh .spinning { animation-duration: 1.1s; }
  .batch-loader__signal i { animation: none; opacity: 0.7; transform: none; }
}
</style>
