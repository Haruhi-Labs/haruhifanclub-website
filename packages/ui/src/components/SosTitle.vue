<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    as?: string
    size?: 'default' | 'xl' | 'hero'
  }>(),
  {
    as: 'h2',
    size: 'default',
  }
)

const classes = computed(() => [
  'sos-title',
  props.size === 'xl' ? 'sos-title--xl' : undefined,
  props.size === 'hero' ? 'sos-title--hero' : undefined,
])
</script>

<template>
  <component :is="as" :class="classes">
    <slot />
  </component>
</template>
