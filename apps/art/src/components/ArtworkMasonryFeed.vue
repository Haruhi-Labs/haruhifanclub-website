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
          <div v-depth-tilt class="artwork-feed__card">
            <button
              class="artwork-feed__open"
              type="button"
              :aria-label="`查看作品：${entry.item.title || '未命名作品'}${isLongArtwork(entry.item) ? '，长图' : ''}`"
              :title="entry.item.title || '未命名作品'"
              :data-artwork-id="entry.item.id"
              :data-artwork-position="entry.position"
              data-feed-artwork
              data-sfx="click"
              @click="openArtwork(entry.item, entry.position)"
            >
              <span class="artwork-feed__surface">
                <span
                  class="artwork-feed__media"
                  :class="{ 'is-long': isLongArtwork(entry.item) }"
                  :style="mediaStyle(entry.item)"
                >
                  <img
                    :src="imageUrl(entry.item, 640)"
                    :srcset="imageSrcset(entry.item, isLongArtwork(entry.item))"
                    :alt="entry.item.title || '画廊作品'"
                    sizes="(max-width: 719px) calc(50vw - 20px), (max-width: 999px) 33vw, (max-width: 1319px) 25vw, 20vw"
                    :loading="entry.position < 4 ? 'eager' : 'lazy'"
                    :fetchpriority="entry.position < 2 ? 'high' : 'auto'"
                    decoding="async"
                  />
                  <ArtworkPopularityBadge :item="entry.item" />
                  <span v-if="isLongArtwork(entry.item)" class="artwork-feed__long-badge" aria-hidden="true">
                    长图
                  </span>
                  <span class="artwork-feed__caption">
                    <strong>{{ entry.item.title || '未命名作品' }}</strong>
                    <small>{{ creatorName(entry.item) }}</small>
                  </span>
                </span>
              </span>
            </button>
          </div>

          <button
            class="artwork-feed__like"
            :class="{ 'is-liked': isLiked(entry.item) }"
            type="button"
            :disabled="isLikePending(entry.item)"
            :aria-pressed="isLiked(entry.item)"
            :aria-label="isLiked(entry.item) ? `已点赞：${entry.item.title || '未命名作品'}` : `点赞：${entry.item.title || '未命名作品'}`"
            :title="isLiked(entry.item) ? '已点赞' : '点赞'"
            @pointerdown.stop
            @click.stop="likeArtwork(entry.item)"
          >
            <Heart
              :size="16"
              :stroke-width="2.2"
              :fill="isLiked(entry.item) ? 'currentColor' : 'none'"
              aria-hidden="true"
            />
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
import { Heart } from 'lucide-vue-next'
import { thumbUrl } from '../services/api.js'
import { trackArtworkImpression, trackArtworkOpen } from '../services/recommendationTracker.js'
import { useGalleryStore } from '../stores/galleryStore.js'
import { artworkDepthDirective } from '../utils/artworkDepth.js'
import ArtworkPopularityBadge from './ArtworkPopularityBadge.vue'

const vDepthTilt = artworkDepthDirective
const LONG_ARTWORK_RATIO_THRESHOLD = 0.45
const LONG_ARTWORK_PREVIEW_RATIO = 3 / 4

const props = defineProps({
  items: { type: Array, default: () => [] },
  total: { type: Number, default: 0 },
  loadingInitial: { type: Boolean, default: false },
  trackingSource: { type: String, default: 'gallery-recommended' },
})

const emit = defineEmits(['open'])
const galleryStore = useGalleryStore()
const feedRoot = ref(null)
const columnCount = ref(2)
const likedArtworkIds = ref(new Set())
const pendingLikeIds = ref(new Set())
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

function isLongArtwork(item) {
  return numericRatio(item) < LONG_ARTWORK_RATIO_THRESHOLD
}

function displayRatio(item) {
  return isLongArtwork(item) ? LONG_ARTWORK_PREVIEW_RATIO : numericRatio(item)
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
    heights[target] += (1 / displayRatio(item)) + 0.06
  })
  return columns
})

function mediaStyle(item) {
  const ratio = displayRatio(item)
  return { aspectRatio: String(ratio) }
}

function rawImageUrl(item) {
  return item?.image_url || item?.imageUrl || item?.url || ''
}

function imageUrl(item, width) {
  return thumbUrl(rawImageUrl(item), width)
}

