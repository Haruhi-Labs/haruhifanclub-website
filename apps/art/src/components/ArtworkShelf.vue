<template>
  <div ref="shelfRoot" class="artwork-shelf" :aria-busy="loading">
    <div v-if="loading && !items.length" class="artwork-shelf__skeletons">
      <div v-for="index in 8" :key="`skeleton-${index}`" class="artwork-tile skeleton"></div>
    </div>

    <JustifiedArtworkGrid
      v-else-if="items.length"
      :items="items"
      :target-row-height="270"
      :mobile-target-row-height="145"
      :gap="18"
      randomize-row-heights
    >
      <template #default="{ item, index: position }">
        <button
          class="artwork-tile"
          type="button"
          :aria-label="`查看作品：${item.title || '未命名作品'}`"
          :title="item.title || '未命名作品'"
          :data-artwork-index="position"
          data-sfx="click"
          @click="openArtwork(item, position)"
        >
          <span
            class="artwork-tile__blur"
            :style="{ backgroundImage: `url(${imageSource(item)})` }"
            aria-hidden="true"
          ></span>
          <img
            class="artwork-tile__image"
            :src="imageSource(item)"
            :alt="item.title || '画廊作品'"
            loading="lazy"
            decoding="async"
          />
          <ArtworkPopularityBadge :item="item" />
          <span class="artwork-tile__label">
            <strong>{{ item.title || '未命名作品' }}</strong>
            <small>{{ creatorName(item) }}</small>
          </span>
        </button>
      </template>
    </JustifiedArtworkGrid>

    <div v-else-if="!loading" class="artwork-shelf__empty">暂无作品</div>
  </div>
</template>

<script setup>
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { thumbUrl } from '../services/api.js'
import { trackArtworkImpression, trackArtworkOpen } from '../services/recommendationTracker.js'
import ArtworkPopularityBadge from './ArtworkPopularityBadge.vue'
import JustifiedArtworkGrid from './JustifiedArtworkGrid.vue'

const props = defineProps({
  items: { type: Array, default: () => [] },
  loading: { type: Boolean, default: false },
  trackingSource: { type: String, default: 'gallery' },
  batchId: { type: String, default: '' },
})

const emit = defineEmits(['open'])
const shelfRoot = ref(null)
const impressionKeys = new Set()
const impressionTimers = new Map()
let observer = null

function trackingContext(position) {
  return {
    batchId: props.batchId || undefined,
    source: props.trackingSource,
    position,
  }
}

function impressionKey(item) {
  return `${props.batchId || 'none'}:${props.trackingSource}:${item?.id}`
}

function clearImpressionTimer(element) {
  const timer = impressionTimers.get(element)
  if (timer) window.clearTimeout(timer)
  impressionTimers.delete(element)
}

function observeTiles() {
  observer?.disconnect()
  for (const element of impressionTimers.keys()) clearImpressionTimer(element)
  if (!shelfRoot.value || !props.items.length) return

  const tiles = shelfRoot.value.querySelectorAll('[data-artwork-index]')
  if (typeof IntersectionObserver === 'undefined') {
    props.items.forEach((item, position) => {
      const key = impressionKey(item)
      if (impressionKeys.has(key)) return
      impressionKeys.add(key)
      trackArtworkImpression(item, trackingContext(position))
    })
    return
  }

  observer = new IntersectionObserver((entries) => {
    for (const entry of entries) {
      const element = entry.target
      const position = Number(element.dataset.artworkIndex)
      const item = props.items[position]
      if (!item) continue
      const key = impressionKey(item)
      if (entry.isIntersecting && entry.intersectionRatio >= 0.6 && !impressionKeys.has(key)) {
        if (impressionTimers.has(element)) continue
        const timer = window.setTimeout(() => {
          impressionTimers.delete(element)
          impressionKeys.add(key)
          trackArtworkImpression(item, trackingContext(position))
          observer?.unobserve(element)
        }, 800)
        impressionTimers.set(element, timer)
      } else if (!entry.isIntersecting || entry.intersectionRatio < 0.6) {
        clearImpressionTimer(element)
      }
    }
  }, { threshold: [0, 0.6] })
  tiles.forEach(element => observer.observe(element))
}

