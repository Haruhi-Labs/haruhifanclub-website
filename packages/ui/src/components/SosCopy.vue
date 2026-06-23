<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    as?: string
    variant?: 'default' | 'lead' | 'small'
  }>(),
  {
    as: 'p',
    variant: 'default',
  }
)

const classes = computed(() => [
  'sos-copy',
  props.variant !== 'default' ? `sos-copy--${props.variant}` : undefined,
])
</script>

<template>
  <component :is="as" :class="classes">
    <slot />
  </component>
</template>
