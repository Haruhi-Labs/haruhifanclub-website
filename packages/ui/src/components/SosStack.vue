<script setup lang="ts">
import { computed } from 'vue'

type StackGap = 'default' | 'tight' | 'loose'

const props = withDefaults(
  defineProps<{
    as?: string
    gap?: StackGap
  }>(),
  {
    as: 'div',
    gap: 'default',
  }
)

const classes = computed(() => [
  'sos-stack',
  props.gap !== 'default' ? `sos-stack--${props.gap}` : undefined,
])
</script>

<template>
  <component :is="props.as" :class="classes">
    <slot />
  </component>
</template>
