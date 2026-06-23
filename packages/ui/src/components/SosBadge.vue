<script setup lang="ts">
import { computed } from 'vue'

type BadgeVariant = 'default' | 'accent' | 'solid' | 'outline' | 'signal'

const props = withDefaults(
  defineProps<{
    variant?: BadgeVariant
  }>(),
  {
    variant: 'default',
  }
)

const classes = computed(() => [
  'sos-badge',
  props.variant !== 'default' ? `sos-badge--${props.variant}` : undefined,
])
</script>

<template>
  <span :class="classes">
    <slot />
  </span>
</template>
