<script setup lang="ts">
import { computed } from 'vue'

type BadgeVariant = 'default' | 'accent' | 'solid' | 'outline' | 'signal'

const props = withDefaults(
  defineProps<{
    variant?: BadgeVariant
    selected?: boolean
    disabled?: boolean
  }>(),
  {
    variant: 'default',
    selected: false,
    disabled: false,
  }
)

const classes = computed(() => [
  'sos-badge',
  props.variant !== 'default' ? `sos-badge--${props.variant}` : undefined,
])
</script>

<template>
  <span
    :class="classes"
    :aria-selected="props.selected ? 'true' : undefined"
    :aria-disabled="props.disabled ? 'true' : undefined"
  >
    <slot />
  </span>
</template>
