<script setup lang="ts">
import { computed } from 'vue'

type ClusterAlign = 'start' | 'center' | 'end' | 'stretch'
type ClusterJustify = 'start' | 'center' | 'end' | 'between'

const props = withDefaults(
  defineProps<{
    as?: string
    align?: ClusterAlign
    justify?: ClusterJustify
    gap?: string
  }>(),
  {
    as: 'div',
    align: 'center',
    justify: 'between',
    gap: undefined,
  }
)

const justifyMap = {
  start: 'flex-start',
  center: 'center',
  end: 'flex-end',
  between: 'space-between',
}

const style = computed(() => ({
  '--sos-cluster-align': props.align,
  '--sos-cluster-justify': justifyMap[props.justify],
  '--sos-cluster-gap': props.gap,
}))
</script>

<template>
  <component :is="props.as" class="sos-cluster" :style="style">
    <slot />
  </component>
</template>
