<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    value: number
    max?: number
    label?: string
    valueLabel?: string
  }>(),
  {
    max: 100,
    label: undefined,
    valueLabel: undefined,
  }
)

const percent = computed(() => {
  if (props.max <= 0) return 0
  return Math.min(100, Math.max(0, (props.value / props.max) * 100))
})
const readableValue = computed(() => props.valueLabel || `${Math.round(percent.value)}%`)
const fillStyle = computed(() => ({ width: `${percent.value}%` }))
</script>

<template>
  <div
    class="sos-progress"
    role="progressbar"
    :aria-label="props.label"
    :aria-valuenow="props.value"
    aria-valuemin="0"
    :aria-valuemax="props.max"
  >
    <div v-if="props.label || readableValue" class="sos-progress__meta">
      <span>{{ props.label }}</span>
      <strong>{{ readableValue }}</strong>
    </div>
    <div class="sos-progress__track">
      <span class="sos-progress__fill" :style="fillStyle" />
    </div>
  </div>
</template>
