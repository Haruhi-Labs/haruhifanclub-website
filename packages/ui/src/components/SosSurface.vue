<script setup lang="ts">
import { computed } from 'vue'

type SurfaceVariant = 'default' | 'subtle' | 'raised'

const props = withDefaults(
  defineProps<{
    as?: string
    variant?: SurfaceVariant
    padded?: boolean
  }>(),
  {
    as: 'div',
    variant: 'default',
    padded: false,
  }
)

const classes = computed(() => [
  'sos-surface',
  props.variant !== 'default' ? `sos-surface--${props.variant}` : undefined,
  props.padded ? 'sos-surface--padded' : undefined,
])
</script>

<template>
  <component :is="props.as" :class="classes">
    <slot />
  </component>
</template>