function imageSrcset(item, includeLarge = false) {
  const sources = [`${imageUrl(item, 320)} 320w`, `${imageUrl(item, 640)} 640w`]
  if (includeLarge) {
    sources.push(`${imageUrl(item, 960)} 960w`, `${imageUrl(item, 1920)} 1920w`)
  }
  return sources.join(', ')
}

function creatorName(item) {
  return item?.uploader_display_name
    || item?.uploader_name
    || item?.uploader_uid
    || '画廊收藏'
}

function artworkKey(item) {
  return String(item?.id ?? '')
}

function isLiked(item) {
  return Boolean(item?.liked) || likedArtworkIds.value.has(artworkKey(item))
}

function isLikePending(item) {
  return pendingLikeIds.value.has(artworkKey(item))
}

function updateIdSet(target, key, shouldInclude) {
  const next = new Set(target.value)
  if (shouldInclude) next.add(key)
  else next.delete(key)
  target.value = next
}

async function likeArtwork(item) {
  const key = artworkKey(item)
  if (!key || isLiked(item) || isLikePending(item)) return

  updateIdSet(pendingLikeIds, key, true)
  const succeeded = await galleryStore.likeArtwork(item)
  if (succeeded) {
    item.liked = true
    updateIdSet(likedArtworkIds, key, true)
  }
  updateIdSet(pendingLikeIds, key, false)
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
  position: relative;
  min-width: 0;
  contain: layout style;
  perspective: clamp(720px, 80vw, 1180px);
}

.artwork-feed__card {
  --depth-size: clamp(6px, 0.7vw, 10px);
  --depth-rx: 0deg;
  --depth-ry: 0deg;
  --depth-lift: 0px;
  --depth-scale: 1;
  position: relative;
  width: 100%;
  overflow: visible;
  border-radius: 9px;
  transform:
    translate3d(0, var(--depth-lift), 0)
    rotateX(var(--depth-rx))
    rotateY(var(--depth-ry))
    scale(var(--depth-scale));
  transform-style: preserve-3d;
  transition: filter var(--sos-duration-base) ease;
}

.artwork-feed__card.is-depth-active {
  z-index: 3;
  will-change: transform;
}

.artwork-feed__card::before,
.artwork-feed__card::after {
  content: '';
  position: absolute;
  pointer-events: none;
  background: linear-gradient(135deg, rgba(31, 55, 61, 0.96), rgba(10, 24, 29, 0.92));
  border: 1px solid color-mix(in srgb, var(--sos-accent) 22%, rgba(5, 16, 19, 0.88));
  backface-visibility: hidden;
}

.artwork-feed__card::before {
  top: 0;
  left: 100%;
  width: var(--depth-size);
  height: 100%;
  transform: rotateY(90deg);
  transform-origin: 0 50%;
}

.artwork-feed__card::after {
  top: 100%;
  left: 0;
  width: 100%;
  height: var(--depth-size);
  transform: rotateX(-90deg);
  transform-origin: 50% 0;
}

.artwork-feed__open {
  display: block;
  width: 100%;
  padding: 0;
  color: white;
  text-align: left;
  cursor: pointer;
  background: transparent;
  border: 0;
  border-radius: inherit;
}

.artwork-feed__surface {
  position: relative;
  z-index: 1;
  display: block;
  overflow: hidden;
  background: color-mix(in srgb, var(--sos-bg-muted) 82%, white);
  border: 1px solid color-mix(in srgb, var(--sos-accent) 28%, var(--sos-border-subtle));
  border-radius: inherit;
  box-shadow: 0 14px 30px -24px rgba(20, 55, 64, 0.72);
  transform: translateZ(0.1px);
  backface-visibility: hidden;
  transition: border-color var(--sos-duration-base) ease, box-shadow var(--sos-duration-base) ease;
}

.artwork-feed__surface::after {
  content: '';
  position: absolute;
  inset: -1px;
  z-index: 3;
  pointer-events: none;
  background: radial-gradient(
    circle at var(--depth-glare-x, 50%) var(--depth-glare-y, 50%),
    rgba(255, 255, 255, 0.86),
    rgba(159, 235, 240, 0.24) 20%,
    transparent 48%
  );
  mix-blend-mode: soft-light;
  opacity: var(--depth-glare-opacity, 0);
  transition: opacity 200ms ease;
}

