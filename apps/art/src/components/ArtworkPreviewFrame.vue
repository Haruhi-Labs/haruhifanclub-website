<template>
  <button
    class="artwork-preview"
    type="button"
    :aria-label="`查看作品：${item?.title || '未命名作品'}`"
    :title="item?.title || '未命名作品'"
    data-sfx="click"
    @click="openArtwork"
  >
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
  </button>
</template>

<script setup>
import { computed } from 'vue'
import { thumbUrl } from '../services/api.js'
import { trackArtworkOpen } from '../services/recommendationTracker.js'
import ArtworkPopularityBadge from './ArtworkPopularityBadge.vue'

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
  position: relative;
  display: block;
  width: 100%;
  min-width: 0;
  min-height: 0;
  padding: 0;
  overflow: hidden;
  color: white;
  cursor: pointer;
  background: color-mix(in srgb, var(--sos-bg-muted) 70%, white);
  border: 1px solid color-mix(in srgb, var(--sos-accent) 28%, var(--sos-border-subtle));
  border-radius: 6px;
  box-shadow: 0 14px 30px -24px rgba(20, 55, 64, 0.72);
  isolation: isolate;
  container-type: inline-size;
  transition:
    transform var(--sos-duration-base) var(--sos-ease-out),
    border-color var(--sos-duration-base) ease,
    box-shadow var(--sos-duration-base) ease;
}

.artwork-preview:hover {
  z-index: 2;
  border-color: color-mix(in srgb, var(--sos-accent) 66%, white);
  box-shadow: 0 22px 38px -24px rgba(20, 55, 64, 0.8);
  transform: translateY(-3px);
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
  .artwork-preview:hover { transform: none; }
  .artwork-preview__label {
    padding-bottom: 7px;
  }
}

@media (prefers-reduced-motion: reduce) {
  .artwork-preview { transition: none; }
}
</style>
