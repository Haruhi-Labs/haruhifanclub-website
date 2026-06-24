<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    value: number
    max?: number
    label?: string
    valueLabel?: string
    tone?: 'default' | 'success' | 'danger'
  }>(),
  {
    max: 100,
    label: undefined,
    valueLabel: undefined,
    tone: 'default',
  }
)

// 把 value 夹到 [0, max]，让 ARIA 与视觉读到同一组归一化值
const clampedValue = computed(() => Math.min(Math.max(props.value, 0), Math.max(props.max, 0)))
const percent = computed(() => {
  if (props.max <= 0) return 0
  return (clampedValue.value / props.max) * 100
})
const readableValue = computed(() => props.valueLabel || `${Math.round(percent.value)}%`)
const fillStyle = computed(() => ({ width: `${percent.value}%` }))
const classes = computed(() => [
  'sos-progress',
  props.tone !== 'default' ? `sos-progress--${props.tone}` : undefined,
])
</script>

<template>
  <div
    :class="classes"
    role="progressbar"
    :aria-label="props.label"
    :aria-valuenow="clampedValue"
    aria-valuemin="0"
    :aria-valuemax="Math.max(props.max, 0)"
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
