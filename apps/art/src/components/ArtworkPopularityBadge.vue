<template>
  <span
    class="artwork-popularity"
    :aria-label="`人气值 ${formattedScore}`"
    :title="`人气值 ${formattedScore}`"
  >
    <Flame :size="13" :stroke-width="2.2" aria-hidden="true" />
    <span>{{ formattedScore }}</span>
  </span>
</template>

<script setup>
import { computed } from 'vue'
import { Flame } from 'lucide-vue-next'

const props = defineProps({
  item: { type: Object, required: true },
})

const formattedScore = computed(() => {
  const score = Math.max(0, Number(
    props.item?.popularity_score
    ?? props.item?.popularity?.score
    ?? props.item?.like_total
    ?? 0
  ))
  return new Intl.NumberFormat('zh-CN', {
    notation: score >= 10_000 ? 'compact' : 'standard',
    maximumFractionDigits: 1,
  }).format(Math.round(score))
})
</script>

<style scoped>
.artwork-popularity {
  position: absolute;
  top: 7px;
  right: 7px;
  z-index: 4;
  display: inline-flex;
  min-width: 0;
  height: 23px;
  align-items: center;
  gap: 3px;
  padding: 0 6px 0 5px;
  color: color-mix(in srgb, var(--sos-text-primary) 82%, var(--sos-accent));
  font-size: 10px;
  font-weight: 800;
  font-variant-numeric: tabular-nums;
  line-height: 1;
  pointer-events: none;
  background: color-mix(in srgb, var(--sos-bg-surface) 88%, transparent);
  border: 1px solid color-mix(in srgb, var(--sos-accent) 24%, rgba(255, 255, 255, 0.78));
  border-radius: 999px;
  box-shadow: 0 3px 10px rgba(31, 61, 68, 0.13);
  backdrop-filter: blur(7px) saturate(0.9);
}

.artwork-popularity :deep(svg) {
  flex: 0 0 auto;
  color: color-mix(in srgb, #e85f66 80%, var(--sos-accent));
}

@media (max-width: 640px) {
  .artwork-popularity {
    top: 5px;
    right: 5px;
    height: 21px;
    padding-inline: 4px 5px;
    font-size: 9px;
  }
}
</style>
