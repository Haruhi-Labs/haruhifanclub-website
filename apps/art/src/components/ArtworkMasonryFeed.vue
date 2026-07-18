<template>
  <div
    ref="feedRoot"
    class="artwork-feed"
    role="feed"
    :aria-busy="loadingInitial"
    :style="{ '--artwork-column-count': columnCount }"
  >
    <div v-if="loadingInitial && !items.length" class="artwork-feed__skeletons" aria-hidden="true">
      <span
        v-for="index in skeletonCount"
        :key="index"
        class="artwork-feed__skeleton"
        :style="{ aspectRatio: index % 3 === 0 ? '3 / 4' : index % 2 === 0 ? '1 / 1' : '4 / 3' }"
      ></span>
    </div>

    <div v-else-if="items.length" class="artwork-feed__columns">
      <div
        v-for="(column, columnIndex) in masonryColumns"
        :key="columnIndex"
        class="artwork-feed__column"
        role="presentation"
      >
        <article
          v-for="entry in column"
          :key="entry.item.id"
          class="artwork-feed__entry"
          :aria-posinset="entry.position + 1"
          :aria-setsize="total || undefined"
        >
          <button
            class="artwork-feed__card"
            type="button"
            :aria-label="`查看作品：${entry.item.title || '未命名作品'}`"
            :title="entry.item.title || '未命名作品'"
            :data-artwork-id="entry.item.id"
            :data-artwork-position="entry.position"
            data-feed-artwork
            data-sfx="click"
            @click="openArtwork(entry.item, entry.position)"
          >
            <span class="artwork-feed__media" :style="mediaStyle(entry.item)">
              <img
                :src="imageUrl(entry.item, 640)"
                :srcset="imageSrcset(entry.item)"
                :alt="entry.item.title || '画廊作品'"
                sizes="(max-width: 719px) calc(50vw - 20px), (max-width: 999px) 33vw, (max-width: 1319px) 25vw, 20vw"
                loading="lazy"
                decoding="async"
              />
              <ArtworkPopularityBadge :item="entry.item" />
              <span class="artwork-feed__caption">
                <strong>{{ entry.item.title || '未命名作品' }}</strong>
                <small>{{ creatorName(entry.item) }}</small>
              </span>
            </span>
          </button>
        </article>
      </div>
    </div>

    <div v-else-if="!loadingInitial" class="artwork-feed__empty">
      当前筛选条件下还没有作品
    </div>
  </div>
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
import { thumbUrl } from '../services/api.js'
import { trackArtworkImpression, trackArtworkOpen } from '../services/recommendationTracker.js'
import ArtworkPopularityBadge from './ArtworkPopularityBadge.vue'

const props = defineProps({
  items: { type: Array, default: () => [] },
  total: { type: Number, default: 0 },
  loadingInitial: { type: Boolean, default: false },
  trackingSource: { type: String, default: 'gallery-recommended' },
})

const emit = defineEmits(['open'])
const feedRoot = ref(null)
const columnCount = ref(2)
const impressionKeys = new Set()
const impressionTimers = new Map()
let resizeObserver = null
let impressionObserver = null

const skeletonCount = computed(() => columnCount.value * 2)

function numericRatio(item) {
  const width = Number(item?.image_width || item?.width || item?.images?.[0]?.width)
  const height = Number(item?.image_height || item?.height || item?.images?.[0]?.height)
  return width > 0 && height > 0 ? width / height : 4 / 3
}

function desiredColumnCount(width) {
  if (width >= 1320) return 5
  if (width >= 1000) return 4
  if (width >= 720) return 3
  return 2
}

const masonryColumns = computed(() => {
  const columns = Array.from({ length: columnCount.value }, () => [])
  const heights = Array(columnCount.value).fill(0)
  props.items.forEach((item, position) => {
    let target = 0
    for (let index = 1; index < heights.length; index += 1) {
      if (heights[index] < heights[target]) target = index
    }
    columns[target].push({ item, position })
    heights[target] += (1 / numericRatio(item)) + 0.06
  })
  return columns
})

