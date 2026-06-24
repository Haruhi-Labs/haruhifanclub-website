<script setup lang="ts">
import { computed } from 'vue'

type InlineGap = 'default' | 'tight' | 'loose'

const props = withDefaults(
  defineProps<{
    as?: string
    gap?: InlineGap
  }>(),
  {
    as: 'div',
    gap: 'default',
  }
)

const classes = computed(() => [
  'sos-inline',
  props.gap !== 'default' ? `sos-inline--${props.gap}` : undefined,
])
</script>

<template>
  <component :is="props.as" :class="classes">
    <slot />
  </component>
</template>
