<template>
  <section class="creators-page" aria-labelledby="creators-title">
    <header class="creators-intro">
      <div>
        <span class="creators-intro__eyebrow">ARTISTS</span>
        <h1 id="creators-title">创作者</h1>
        <p>顺着作品认识画廊里的每一位创作者。</p>
      </div>
      <span v-if="total" class="creators-intro__count">{{ total }} 位创作者</span>
    </header>

    <div v-if="error && !creators.length" class="creators-state creators-state--error" role="alert">
      <p>{{ error }}</p>
      <button type="button" data-sfx="click" @click="loadMore">重新加载</button>
    </div>

    <div
      v-else-if="loading && !creators.length"
      class="creator-masonry creator-masonry--skeleton"
      :style="{ '--creator-column-count': creatorColumnCount }"
      aria-hidden="true"
    >
      <article v-for="index in 6" :key="index" class="creator-card creator-card--skeleton">
        <div class="creator-skeleton creator-skeleton--identity"></div>
        <div class="creator-skeleton creator-skeleton--work"></div>
        <div class="creator-skeleton creator-skeleton--work creator-skeleton--short"></div>
      </article>
    </div>

    <div
      v-else-if="creators.length"
      class="creator-masonry"
      :style="{ '--creator-column-count': creatorColumnCount }"
      role="feed"
      :aria-busy="loading"
    >
      <div
        v-for="(column, columnIndex) in creatorColumns"
        :key="columnIndex"
        class="creator-column"
        role="presentation"
      >
        <article
          v-for="entry in column"
          :key="entry.creator.uid"
          class="creator-card"
          :aria-posinset="entry.position + 1"
          :aria-setsize="total"
        >
          <header class="creator-card__header">
            <button
              class="creator-card__identity"
              type="button"
              :aria-label="`查看创作者 ${entry.creator.name} 的个人主页`"
              data-sfx="click"
              @click="openCreator(entry.creator)"
            >
              <span class="creator-card__avatar">
                <img
                  v-if="entry.creator.avatar"
                  :src="entry.creator.avatar"
                  alt=""
                  loading="lazy"
                  decoding="async"
                />
                <UserRound v-else :size="20" aria-hidden="true" />
              </span>
              <span class="creator-card__name">
                <strong>{{ entry.creator.name }}</strong>
                <small>
                  从 {{ entry.creator.totalWorks }} 件作品中推荐 {{ entry.creator.items.length }} 件
                </small>
              </span>
              <ArrowUpRight :size="17" aria-hidden="true" />
            </button>
          </header>

          <div
            class="creator-card__works"
            :class="`creator-card__works--${Math.min(entry.creator.items.length, 3)}`"
            :aria-label="`${entry.creator.name} 的推荐作品`"
          >
            <button
              v-for="(item, artworkIndex) in entry.creator.items"
              :key="item.id"
              class="creator-work"
              type="button"
              :style="{ '--artwork-ratio': artworkRatio(item) }"
              :aria-label="`查看作品：${item.title || '未命名作品'}`"
              data-sfx="click"
              @click="openArtwork(item, entry.position * 3 + artworkIndex)"
            >
              <img
                :src="artworkImage(item)"
                :alt="item.title || '画廊作品'"
                loading="lazy"
                decoding="async"
              />
              <span class="creator-work__label">
                <strong>{{ item.title || '未命名作品' }}</strong>
              </span>
            </button>
          </div>
        </article>
      </div>
    </div>

    <div v-else-if="!loading" class="creators-state">
      <p>还没有可以展示的创作者作品。</p>
    </div>

    <footer v-if="creators.length" ref="sentinel" class="creator-feed-end" aria-live="polite">
      <span v-if="loading" class="creator-feed-end__loading">
        <LoaderCircle :size="17" aria-hidden="true" />
        正在继续寻找创作者
      </span>
      <button v-else-if="error" type="button" data-sfx="click" @click="loadMore">
        加载失败，点击重试
      </button>
      <button v-else-if="hasMore" type="button" data-sfx="click" @click="loadMore">
        继续浏览
      </button>
      <span v-else>已经看完全部创作者</span>
    </footer>
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
} from 'vue'
import { useRouter } from 'vue-router'
import { ArrowUpRight, LoaderCircle, UserRound } from 'lucide-vue-next'
import { api, thumbUrl } from '../services/api.js'
import { trackArtworkOpen } from '../services/recommendationTracker.js'