function mediaStyle(item) {
  const ratio = numericRatio(item)
  return { aspectRatio: String(ratio) }
}

function rawImageUrl(item) {
  return item?.image_url || item?.imageUrl || item?.url || ''
}

function imageUrl(item, width) {
  return thumbUrl(rawImageUrl(item), width)
}

function imageSrcset(item) {
  return `${imageUrl(item, 320)} 320w, ${imageUrl(item, 640)} 640w`
}

function creatorName(item) {
  return item?.uploader_display_name
    || item?.uploader_name
    || item?.uploader_uid
    || '画廊收藏'
}

function trackingContext(item, position) {
  return {
    batchId: item?.recommendation?.batch_id || undefined,
    source: props.trackingSource,
    position,
  }
}

function impressionKey(item) {
  const batchId = item?.recommendation?.batch_id || 'none'
  return `${batchId}:${props.trackingSource}:${item?.id}`
}

function clearImpressionTimer(element) {
  const timer = impressionTimers.get(element)
  if (timer) window.clearTimeout(timer)
  impressionTimers.delete(element)
}

function ensureImpressionObserver() {
  if (impressionObserver || typeof IntersectionObserver === 'undefined') return
  impressionObserver = new IntersectionObserver((entries) => {
    for (const entry of entries) {
      const element = entry.target
      const position = Number(element.dataset.artworkPosition)
      const item = props.items[position]
      if (!item) continue
      const key = impressionKey(item)
      if (entry.isIntersecting && entry.intersectionRatio >= 0.55 && !impressionKeys.has(key)) {
        if (impressionTimers.has(element)) continue
        const timer = window.setTimeout(() => {
          impressionTimers.delete(element)
          impressionKeys.add(key)
          trackArtworkImpression(item, trackingContext(item, position))
          impressionObserver?.unobserve(element)
        }, 650)
        impressionTimers.set(element, timer)
      } else if (!entry.isIntersecting || entry.intersectionRatio < 0.55) {
        clearImpressionTimer(element)
      }
    }
  }, { threshold: [0, 0.55] })
}

function observeNewCards({ reset = false } = {}) {
  if (!feedRoot.value || !props.items.length) return
  if (typeof IntersectionObserver === 'undefined') {
    props.items.forEach((item, position) => {
      const key = impressionKey(item)
      if (impressionKeys.has(key)) return
      impressionKeys.add(key)
      trackArtworkImpression(item, trackingContext(item, position))
    })
    return
  }
  ensureImpressionObserver()
  if (reset) {
    feedRoot.value.querySelectorAll('[data-feed-observed]').forEach((element) => {
      element.removeAttribute('data-feed-observed')
    })
  }
  feedRoot.value.querySelectorAll('[data-feed-artwork]:not([data-feed-observed])').forEach((element) => {
    element.dataset.feedObserved = '1'
    impressionObserver?.observe(element)
  })
}

function pauseImpressionTracking() {
  impressionObserver?.disconnect()
  impressionObserver = null
  for (const element of impressionTimers.keys()) clearImpressionTimer(element)
  feedRoot.value?.querySelectorAll('[data-feed-observed]').forEach((element) => {
    element.removeAttribute('data-feed-observed')
  })
}

function openArtwork(item, position) {
  trackArtworkOpen(item, trackingContext(item, position))
  emit('open', item)
}

watch(
  [() => props.items, () => props.trackingSource],
  () => nextTick(() => observeNewCards()),
)

onMounted(() => {
  resizeObserver = new ResizeObserver((entries) => {
    const width = entries[0]?.contentRect?.width || feedRoot.value?.clientWidth || 0
    columnCount.value = desiredColumnCount(width)
  })
  resizeObserver.observe(feedRoot.value)
  nextTick(() => observeNewCards())
})

onActivated(() => nextTick(() => observeNewCards({ reset: true })))
onDeactivated(pauseImpressionTracking)
onBeforeUnmount(() => {
  resizeObserver?.disconnect()
  pauseImpressionTracking()
})
</script>

