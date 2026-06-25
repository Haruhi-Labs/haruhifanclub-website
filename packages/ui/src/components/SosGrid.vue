<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    as?: string
    min?: string
    gap?: string
  }>(),
  {
    as: 'div',
    min: undefined,
    gap: undefined,
  }
)

const style = computed(() => ({
  '--sos-grid-min': props.min,
  '--sos-grid-gap': props.gap,
}))
</script>

<template>
  <component :is="props.as" class="sos-grid" :style="style">
    <slot />
  </component>
</template>