function openArtwork(item, position) {
  trackArtworkOpen(item, trackingContext(position))
  emit('open', item)
}

function imageSource(item) {
  return thumbUrl(item?.image_url || item?.imageUrl || item?.url || '', 640)
}

function creatorName(item) {
  return item?.uploader_display_name
    || item?.uploader_name
    || item?.uploader_uid
    || '画廊收藏'
}

watch(
  [() => props.items, () => props.batchId, () => props.trackingSource],
  () => nextTick(observeTiles),
)

onMounted(() => nextTick(observeTiles))
onBeforeUnmount(() => {
  observer?.disconnect()
  for (const element of impressionTimers.keys()) clearImpressionTimer(element)
})
</script>

<style scoped>
.artwork-shelf {
  width: 100%;
}

.artwork-shelf__skeletons {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 18px;
}

.artwork-tile {
  position: relative;
  display: block;
  width: 100%;
  padding: 0;
  overflow: hidden;
  cursor: pointer;
  background: rgba(255, 255, 255, 0.56);
  border: 1px solid rgba(95, 215, 226, 0.72);
  border-radius: 8px;
  box-shadow: 0 12px 28px -18px rgba(22, 66, 76, 0.7);
  isolation: isolate;
  container-type: inline-size;
  transition:
    transform 0.22s ease,
    border-color 0.22s ease,
    box-shadow 0.22s ease;
}

.artwork-tile:hover {
  z-index: 2;
  border-color: color-mix(in srgb, var(--sos-accent) 72%, white);
  box-shadow: 0 18px 32px -18px rgba(22, 66, 76, 0.82);
  transform: translateY(-4px);
}

.artwork-tile:focus-visible {
  outline: 3px solid color-mix(in srgb, var(--sos-accent) 45%, transparent);
  outline-offset: 3px;
}

.artwork-tile__blur {
  position: absolute;
  inset: -10%;
  z-index: 0;
  background-position: center;
  background-size: cover;
  filter: blur(18px) saturate(0.88);
  opacity: 0.48;
  transform: scale(1.08);
}

.artwork-tile__image {
  position: relative;
  z-index: 1;
  display: block;
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.artwork-tile__label {
  position: absolute;
  inset: auto 0 0;
  z-index: 2;
  display: flex;
  min-width: 0;
  align-items: flex-end;
  justify-content: space-between;
  gap: 10px;
  padding: 40px 11px 9px;
  color: white;
  text-align: left;
  background: linear-gradient(transparent, rgba(14, 23, 31, 0.78));
}

.artwork-tile__label strong,
.artwork-tile__label small {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.artwork-tile__label strong { font-size: 14px; font-weight: 850; }
.artwork-tile__label small {
  flex: 0 1 auto;
  color: rgba(255, 255, 255, 0.78);
  font-size: 10px;
}

@container (max-width: 230px) {
  .artwork-tile__label {
    align-items: stretch;
    flex-direction: column;
    gap: 1px;
    padding: 30px 7px 6px;
  }

  .artwork-tile__label strong { font-size: 11px; }
  .artwork-tile__label small { font-size: 9px; }
}

.artwork-tile.skeleton {
  aspect-ratio: 4 / 3;
  pointer-events: none;
  border-color: rgba(95, 215, 226, 0.28);
  background: rgba(255, 255, 255, 0.5);
  animation: skeleton-pulse 1.2s ease-in-out infinite alternate;
}

.artwork-shelf__empty {
  grid-column: 1 / -1;
  min-height: 120px;
  display: grid;
  place-items: center;
  color: var(--sos-text-tertiary);
  font-size: 14px;
  border: 1px dashed var(--sos-border-subtle);
  border-radius: 8px;
}

@keyframes skeleton-pulse {
  from { opacity: 0.5; }
  to { opacity: 0.9; }
}

@media (max-width: 768px) {
  .artwork-shelf__skeletons {
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
  }

  .artwork-tile:hover { transform: none; }
}

@media (prefers-reduced-motion: reduce) {
  .artwork-tile { transition: none; }
  .artwork-tile.skeleton { animation: none; }
}
</style>