defineOptions({ name: 'CreatorsView' })

const PAGE_SIZE = 6
const router = useRouter()
const creators = ref([])
const page = ref(0)
const total = ref(0)
const hasMore = ref(true)
const loading = ref(false)
const error = ref('')
const feedId = ref('')
const sentinel = ref(null)
const creatorColumnCount = ref(desiredCreatorColumnCount())
let observer = null
let viewActive = false
let fillFrame = 0
let resizeFrame = 0

function artworkImage(item) {
  return thumbUrl(item?.image_url || item?.imageUrl || item?.url || '', 640)
}

function artworkRatioValue(item) {
  const width = Number(item?.image_width || item?.width || item?.images?.[0]?.width)
  const height = Number(item?.image_height || item?.height || item?.images?.[0]?.height)
  const ratio = width > 0 && height > 0 ? width / height : 4 / 3
  return Math.min(1.8, Math.max(0.72, ratio))
}

function artworkRatio(item) {
  return String(artworkRatioValue(item))
}

function desiredCreatorColumnCount() {
  if (typeof window === 'undefined') return 1
  if (window.innerWidth >= 1280) return 3
  if (window.innerWidth >= 900) return 2
  return 1
}

function estimateCreatorHeight(creator) {
  const items = Array.isArray(creator?.items) ? creator.items : []
  if (!items.length) return 1
  if (items.length === 1) return 0.42 + 1 / artworkRatioValue(items[0])

  const lanes = [0, 0]
  for (const item of items) {
    const lane = lanes[0] <= lanes[1] ? 0 : 1
    lanes[lane] += 1 / artworkRatioValue(item) + 0.04
  }
  return 0.42 + Math.max(...lanes)
}

const creatorColumns = computed(() => {
  const columns = Array.from({ length: creatorColumnCount.value }, () => [])
  const heights = columns.map(() => 0)
  creators.value.forEach((creator, position) => {
    let shortestColumn = 0
    for (let index = 1; index < heights.length; index += 1) {
      if (heights[index] < heights[shortestColumn]) shortestColumn = index
    }
    columns[shortestColumn].push({ creator, position })
    heights[shortestColumn] += estimateCreatorHeight(creator)
  })
  return columns
})

function scheduleColumnSync() {
  if (resizeFrame) window.cancelAnimationFrame(resizeFrame)
  resizeFrame = window.requestAnimationFrame(() => {
    resizeFrame = 0
    creatorColumnCount.value = desiredCreatorColumnCount()
  })
}

function openCreator(creator) {
  if (!creator?.uid) return
  router.push({
    name: 'adventurer-profile',
    params: { uid: creator.uid },
    query: { from: 'creators' },
  })
}

function openArtwork(item, position) {
  trackArtworkOpen(item, { source: 'creator-feed', position })
  router.push({
    name: 'artwork-detail',
    params: { id: item.id },
    query: { from: 'creators' },
  })
}

function scheduleViewportFill() {
  if (!viewActive || !hasMore.value || loading.value || fillFrame) return
  fillFrame = window.requestAnimationFrame(() => {
    fillFrame = 0
    const boundary = window.innerHeight + 640
    if (sentinel.value?.getBoundingClientRect().top < boundary) loadMore()
  })
}

function observeSentinel() {
  observer?.disconnect()
  observer = null
  if (!viewActive || !sentinel.value || typeof IntersectionObserver === 'undefined') return
  observer = new IntersectionObserver((entries) => {
    if (entries.some(entry => entry.isIntersecting)) loadMore()
  }, { rootMargin: '640px 0px', threshold: 0 })
  observer.observe(sentinel.value)
}