<style scoped>
.artwork-feed {
  width: 100%;
}

.artwork-feed__columns,
.artwork-feed__skeletons {
  display: grid;
  grid-template-columns: repeat(var(--artwork-column-count), minmax(0, 1fr));
  gap: 18px;
  align-items: start;
}

.artwork-feed__column {
  display: grid;
  min-width: 0;
  gap: 18px;
}

.artwork-feed__entry {
  min-width: 0;
  content-visibility: auto;
  contain: layout paint style;
  contain-intrinsic-size: auto 280px;
}

.artwork-feed__card {
  display: block;
  width: 100%;
  padding: 0;
  overflow: hidden;
  color: white;
  text-align: left;
  cursor: pointer;
  background: color-mix(in srgb, var(--sos-bg-muted) 82%, white);
  border: 1px solid color-mix(in srgb, var(--sos-accent) 28%, var(--sos-border-subtle));
  border-radius: 9px;
  box-shadow: 0 14px 30px -26px rgba(20, 55, 64, 0.76);
  transition:
    transform 150ms cubic-bezier(0.23, 1, 0.32, 1),
    border-color 180ms ease,
    box-shadow 180ms ease;
}

.artwork-feed__card:active { transform: scale(0.985); }

.artwork-feed__card:focus-visible {
  outline: 3px solid color-mix(in srgb, var(--sos-focus) 45%, transparent);
  outline-offset: 3px;
}

.artwork-feed__media {
  position: relative;
  display: block;
  width: 100%;
  overflow: hidden;
}

.artwork-feed__media img {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: contain;
  background: color-mix(in srgb, var(--sos-bg-muted) 88%, white);
}

.artwork-feed__caption {
  position: absolute;
  inset: auto 0 0;
  display: grid;
  gap: 2px;
  min-width: 0;
  padding: 44px 11px 10px;
  background: linear-gradient(transparent, rgba(10, 20, 27, 0.82));
}

.artwork-feed__caption strong,
.artwork-feed__caption small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.artwork-feed__caption strong { font-size: 13px; font-weight: 850; }
.artwork-feed__caption small { color: rgba(255, 255, 255, 0.75); font-size: 10px; }

.artwork-feed__skeleton {
  display: block;
  background: color-mix(in srgb, var(--sos-bg-surface) 78%, var(--sos-accent) 6%);
  border: 1px solid color-mix(in srgb, var(--sos-border-subtle) 74%, transparent);
  border-radius: 9px;
  animation: artwork-skeleton 900ms ease-in-out infinite alternate;
}

.artwork-feed__empty {
  min-height: 180px;
  display: grid;
  place-items: center;
  color: var(--sos-text-tertiary);
  font-size: 14px;
  border: 1px dashed var(--sos-border-subtle);
  border-radius: 9px;
}

@keyframes artwork-skeleton {
  from { opacity: 0.48; }
  to { opacity: 0.82; }
}

@media (hover: hover) and (pointer: fine) {
  .artwork-feed__card:hover {
    border-color: color-mix(in srgb, var(--sos-accent) 68%, white);
    box-shadow: 0 24px 42px -27px rgba(20, 55, 64, 0.92);
  }
}

@media (max-width: 719px) {
  .artwork-feed__columns,
  .artwork-feed__skeletons { gap: 9px; }
  .artwork-feed__column { gap: 9px; }
  .artwork-feed__entry { contain-intrinsic-size: auto 190px; }
  .artwork-feed__card { border-radius: 7px; }
  .artwork-feed__caption { padding: 34px 7px 7px; }
  .artwork-feed__caption strong { font-size: 11px; }
  .artwork-feed__caption small { font-size: 9px; }
}

@media (prefers-reduced-motion: reduce) {
  .artwork-feed__card { transition: border-color 180ms ease, box-shadow 180ms ease; }
  .artwork-feed__card:active { transform: none; }
  .artwork-feed__skeleton { animation: none; opacity: 0.68; }
}
</style>
