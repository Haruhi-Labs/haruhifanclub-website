<template>
  <button
    v-depth-tilt
    class="artwork-preview"
    type="button"
    :aria-label="`查看作品：${item?.title || '未命名作品'}`"
    :title="item?.title || '未命名作品'"
    data-sfx="click"
    @click="openArtwork"
  >
    <span class="artwork-preview__surface">
      <span
        class="artwork-preview__wash"
        :style="{ backgroundImage: `url(${imageSource})` }"
        aria-hidden="true"
      ></span>
      <img
        class="artwork-preview__image"
        :src="imageSource"
        :alt="item?.title || '画廊作品'"
        :style="{ objectFit: imageFit }"
        loading="lazy"
        decoding="async"
      />
      <ArtworkPopularityBadge :item="item" />
      <span v-if="showLabel" class="artwork-preview__label">
        <strong>{{ item?.title || '未命名作品' }}</strong>
        <small>{{ creatorName }}</small>
      </span>
    </span>
  </button>
</template>

<script setup>
import { computed } from 'vue'
import { thumbUrl } from '../services/api.js'
import { trackArtworkOpen } from '../services/recommendationTracker.js'
import { artworkDepthDirective } from '../utils/artworkDepth.js'
import ArtworkPopularityBadge from './ArtworkPopularityBadge.vue'

const vDepthTilt = artworkDepthDirective

const props = defineProps({
  item: { type: Object, required: true },
  showLabel: { type: Boolean, default: false },
  imageFit: { type: String, default: 'contain' },
  source: { type: String, default: 'gallery' },
  position: { type: Number, default: 0 },
})

const emit = defineEmits(['open'])

const imageSource = computed(() => thumbUrl(
  props.item?.image_url || props.item?.imageUrl || props.item?.url || '',
  640,
))

const creatorName = computed(() => (
  props.item?.uploader_display_name
  || props.item?.uploader_name
  || props.item?.uploader_uid
  || '画廊收藏'
))

function openArtwork() {
  trackArtworkOpen(props.item, { source: props.source, position: props.position })
  emit('open', props.item)
}
</script>

<style scoped>
.artwork-preview {
  --depth-size: clamp(6px, 0.7vw, 10px);
  --depth-rx: 0deg;
  --depth-ry: 0deg;
  --depth-lift: 0px;
  --depth-scale: 1;
  position: relative;
  display: block;
  width: 100%;
  min-width: 0;
  min-height: 0;
  padding: 0;
  overflow: visible;
  color: white;
  cursor: pointer;
  background: transparent;
  border: 0;
  border-radius: 6px;
  container-type: inline-size;
  transform:
    translate3d(0, var(--depth-lift), 0)
    rotateX(var(--depth-rx))
    rotateY(var(--depth-ry))
    scale(var(--depth-scale));
  transform-style: preserve-3d;
  will-change: transform;
  transition:
    filter var(--sos-duration-base) ease;
}

.artwork-preview::before,
.artwork-preview::after {
  content: '';
  position: absolute;
  pointer-events: none;
  background: linear-gradient(135deg, rgba(31, 55, 61, 0.96), rgba(10, 24, 29, 0.92));
  border: 1px solid color-mix(in srgb, var(--sos-accent) 22%, rgba(5, 16, 19, 0.88));
  backface-visibility: hidden;
}

.artwork-preview::before {
  top: 0;
  left: 100%;
  width: var(--depth-size);
  height: 100%;
  transform: rotateY(90deg);
  transform-origin: 0 50%;
}

.artwork-preview::after {
  top: 100%;
  left: 0;
  width: 100%;
  height: var(--depth-size);
  transform: rotateX(-90deg);
  transform-origin: 50% 0;
}

.artwork-preview__surface {
  position: absolute;
  inset: 0;
  z-index: 1;
  display: block;
  overflow: hidden;
  background: color-mix(in srgb, var(--sos-bg-muted) 70%, white);
  border: 1px solid color-mix(in srgb, var(--sos-accent) 28%, var(--sos-border-subtle));
  border-radius: inherit;
  box-shadow: 0 14px 30px -24px rgba(20, 55, 64, 0.72);
  transform: translateZ(0.1px);
  backface-visibility: hidden;
  transition: border-color var(--sos-duration-base) ease, box-shadow var(--sos-duration-base) ease;
}

.artwork-preview__surface::after {
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
  transition: opacity 0.2s ease;
}

.artwork-preview:hover {
  z-index: 2;
}

.artwork-preview:hover .artwork-preview__surface {
  border-color: color-mix(in srgb, var(--sos-accent) 66%, white);
  box-shadow: 0 26px 44px -24px rgba(20, 55, 64, 0.86);
}

.artwork-preview:focus-visible {
  z-index: 3;
  outline: 3px solid color-mix(in srgb, var(--sos-focus) 44%, transparent);
  outline-offset: 3px;
}

.artwork-preview__wash {
  position: absolute;
  inset: -12%;
  z-index: 0;
  background-position: center;
  background-size: cover;
  filter: blur(22px) saturate(0.82);
  opacity: 0.46;
  transform: scale(1.08);
}

.artwork-preview__image {
  position: relative;
  z-index: 1;
  display: block;
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.artwork-preview__label {
  position: absolute;
  inset: auto 0 0;
  z-index: 2;
  display: flex;
  min-width: 0;
  align-items: flex-end;
  justify-content: space-between;
  gap: 10px;
  padding: 40px 11px 9px;
  text-align: left;
  background: linear-gradient(transparent, rgba(14, 23, 31, 0.78));
}

.artwork-preview__label strong,
.artwork-preview__label small {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.artwork-preview__label strong {
  font-size: 14px;
  font-weight: 850;
}

.artwork-preview__label small {
  flex: 0 1 auto;
  color: rgba(255, 255, 255, 0.78);
  font-size: 10px;
}

@container (max-width: 230px) {
  .artwork-preview__label {
    display: flex;
    align-items: stretch;
    flex-direction: column;
    gap: 1px;
    padding: 30px 7px 6px;
  }

  .artwork-preview__label strong { font-size: 11px; }
  .artwork-preview__label small { font-size: 9px; }
}

@media (max-width: 768px) {
  .artwork-preview__label {
    padding-bottom: 7px;
  }
}

@media (prefers-reduced-motion: reduce) {
  .artwork-preview {
    transform: none;
    transition: none;
  }

  .artwork-preview::before,
  .artwork-preview::after { display: none; }
}

@media (hover: none), (pointer: coarse) {
  .artwork-preview {
    transform: none;
  }

  .artwork-preview::before,
  .artwork-preview::after { display: none; }
}
</style>