async function loadMore() {
  if (loading.value || !hasMore.value) return
  loading.value = true
  error.value = ''
  const requestedPage = page.value + 1
  try {
    const out = await api.creatorFeed({
      page: requestedPage,
      pageSize: PAGE_SIZE,
      feedId: feedId.value || undefined,
    })
    const nextFeedId = String(out.feedId || '')
    const cacheReset = Boolean(feedId.value && nextFeedId && nextFeedId !== feedId.value)
    if (cacheReset || out.cacheReset) {
      creators.value = []
      page.value = 0
    }
    if (nextFeedId) feedId.value = nextFeedId

    const seen = new Set(creators.value.map(creator => creator.uid))
    const additions = (out.data || []).filter(creator => creator?.uid && !seen.has(creator.uid))
    creators.value.push(...additions)
    page.value = Number(out.page || requestedPage)
    total.value = Number(out.total || creators.value.length)
    hasMore.value = Boolean(out.hasMore)
  } catch (loadError) {
    error.value = '创作者加载失败，请检查网络后重试'
    console.warn('[Creators] 创作者信息流加载失败：', loadError)
  } finally {
    loading.value = false
    await nextTick()
    observeSentinel()
    scheduleViewportFill()
  }
}

function activateFeed() {
  if (viewActive) return
  viewActive = true
  nextTick(() => {
    observeSentinel()
    if (!creators.value.length) loadMore()
    else scheduleViewportFill()
  })
}

function deactivateFeed() {
  viewActive = false
  observer?.disconnect()
  observer = null
  if (fillFrame) window.cancelAnimationFrame(fillFrame)
  fillFrame = 0
}

onMounted(() => {
  creatorColumnCount.value = desiredCreatorColumnCount()
  window.addEventListener('resize', scheduleColumnSync, { passive: true })
  activateFeed()
})
onActivated(activateFeed)
onDeactivated(deactivateFeed)
onBeforeUnmount(() => {
  window.removeEventListener('resize', scheduleColumnSync)
  if (resizeFrame) window.cancelAnimationFrame(resizeFrame)
  resizeFrame = 0
  deactivateFeed()
})
</script>

<style scoped>
.creators-page {
  width: min(1360px, calc(100% - 80px));
  min-height: 75vh;
  margin: 0 auto;
  padding: 8px 0 112px;
}

.creators-intro {
  display: flex;
  align-items: end;
  justify-content: space-between;
  gap: 24px;
  margin-bottom: 26px;
  padding: 10px 0 20px;
  border-bottom: 1px solid var(--sos-border-default);
}

.creators-intro__eyebrow {
  display: block;
  margin-bottom: 6px;
  color: var(--sos-text-tertiary);
  font-size: 11px;
  font-weight: 800;
}

.creators-intro h1 {
  margin: 0;
  color: var(--sos-text-primary);
  font-size: 34px;
  font-weight: 900;
  letter-spacing: -0.02em;
}

.creators-intro p {
  margin: 8px 0 0;
  color: var(--sos-text-secondary);
  font-size: 14px;
}

.creators-intro__count {
  flex: 0 0 auto;
  padding-bottom: 3px;
  color: var(--sos-text-tertiary);
  font-size: 13px;
  font-weight: 750;
}

.creator-masonry {
  display: grid;
  grid-template-columns: repeat(var(--creator-column-count, 1), minmax(0, 1fr));
  align-items: start;
  gap: 24px;
}

.creator-column {
  display: grid;
  min-width: 0;
  align-content: start;
  gap: 24px;
}

.creator-card {
  display: block;
  width: 100%;
  margin: 0;
  padding: 18px;
  overflow: hidden;
  background: color-mix(in srgb, var(--sos-bg-surface) 92%, transparent);
  border: 1px solid var(--sos-border-default);
  border-radius: 12px;
  box-shadow: 0 18px 40px -34px rgba(18, 62, 70, 0.72);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
}

.creator-card__header { margin-bottom: 14px; }

.creator-card__identity {
  display: grid;
  width: 100%;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: center;
  gap: 11px;
  padding: 0;
  color: inherit;
  font: inherit;
  text-align: left;
  cursor: pointer;
  background: transparent;
  border: 0;
}

.creator-card__identity > svg {
  color: var(--sos-text-tertiary);
  transition: color var(--sos-duration-fast) ease, transform var(--sos-duration-fast) ease;
}

.creator-card__identity:hover > svg {
  color: var(--sos-accent);
  transform: translate(2px, -2px);
}

.creator-card__identity:focus-visible {
  outline: 3px solid color-mix(in srgb, var(--sos-focus) 42%, transparent);
  outline-offset: 5px;
  border-radius: 8px;
}

