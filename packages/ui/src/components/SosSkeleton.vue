<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    variant?: 'text' | 'title' | 'block' | 'circle'
    width?: string
    height?: string
  }>(),
  {
    variant: 'block',
    width: undefined,
    height: undefined,
  }
)

const classes = computed(() => [
  'sos-skeleton',
  props.variant !== 'block' ? `sos-skeleton--${props.variant}` : undefined,
])
</script>

<template>
  <span :class="classes" :style="{ width, height }" aria-hidden="true" />
</template>