.artwork-feed__card.is-depth-active .artwork-feed__surface {
  border-color: color-mix(in srgb, var(--sos-accent) 66%, white);
  box-shadow: 0 26px 44px -24px rgba(20, 55, 64, 0.86);
}

.artwork-feed__open:focus-visible {
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

.artwork-feed__media.is-long img {
  object-fit: cover;
  object-position: 50% 50%;
}

.artwork-feed__long-badge {
  position: absolute;
  top: 7px;
  left: 50%;
  z-index: 4;
  display: inline-flex;
  height: 23px;
  align-items: center;
  padding: 0 8px;
  color: color-mix(in srgb, var(--sos-text-primary) 78%, var(--sos-accent));
  font-size: 10px;
  font-weight: 850;
  line-height: 1;
  pointer-events: none;
  background: color-mix(in srgb, var(--sos-bg-surface) 88%, transparent);
  border: 1px solid color-mix(in srgb, var(--sos-accent) 26%, rgba(255, 255, 255, 0.8));
  border-radius: 999px;
  box-shadow: 0 3px 10px rgba(31, 61, 68, 0.13);
  backdrop-filter: blur(7px) saturate(0.9);
  transform: translateX(-50%);
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

.artwork-feed__like {
  position: absolute;
  top: 7px;
  left: 7px;
  z-index: 6;
  display: inline-grid;
  width: 30px;
  height: 30px;
  padding: 0;
  place-items: center;
  color: color-mix(in srgb, #e54d68 88%, var(--sos-accent));
  cursor: pointer;
  background: color-mix(in srgb, var(--sos-bg-surface) 88%, transparent);
  border: 1px solid color-mix(in srgb, var(--sos-accent) 24%, rgba(255, 255, 255, 0.82));
  border-radius: 999px;
  box-shadow: 0 4px 14px rgba(31, 61, 68, 0.18);
  opacity: 0;
  pointer-events: none;
  backdrop-filter: blur(8px) saturate(0.92);
  transform: translateY(-3px) scale(0.96);
  transition:
    opacity 140ms ease,
    color 140ms ease,
    background-color 140ms ease,
    transform 180ms cubic-bezier(0.2, 0.8, 0.2, 1);
}

.artwork-feed__like:hover {
  color: #d93859;
  background: color-mix(in srgb, var(--sos-bg-surface) 96%, white);
}

.artwork-feed__like:active {
  transform: translateY(0) scale(0.9);
  transition-duration: 90ms;
}

.artwork-feed__like:focus-visible {
  outline: 3px solid color-mix(in srgb, var(--sos-focus) 45%, transparent);
  outline-offset: 2px;
}

.artwork-feed__like:disabled {
  cursor: wait;
  opacity: 0.68;
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
  .artwork-feed__entry:hover .artwork-feed__like,
  .artwork-feed__entry:focus-within .artwork-feed__like,
  .artwork-feed__like.is-liked {
    opacity: 1;
    pointer-events: auto;
    transform: translateY(0) scale(1);
  }
}

@media (max-width: 719px) {
  .artwork-feed__columns,
  .artwork-feed__skeletons { gap: 9px; }
  .artwork-feed__column { gap: 9px; }
  .artwork-feed__card { border-radius: 7px; }
  .artwork-feed__caption { padding: 34px 7px 7px; }
  .artwork-feed__caption strong { font-size: 11px; }
  .artwork-feed__caption small { font-size: 9px; }
  .artwork-feed__long-badge {
    top: 5px;
    height: 21px;
    padding-inline: 6px;
    font-size: 9px;
  }
  .artwork-feed__like {
    top: 5px;
    left: 5px;
    width: 28px;
    height: 28px;
  }
}

@media (prefers-reduced-motion: reduce) {
  .artwork-feed__card { transform: none; transition: none; }
  .artwork-feed__card::before,
  .artwork-feed__card::after { display: none; }
  .artwork-feed__like { transition: none; }
  .artwork-feed__skeleton { animation: none; opacity: 0.68; }
}

@media (hover: none), (pointer: coarse) {
  .artwork-feed__card { transform: none; }
  .artwork-feed__card::before,
  .artwork-feed__card::after { display: none; }
  .artwork-feed__like {
    opacity: 1;
    pointer-events: auto;
    transform: translateY(0) scale(1);
  }
}
</style>
