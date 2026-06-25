<script setup lang="ts">
import { computed } from 'vue'

type SplitAlign = 'start' | 'center' | 'end' | 'stretch'

const props = withDefaults(
  defineProps<{
    as?: string
    left?: string
    right?: string
    gap?: string
    align?: SplitAlign
  }>(),
  {
    as: 'div',
    left: undefined,
    right: undefined,
    gap: undefined,
    align: 'start',
  }
)

const style = computed(() => ({
  '--sos-split-left': props.left,
  '--sos-split-right': props.right,
  '--sos-split-gap': props.gap,
  '--sos-split-align': props.align,
}))
</script>

<template>
  <component :is="props.as" class="sos-split" :style="style">
    <slot />
  </component>
</template>