.creator-card__avatar {
  display: grid;
  width: 42px;
  height: 42px;
  place-items: center;
  overflow: hidden;
  color: var(--sos-text-secondary);
  background: var(--sos-bg-muted);
  border: 1px solid var(--sos-border-subtle);
  border-radius: 50%;
}

.creator-card__avatar img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.creator-card__name { min-width: 0; }

.creator-card__name strong,
.creator-card__name small {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.creator-card__name strong {
  color: var(--sos-text-primary);
  font-size: 15px;
  font-weight: 850;
}

.creator-card__name small {
  margin-top: 3px;
  color: var(--sos-text-tertiary);
  font-size: 11px;
}

.creator-card__works {
  column-count: 2;
  column-gap: 9px;
}

.creator-card__works--1 { column-count: 1; }

.creator-work {
  position: relative;
  display: block;
  width: 100%;
  aspect-ratio: var(--artwork-ratio, 1.333);
  margin: 0 0 9px;
  padding: 0;
  overflow: hidden;
  break-inside: avoid;
  color: white;
  cursor: pointer;
  background: var(--sos-bg-muted);
  border: 0;
  border-radius: 7px;
}

.creator-work:last-child { margin-bottom: 0; }

.creator-work img {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.32s ease, filter 0.32s ease;
}

.creator-work__label {
  position: absolute;
  inset: auto 0 0;
  padding: 30px 9px 7px;
  text-align: left;
  background: linear-gradient(transparent, rgba(10, 24, 29, 0.78));
}

.creator-work__label strong {
  display: block;
  overflow: hidden;
  font-size: 11px;
  font-weight: 800;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.creator-work:hover img { filter: saturate(1.04); transform: scale(1.025); }

.creator-work:focus-visible {
  outline: 3px solid color-mix(in srgb, var(--sos-focus) 54%, white);
  outline-offset: -3px;
}

.creator-feed-end,
.creators-state {
  display: flex;
  min-height: 72px;
  align-items: center;
  justify-content: center;
  padding: 18px;
  color: var(--sos-text-tertiary);
  font-size: 13px;
  text-align: center;
}

.creators-state {
  min-height: 320px;
  flex-direction: column;
  gap: 12px;
}

.creators-state p { margin: 0; }

.creator-feed-end button,
.creators-state button {
  padding: 8px 13px;
  color: var(--sos-text-secondary);
  font: inherit;
  font-weight: 750;
  cursor: pointer;
  background: var(--sos-bg-surface);
  border: 1px solid var(--sos-border-default);
  border-radius: 999px;
}

.creator-feed-end button:hover,
.creators-state button:hover { color: var(--sos-accent); border-color: var(--sos-accent); }

.creator-feed-end__loading {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.creator-feed-end__loading svg { animation: creator-spin 0.85s linear infinite; }

.creator-card--skeleton { min-height: 360px; }

.creator-skeleton {
  background: linear-gradient(100deg, var(--sos-bg-muted) 20%, var(--sos-bg-surface) 42%, var(--sos-bg-muted) 64%);
  background-size: 220% 100%;
  border-radius: 8px;
  animation: creator-shimmer 1.25s ease-in-out infinite;
}

.creator-skeleton--identity { width: 62%; height: 42px; margin-bottom: 16px; }
.creator-skeleton--work { height: 176px; margin-bottom: 10px; }
.creator-skeleton--short { width: 74%; height: 130px; }

@keyframes creator-spin { to { transform: rotate(360deg); } }
@keyframes creator-shimmer { to { background-position: -120% 0; } }

@media (max-width: 640px) {
  .creators-page { width: calc(100% - 28px); padding-top: 0; }
  .creators-intro { margin-bottom: 16px; padding-bottom: 14px; }
  .creators-intro h1 { font-size: 26px; }
  .creators-intro p { font-size: 13px; }
  .creators-intro__count { font-size: 11px; }
  .creator-masonry,
  .creator-column { gap: 14px; }
  .creator-card { padding: 14px; border-radius: 10px; }
  .creator-card__header { margin-bottom: 12px; }
  .creator-card__avatar { width: 38px; height: 38px; }
}

@media (prefers-reduced-motion: reduce) {
  .creator-card__identity > svg,
  .creator-work img { transition: none; }
  .creator-work:hover img { transform: none; }
  .creator-feed-end__loading svg,
  .creator-skeleton { animation: none; }
}
</style>
